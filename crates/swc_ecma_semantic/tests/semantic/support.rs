use super::*;

#[track_caller]
pub(super) fn assert_uncertain_alias_targets(source: &str, declarations: &[&str]) {
    let expected = declarations
        .iter()
        .map(|text| declaration_start(source, text))
        .collect::<Vec<_>>();
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let mut references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "A");
        let entry = references.next().unwrap();
        assert!(references.next().is_none());
        let reference = entry.reference;
        let candidate = reference.symbol.unwrap();
        let resolution = NameResolution::Uncertain { candidate };
        assert_eq!(reference.resolution(), resolution);
        assert!(reference.resolution().symbol().is_none());
        assert!(reference.flags.is_uncertain());
        assert_eq!(
            semantic
                .reference_value_targets(&ast, entry.id)
                .collect::<Vec<_>>(),
            semantic
                .reference_candidates(entry.id)
                .map(|candidate| NameResolution::Uncertain { candidate })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            semantic.lookup(reference.scope, b"A", reference.flags.space),
            Some(candidate)
        );
        let targets = semantic.reference_candidates(entry.id).collect::<Vec<_>>();
        assert_eq!(targets.first(), Some(&candidate));
        let mut actual = targets
            .iter()
            .map(|&symbol| {
                let declaration = semantic.declarations(symbol).next().unwrap();
                ast.span(declaration.node).start
            })
            .collect::<Vec<_>>();
        assert_eq!(actual[0], expected[0]);
        actual.sort_unstable();
        let mut expected = expected.clone();
        expected.sort_unstable();
        assert_eq!(actual, expected);
        for symbol in semantic
            .iter_symbols()
            .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"A")
        {
            if targets.contains(&symbol.id) {
                assert_eq!(semantic.uses(symbol.id), &[entry.id]);
            } else {
                assert!(semantic.uses(symbol.id).is_empty());
            }
        }
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[track_caller]
pub(super) fn assert_named_declaration_targets(
    source: &str,
    parser_options: Options,
    name: &[u8],
    declarations: &[Option<&str>],
) {
    let expected = declarations
        .iter()
        .map(|declaration| declaration.map(|text| declaration_start(source, text)))
        .collect::<Vec<_>>();
    let allocator = Allocator::new();
    let check = |parser_options, options| {
        let (ast, semantic) = analyze_with_options(&allocator, source, parser_options, options);
        let targets = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
            .map(|entry| {
                let reference = entry.reference;
                assert!(!reference.flags.is_uncertain());
                assert_eq!(
                    semantic.lookup(reference.scope, name, reference.flags.space),
                    reference.symbol
                );
                assert_eq!(
                    reference.resolution(),
                    reference
                        .symbol
                        .map_or(NameResolution::Unresolved, NameResolution::Resolved)
                );
                assert_eq!(
                    semantic.reference_candidates(entry.id).collect::<Vec<_>>(),
                    reference.symbol.into_iter().collect::<Vec<_>>()
                );
                if matches!(
                    reference.flags.space,
                    ReferenceSpace::Type | ReferenceSpace::Typeof
                ) {
                    assert!(!reference.flags.is_read());
                    assert!(!reference.flags.is_write());
                }
                for symbol in semantic.iter_symbols() {
                    assert_eq!(
                        semantic.uses(symbol.id).contains(&entry.id),
                        reference.symbol == Some(symbol.id)
                    );
                }
                reference.symbol.map(|symbol| {
                    let declaration = semantic.declarations(symbol).next().unwrap();
                    ast.span(declaration.node).start
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(targets, expected);
    };
    check(parser_options, AnalyzeOptions::default());
    check(
        parser_options,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            ..AnalyzeOptions::default()
        },
    );
}

#[track_caller]
pub(super) fn assert_merged_type_parameter(ast: &Ast<'_>, semantic: &Semantic<'_>, name: &[u8]) {
    let symbol = symbol_named(ast, semantic, name);
    let declarations = semantic.declarations(symbol).collect::<Vec<_>>();
    assert_eq!(declarations.len(), 2);
    assert_ne!(declarations[0].scope, declarations[1].scope);
    for declaration in declarations {
        assert_eq!(semantic.symbol_of(declaration.node), Some(symbol));
        assert_eq!(semantic.binding(declaration.scope, name), Some(symbol));
    }
    let references = semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
        .collect::<Vec<_>>();
    assert_eq!(references.len(), 2);
    for entry in &references {
        let reference = entry.reference;
        assert_eq!(reference.symbol, Some(symbol));
        assert_eq!(
            semantic.lookup(reference.scope, name, reference.flags.space),
            Some(symbol)
        );
    }
    assert_eq!(
        semantic.uses(symbol),
        references.iter().map(|entry| entry.id).collect::<Vec<_>>()
    );
}

#[track_caller]
fn declaration_start(source: &str, declaration: &str) -> u32 {
    let mut matches = source.match_indices(declaration);
    let (start, _) = matches.next().unwrap();
    assert!(matches.next().is_none());
    start as u32
}

#[track_caller]
pub(super) fn symbol_at_declaration(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    source: &str,
    declaration: &str,
) -> SymbolId {
    let start = declaration_start(source, declaration);
    let mut declarations = semantic
        .iter_declarations()
        .filter(|entry| ast.span(entry.declaration.node).start == start);
    let symbol = declarations.next().unwrap().declaration.symbol;
    assert!(declarations.next().is_none());
    symbol
}
