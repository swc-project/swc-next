use swc_next_allocator::{
    Allocator,
    hash_map::{Entry, HashMap as ArenaHashMap},
};
use swc_next_ecma_ast::{Ast, AstInternedUtf8, GeneratedName, GeneratedUtf8State};

/// Tracks interned names used in one collision domain.
///
/// The AST string pool owns content collision state. This type keeps the
/// suffix cursor and current compact prefix associated with each generated-name base.
pub struct UsedNameState<'a> {
    next_names: ArenaHashMap<'a, AstInternedUtf8, NextNameState>,
    generated_names: [Option<GeneratedNameState>; GeneratedName::COUNT],
}

#[derive(Debug)]
struct NextNameState {
    strings: GeneratedUtf8State,
    next_suffix: u64,
}

/// Reusable state for allocating a series of names with the same base.
///
/// The base is interned and looked up once. Subsequent allocations keep their
/// suffix cursor and cached physical prefix here while [`UsedNameState`]
/// remains the source of truth for collisions.
#[derive(Debug)]
pub struct GeneratedNameState {
    strings: GeneratedUtf8State,
    next_suffix: u64,
    return_base: bool,
}

impl std::fmt::Debug for UsedNameState<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UsedNameState")
            .field("next_names", &self.next_names)
            .field("generated_names", &self.generated_names)
            .finish()
    }
}

impl<'a> UsedNameState<'a> {
    /// Creates an empty name state backed by `allocator`.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            next_names: ArenaHashMap::new_in(allocator),
            generated_names: [const { None }; GeneratedName::COUNT],
        }
    }

    /// Records an interned name in the AST string pool's UID collision domain.
    pub fn observe(&mut self, ast: &mut Ast<'_>, name: AstInternedUtf8) {
        ast.observe_name(name);
    }

    /// Returns whether an interned name is observed or reserved.
    pub fn contains(&self, ast: &Ast<'_>, name: AstInternedUtf8) -> bool {
        ast.is_name_reserved(name)
    }

    /// Reserves and returns an available fixed transform-generated name.
    pub fn next_name(&mut self, ast: &mut Ast<'_>, name: GeneratedName) -> AstInternedUtf8 {
        let index = name.index();
        if self.generated_names[index].is_none() {
            self.generated_names[index] =
                Some(Self::create_generated_name_state(ast, name.base(), 2));
        }
        Self::allocate_from_state(
            ast,
            self.generated_names[index]
                .as_mut()
                .expect("generated name state was initialized"),
        )
    }

    /// Reserves and returns an available generated name based on `base`.
    ///
    /// The unsuffixed base is preferred. If it is occupied, suffixes start at
    /// `min_suffix`, for example `next_name_with_base(ast, "_temp", 2)` produces
    /// `_temp2`.
    pub fn next_name_with_base(
        &mut self,
        ast: &mut Ast<'_>,
        base: &str,
        min_suffix: u64,
    ) -> AstInternedUtf8 {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let strings = ast.generated_utf8_state(base);
        let base_name = Ast::generated_utf8_base(&strings);
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
            suffix = ast.skip_observed_generated_suffixes(&mut state.strings, suffix);
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

    /// Creates reusable allocation state for a series of names based on
    /// `base`.
    pub fn generated_name_state(
        &mut self,
        ast: &mut Ast<'_>,
        base: &str,
        min_suffix: u64,
    ) -> GeneratedNameState {
        Self::create_generated_name_state(ast, base, min_suffix)
    }

    /// Reserves and returns the next name from a reusable generated-name
    /// series.
    pub fn next_name_from_state(
        &mut self,
        ast: &mut Ast<'_>,
        state: &mut GeneratedNameState,
    ) -> AstInternedUtf8 {
        Self::allocate_from_state(ast, state)
    }

    fn create_generated_name_state(
        ast: &mut Ast<'_>,
        base: &str,
        min_suffix: u64,
    ) -> GeneratedNameState {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let strings = ast.generated_utf8_state(base);
        let base_name = Ast::generated_utf8_base(&strings);
        let return_base = ast.reserve_name(base_name);
        GeneratedNameState {
            strings,
            next_suffix: min_suffix,
            return_base,
        }
    }

    fn allocate_from_state(ast: &mut Ast<'_>, state: &mut GeneratedNameState) -> AstInternedUtf8 {
        if state.return_base {
            state.return_base = false;
            return Ast::generated_utf8_base(&state.strings);
        }

        loop {
            let suffix =
                ast.skip_observed_generated_suffixes(&mut state.strings, state.next_suffix);
            state.next_suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
            let (candidate, was_free) =
                ast.reserve_generated_name_from_state(&mut state.strings, suffix);
            if was_free {
                return candidate;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, GeneratedName, Lang};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::UsedNameState;
    use crate::{AnalyzeOptions, analyze};

    #[test]
    fn semantic_canonicalization_populates_collision_state() {
        let allocator = Allocator::new();
        let mut ast = Parser::init_copying(
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
        let generated = names.next_name_with_base(&mut ast, "_temp", 2);
        assert_eq!(generated, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn allocates_interned_generated_names() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.intern_utf8("_temp");
        names.observe(&mut ast, temp);

        let second = names.next_name_with_base(&mut ast, "_temp", 2);
        let third = names.next_name_with_base(&mut ast, "_temp", 2);

        assert_eq!(second, ast.intern_utf8("_temp2"));
        assert_eq!(third, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn skips_interned_collisions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.intern_utf8("_temp");
        let temp2 = ast.intern_utf8("_temp2");
        names.observe(&mut ast, temp);
        names.observe(&mut ast, temp2);

        let name = names.next_name_with_base(&mut ast, "_temp", 2);

        assert_eq!(name, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn reuses_generated_name_state() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = ast.intern_utf8("_temp");
        names.observe(&mut ast, base);

        let mut state = names.generated_name_state(&mut ast, "_temp", 2);
        let second = names.next_name_from_state(&mut ast, &mut state);
        let third = names.next_name_from_state(&mut ast, &mut state);

        assert_eq!(second, ast.intern_utf8("_temp2"));
        assert_eq!(third, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn generated_name_states_share_collision_domain() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.intern_utf8("_temp");
        names.observe(&mut ast, temp);

        let mut first_state = names.generated_name_state(&mut ast, "_temp", 2);
        let mut second_state = names.generated_name_state(&mut ast, "_temp", 2);
        let second = names.next_name_from_state(&mut ast, &mut first_state);
        let third = names.next_name_from_state(&mut ast, &mut second_state);

        assert_eq!(second, ast.intern_utf8("_temp2"));
        assert_eq!(third, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn allocates_hexadecimal_suffix_blocks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.intern_utf8("_temp");
        names.observe(&mut ast, temp);

        let mut state = names.generated_name_state(&mut ast, "_temp", 0xf);
        let f = names.next_name_from_state(&mut ast, &mut state);
        let ten = names.next_name_from_state(&mut ast, &mut state);
        let eleven = names.next_name_from_state(&mut ast, &mut state);

        assert_eq!(f, ast.intern_utf8("_tempf"));
        assert_eq!(ten, ast.intern_utf8("_temp10"));
        assert_eq!(eleven, ast.intern_utf8("_temp11"));
    }

    #[test]
    fn crosses_hexadecimal_prefix_blocks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let temp = ast.intern_utf8("_temp");
        names.observe(&mut ast, temp);

        let mut one_f_state = names.generated_name_state(&mut ast, "_temp", 0x1f);
        let one_f = names.next_name_from_state(&mut ast, &mut one_f_state);
        let twenty = names.next_name_from_state(&mut ast, &mut one_f_state);
        assert_eq!(one_f, ast.intern_utf8("_temp1f"));
        assert_eq!(twenty, ast.intern_utf8("_temp20"));

        let other = ast.intern_utf8("_other");
        names.observe(&mut ast, other);
        let mut ff_state = names.generated_name_state(&mut ast, "_other", 0xff);
        let ff = names.next_name_from_state(&mut ast, &mut ff_state);
        let hundred = names.next_name_from_state(&mut ast, &mut ff_state);
        assert_eq!(ff, ast.intern_utf8("_otherff"));
        assert_eq!(hundred, ast.intern_utf8("_other100"));
    }

    #[test]
    fn supports_unicode_long_and_hex_ending_bases() {
        let allocator = Allocator::new();
        let mut ast = Ast::new_copying("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = "变量abcdef0123456789abcdef0123456789f";
        let base_name = ast.intern_utf8(base);
        names.observe(&mut ast, base_name);

        let generated = names.next_name_with_base(&mut ast, base, 0x10);
        assert_eq!(generated, ast.intern_utf8(&format!("{base}10")));
    }

    #[test]
    fn fixed_generated_names_skip_semantic_collisions() {
        let allocator = Allocator::new();
        let mut source = String::from("_staticBlock;");
        for suffix in 2..=0x100 {
            source.push_str(&format!("_staticBlock{suffix:x};"));
        }
        let mut ast = Parser::init_copying(
            &allocator,
            &source,
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
        let generated = names.next_name(&mut ast, GeneratedName::StaticBlock);
        assert_eq!(generated, ast.intern_utf8("_staticBlock101"));
    }
}
