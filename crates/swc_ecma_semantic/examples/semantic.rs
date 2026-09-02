//! Print the scope tree for a JavaScript or TypeScript file.
//!
//! Run with:
//! `cargo run -p swc_next_ecma_semantic --example semantic -- [filename]`

use std::{fmt::Write, fs, path::Path};

use clap::Parser;
use oxc_index::Idx;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser as EcmaParser};
use swc_next_ecma_semantic::{
    AnalyzeOptions, Semantic, SymbolFlags, SymbolName, analyze,
    scope::{Scope, ScopeKind},
};

#[derive(Parser)]
#[command(about = "Print the scope tree for a JavaScript or TypeScript file")]
struct Args {
    #[arg(default_value = "input.js")]
    filename: String,
}

struct ScopesTextWriter<'a> {
    ast: &'a Ast<'a>,
    semantic: &'a Semantic<'a>,
    children: Vec<Vec<ScopeId>>,
    output: String,
    indent: usize,
}

impl<'a> ScopesTextWriter<'a> {
    fn new(ast: &'a Ast<'a>, semantic: &'a Semantic<'a>) -> Self {
        let scope_count = semantic.iter_scopes().count();
        let mut children = vec![Vec::new(); scope_count];

        for entry in semantic.iter_scopes() {
            if let Some(parent) = entry.scope.parent {
                children[parent.index()].push(entry.id);
            }
        }

        Self {
            ast,
            semantic,
            children,
            output: String::new(),
            indent: 0,
        }
    }

    fn build(mut self) -> String {
        self.write_scope(ScopeId::ROOT);
        self.output
    }

    fn write_scope(&mut self, scope_id: ScopeId) {
        let scope = self.semantic.scope(scope_id);
        let flags = format_scope_flags(scope);
        self.write_line(format_args!("Scope {} ({flags}) {{", scope_id.index()));
        self.indent += 2;

        let bindings: Vec<_> = self.semantic.bindings(scope_id).collect();
        if !bindings.is_empty() {
            self.write_line("Bindings: {");
            self.indent += 2;
            for symbol_id in bindings {
                let symbol = self.semantic.symbol(symbol_id);
                let bytes = match symbol.name {
                    SymbolName::Utf8(name) => self.ast.get_utf8(name.into()).as_bytes(),
                    SymbolName::Wtf8(name) => self.ast.get_wtf8(name).as_bytes(),
                };
                let name = String::from_utf8_lossy(bytes);
                let flags = format_symbol_flags(symbol.flags);
                self.write_line(format_args!("{name} ({symbol_id:?} {flags})"));
            }
            self.indent -= 2;
            self.write_line("}");
        }

        for child in self.children[scope_id.index()].clone() {
            self.write_scope(child);
        }

        self.indent -= 2;
        self.write_line("}");
    }

    fn write_line(&mut self, line: impl std::fmt::Display) {
        writeln!(self.output, "{:indent$}{line}", "", indent = self.indent).unwrap()
    }
}

fn format_scope_flags(scope: Scope) -> String {
    let mut names = Vec::new();

    if scope.flags.strict {
        names.push("StrictMode");
    }
    match scope.kind {
        ScopeKind::Global => names.push("Global"),
        ScopeKind::Module => names.push("Module"),
        ScopeKind::Block => names.push("Block"),
        ScopeKind::Class => names.push("Class"),
        ScopeKind::ExpressionName => names.push("ExpressionName"),
        ScopeKind::Function | ScopeKind::FunctionBody => names.push("Function"),
        ScopeKind::With => names.push("With"),
        ScopeKind::StaticBlock => names.push("ClassStaticBlock"),
        ScopeKind::TsModule => names.push("TsModuleBlock"),
    }
    if scope.flags.direct_eval {
        names.push("DirectEval");
    }

    format!("ScopeFlags({})", names.join(" | "))
}

fn format_symbol_flags(flags: SymbolFlags) -> String {
    let mut names = Vec::new();
    let known_flags = [
        (SymbolFlags::FUNCTION_SCOPED_VAR, "FunctionScopedVariable"),
        (SymbolFlags::BLOCK_SCOPED_VAR, "BlockScopedVariable"),
        (SymbolFlags::CONST_VAR, "ConstVariable"),
        (SymbolFlags::CLASS, "Class"),
        (SymbolFlags::CATCH_VAR, "CatchVariable"),
        (SymbolFlags::FUNCTION, "Function"),
        (SymbolFlags::IMPORT, "Import"),
        (SymbolFlags::TYPE_IMPORT, "TypeImport"),
        (SymbolFlags::TYPE_ALIAS, "TypeAlias"),
        (SymbolFlags::INTERFACE, "Interface"),
        (SymbolFlags::REGULAR_ENUM, "RegularEnum"),
        (SymbolFlags::CONST_ENUM, "ConstEnum"),
        (SymbolFlags::ENUM_MEMBER, "EnumMember"),
        (SymbolFlags::TYPE_PARAMETER, "TypeParameter"),
        (SymbolFlags::NAMESPACE_MODULE, "NamespaceModule"),
        (SymbolFlags::VALUE_MODULE, "ValueModule"),
        (SymbolFlags::AMBIENT, "Ambient"),
        (SymbolFlags::PARAMETER, "Parameter"),
        (SymbolFlags::EXPORTED, "Exported"),
        (SymbolFlags::IS_DEFAULT, "Default"),
    ];

    for (flag, name) in known_flags {
        if flags.contains(flag) {
            names.push(name);
        }
    }

    format!("SymbolFlags({})", names.join(" | "))
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let path = Path::new(&args.filename);
    let source_text =
        fs::read_to_string(path).map_err(|_| format!("Missing '{}'", path.display()))?;

    let allocator = Allocator::new();
    let parse_options = ParserOptions {
        source_type: SourceType::from_path(&args.filename),
        lang: Lang::from_path(&args.filename),
        preserve_parens: false,
        ..ParserOptions::default()
    };
    let mut parse =
        EcmaParser::init(&allocator, &source_text, parse_options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());

    let semantic = analyze(
        &mut parse.ast,
        AnalyzeOptions {
            check_syntax: true,
            ..AnalyzeOptions::default()
        },
    );
    for diagnostic in semantic.diagnostics {
        eprintln!(
            "{}:{}..{}: {}: {}",
            path.display(),
            diagnostic.span.start,
            diagnostic.span.end,
            diagnostic.severity.as_str(),
            diagnostic.message,
        );
        if let Some(help) = &diagnostic.help {
            eprintln!("  help: {help}");
        }
        for label in &diagnostic.labels {
            eprintln!(
                "  {}..{}: {}",
                label.span.start, label.span.end, label.message
            );
        }
    }

    print!(
        "{}",
        ScopesTextWriter::new(&parse.ast, &semantic.semantic).build()
    );
    Ok(())
}
