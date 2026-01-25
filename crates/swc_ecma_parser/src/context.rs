use swc_experimental_ecma_ast::{
    UTF8_AWAIT, UTF8_BREAK, UTF8_CASE, UTF8_CATCH, UTF8_CLASS, UTF8_CONST, UTF8_CONTINUE,
    UTF8_DEBUGGER, UTF8_DEFAULT, UTF8_DELETE, UTF8_DO, UTF8_ELSE, UTF8_ENUM, UTF8_EXPORT,
    UTF8_EXTENDS, UTF8_FALSE, UTF8_FINALLY, UTF8_FOR, UTF8_FUNCTION, UTF8_IF, UTF8_IMPLEMENTS,
    UTF8_IMPORT, UTF8_IN, UTF8_INSTANCEOF, UTF8_INTERFACE, UTF8_LET, UTF8_NEW, UTF8_NULL,
    UTF8_PACKAGE, UTF8_PRIVATE, UTF8_PROTECTED, UTF8_PUBLIC, UTF8_RETURN, UTF8_SUPER, UTF8_SWITCH,
    UTF8_THIS, UTF8_THROW, UTF8_TRUE, UTF8_TRY, UTF8_TYPEOF, UTF8_VAR, UTF8_VOID, UTF8_WHILE,
    UTF8_WITH, UTF8_YIELD, Utf8Ref,
};

bitflags::bitflags! {
  #[derive(Debug, Clone, Copy, Default)]
  pub struct Context: u32 {

      /// `true` while backtracking
      const IgnoreError = 1 << 0;

      /// Is in module code?
      const Module = 1 << 1;
      const CanBeModule = 1 << 2;
      const Strict = 1 << 3;

      const ForLoopInit = 1 << 4;
      const ForAwaitLoopInit = 1 << 5;

      const IncludeInExpr = 1 << 6;
      /// If true, await expression is parsed, and "await" is treated as a
      /// keyword.
      const InAsync = 1 << 7;
      /// If true, yield expression is parsed, and "yield" is treated as a
      /// keyword.
      const InGenerator = 1 << 8;

      /// If true, await is treated as a keyword.
      const InStaticBlock = 1 << 9;

      const IsContinueAllowed = 1 << 10;
      const IsBreakAllowed = 1 << 11;

      const InType = 1 << 12;
      /// Typescript extension.
      const ShouldNotLexLtOrGtAsType = 1 << 13;
      /// Typescript extension.
      const InDeclare = 1 << 14;

      /// If true, `:` should not be treated as a type annotation.
      const InCondExpr = 1 << 15;
      const WillExpectColonForCond = 1 << 16;

      const InClass = 1 << 17;

      const InClassField = 1 << 18;

      const InFunction = 1 << 19;

      /// This indicates current scope or the scope out of arrow function is
      /// function declaration or function expression or not.
      const InsideNonArrowFunctionScope = 1 << 20;

      const InParameters = 1 << 21;

      const HasSuperClass = 1 << 22;

      const InPropertyName = 1 << 23;

      const InForcedJsxContext = 1 << 24;

      // If true, allow super.x and super[x]
      const AllowDirectSuper = 1 << 25;

      const DisallowConditionalTypes = 1 << 27;

      const AllowUsingDecl = 1 << 28;

      const TopLevel = 1 << 29;

      const TsModuleBlock = 1 << 30;
  }
}

impl Context {
    #[cfg_attr(not(feature = "verify"), inline(always))]
    pub fn is_reserved_word(self, word: Utf8Ref) -> bool {
        if word == UTF8_LET {
            return self.contains(Context::Strict);
        }

        // SyntaxError in the module only, not in the strict.
        // ```JavaScript
        // function foo() {
        //     "use strict";
        //     let await = 1;
        // }
        // ```
        if word == UTF8_AWAIT {
            return self.contains(Context::InAsync)
                || self.contains(Context::InStaticBlock)
                || self.contains(Context::Module);
        }

        if word == UTF8_YIELD {
            return self.contains(Context::InGenerator) || self.contains(Context::Strict);
        }

        // Future reserved word
        if word == UTF8_ENUM {
            return true;
        }

        #[rustfmt::skip]
        const RESERVED: &[Utf8Ref] = &[
            UTF8_NULL, UTF8_TRUE, UTF8_FALSE, UTF8_BREAK, UTF8_CASE, UTF8_CATCH, UTF8_CONTINUE, UTF8_DEBUGGER,
            UTF8_DEFAULT, UTF8_DO, UTF8_EXPORT, UTF8_ELSE, UTF8_FINALLY, UTF8_FOR, UTF8_FUNCTION, UTF8_IF,
            UTF8_RETURN, UTF8_SWITCH, UTF8_THROW, UTF8_TRY, UTF8_VAR, UTF8_CONST, UTF8_WHILE, UTF8_WITH,
            UTF8_NEW, UTF8_THIS, UTF8_SUPER, UTF8_CLASS, UTF8_EXTENDS, UTF8_IMPORT, UTF8_IN, UTF8_INSTANCEOF,
            UTF8_TYPEOF, UTF8_VOID, UTF8_DELETE
        ];

        if RESERVED.contains(&word) {
            return true;
        }

        if self.contains(Context::Strict) {
            if word == UTF8_IMPLEMENTS
                || word == UTF8_PACKAGE
                || word == UTF8_PROTECTED
                || word == UTF8_INTERFACE
                || word == UTF8_PRIVATE
                || word == UTF8_PUBLIC
            {
                return true;
            }
        }

        false
    }
}
