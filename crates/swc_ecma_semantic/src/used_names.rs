use swc_next_allocator::{
    Allocator,
    hash_map::{Entry, HashMap as ArenaHashMap},
};
use swc_next_ecma_ast::{Ast, AstInternedUtf8, GeneratedUtf8State};

/// Tracks interned names used in one collision domain.
///
/// The AST string pool owns content collision state. This type keeps the
/// suffix cursor and current compact prefix associated with each generated-name base.
pub struct UsedNameState<'a> {
    next_names: ArenaHashMap<'a, AstInternedUtf8, NextNameState>,
}

#[derive(Debug)]
struct NextNameState {
    strings: GeneratedUtf8State,
    next_suffix: u64,
}

impl std::fmt::Debug for UsedNameState<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UsedNameState")
            .field("next_names", &self.next_names)
            .finish()
    }
}

impl<'a> UsedNameState<'a> {
    /// Creates an empty name state backed by `allocator`.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            next_names: ArenaHashMap::new_in(allocator),
        }
    }

    /// Reserves and returns an available generated name based on `base`.
    ///
    /// The unsuffixed base is preferred. If it is occupied, suffixes start at
    /// `min_suffix`, for example `next_name(ast, "_temp", 2)` produces
    /// `_temp2`.
    pub fn next_name(&mut self, ast: &mut Ast<'_>, base: &str, min_suffix: u64) -> AstInternedUtf8 {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let strings = ast.generated_utf8_state(base);
        let base_name = strings.base();
        let state = match self.next_names.entry(base_name) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                if ast.reserve_name(base_name) {
                    return base_name;
                }
                entry.insert(NextNameState {
                    strings,
                    next_suffix: min_suffix,
                })
            }
        };
        let mut suffix = state.next_suffix.max(min_suffix);

        loop {
            let next_suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
            let (candidate, was_free) =
                ast.reserve_generated_name_from_state(&mut state.strings, suffix);
            if was_free {
                state.next_suffix = next_suffix;
                return candidate;
            }
            suffix = next_suffix;
        }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Lang};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::UsedNameState;
    use crate::{AnalyzeOptions, analyze};

    #[test]
    fn semantic_canonicalization_populates_collision_state() {
        let allocator = Allocator::new();
        let mut ast = Parser::init(
            &allocator,
            "_temp; _temp2;",
            Options {
                lang: Lang::Js,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse()
        .ast;
        let result = analyze(&mut ast, AnalyzeOptions::default());
        assert!(result.diagnostics.is_empty());

        let mut names = UsedNameState::new(&allocator);
        let generated = names.next_name(&mut ast, "_temp", 2);
        assert_eq!(generated, ast.add_utf8("_temp3"));
    }

    #[test]
    fn allocates_interned_generated_names() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.add_utf8("_temp");
        ast.reserve_name(temp);

        let second = names.next_name(&mut ast, "_temp", 2);
        let third = names.next_name(&mut ast, "_temp", 2);

        assert_eq!(second, ast.add_utf8("_temp2"));
        assert_eq!(third, ast.add_utf8("_temp3"));
    }

    #[test]
    fn skips_interned_collisions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.add_utf8("_temp");
        let temp2 = ast.add_utf8("_temp2");
        ast.reserve_name(temp);
        ast.reserve_name(temp2);

        let name = names.next_name(&mut ast, "_temp", 2);

        assert_eq!(name, ast.add_utf8("_temp3"));
    }

    #[test]
    fn allocates_hexadecimal_suffix_blocks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.add_utf8("_temp");
        ast.reserve_name(temp);

        let f = names.next_name(&mut ast, "_temp", 0xf);
        let ten = names.next_name(&mut ast, "_temp", 0xf);
        let eleven = names.next_name(&mut ast, "_temp", 0xf);

        assert_eq!(f, ast.add_utf8("_tempf"));
        assert_eq!(ten, ast.add_utf8("_temp10"));
        assert_eq!(eleven, ast.add_utf8("_temp11"));
    }

    #[test]
    fn crosses_hexadecimal_prefix_blocks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.add_utf8("_temp");
        ast.reserve_name(temp);

        let one_f = names.next_name(&mut ast, "_temp", 0x1f);
        let twenty = names.next_name(&mut ast, "_temp", 0x1f);
        assert_eq!(one_f, ast.add_utf8("_temp1f"));
        assert_eq!(twenty, ast.add_utf8("_temp20"));

        let other = ast.add_utf8("_other");
        ast.reserve_name(other);
        let ff = names.next_name(&mut ast, "_other", 0xff);
        let hundred = names.next_name(&mut ast, "_other", 0xff);
        assert_eq!(ff, ast.add_utf8("_otherff"));
        assert_eq!(hundred, ast.add_utf8("_other100"));
    }

    #[test]
    fn supports_unicode_long_and_hex_ending_bases() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = "变量abcdef0123456789abcdef0123456789f";
        let base_name = ast.add_utf8(base);
        ast.reserve_name(base_name);

        let generated = names.next_name(&mut ast, base, 0x10);
        assert_eq!(generated, ast.add_utf8(&format!("{base}10")));
    }
}
