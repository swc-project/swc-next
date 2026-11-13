use serde::{Deserialize, Serialize};

/// AST optimization options. These do not control code generation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinifierOptions {
    /// Compression settings. `None` disables all compression.
    pub compress: Option<CompressOptions>,
    /// Shorten the names of local bindings. Top-level names and the bindings
    /// that `eval` or a `with` statement can see are kept. Enabled by default.
    pub mangle: bool,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self {
            compress: Some(CompressOptions::default()),
            mangle: true,
        }
    }
}

/// Settings for the compression visitor.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompressOptions {
    /// Remove empty statements, merge blocks that declare no block-scoped
    /// names into the enclosing statement list, and replace a block body,
    /// such as the body of an `if` or a loop, with its only statement. The
    /// block body of a `do` loop is kept. Enabled by default.
    pub blocks: bool,
    /// Shorten boolean expressions: `true` to `!0` and `false` to `!1`.
    /// Enabled by default.
    pub booleans: bool,
    /// Shorten comparisons: `===` to `==` when both operands have the same
    /// type, as in `typeof a === "b"`, and `!(a == b)` to `a != b`. Enabled
    /// by default.
    pub comparisons: bool,
    /// Rewrite `if` statements whose branches are expressions or returns
    /// as expressions, such as `if (a) b()` to `a && b()`, `if (a) b(); else
    /// c()` to `a ? b() : c()`, and `if (a) return b; else return c` to
    /// `return a ? b : c`. Join the tests of nested `if` statements, remove
    /// empty branches, and shorten conditional expressions, such as
    /// `!a ? b : c` to `a ? c : b`. Enabled by default.
    pub conditionals: bool,
    /// Remove code that a constant condition never runs, such as the
    /// branches of `if (0) a(); else b()` and `!0 ? a : b`, `!1 && a`, and
    /// `while (0) a()`. `var` names declared in removed code are kept.
    /// Enabled by default.
    pub dead_code: bool,
    /// Fold operators applied to constants, such as `1 + 2` to `3` and
    /// `"a" + "b"` to `"ab"`, when the result prints shorter. Enabled by
    /// default.
    pub evaluate: bool,
    /// Shorten unshadowed global constants. Enabled by default.
    pub global_defs: bool,
    /// Shorten returns: `if (a) return b; return c` to `return a ? b : c`,
    /// remove a `return;` that ends a function, and move the statements
    /// after `if (a) return;` at the end of a function into `if (!a)`.
    /// Enabled by default.
    pub if_return: bool,
    /// Join adjacent `var`, `let`, or `const` declarations of the same kind,
    /// such as `var a; var b` to `var a, b`, and move `var` declarations into
    /// the initializer of a following `for` loop. Enabled by default.
    pub join_vars: bool,
    /// Negate a function expression that an expression statement calls, so
    /// `(function(){})()` prints as `!function(){}()`. Enabled by default.
    pub negate_iife: bool,
    /// Shorten property names: `a["b"]` to `a.b`, and `{ "a": b }` to
    /// `{ a: b }`. Only ASCII names are converted. Enabled by default.
    pub properties: bool,
    /// Replace the reads of a local binding that is initialized with a
    /// constant and never assigned with the constant, when the reads run
    /// after the initialization and the output gets shorter, and remove
    /// the declaration. Enabled by default.
    pub reduce_vars: bool,
    /// Join adjacent expression statements with the comma operator, such as
    /// `a(); b()` to `a(), b()`, and move them into the leading expression of
    /// a following `if`, `return`, `throw`, `switch`, or `for` statement.
    /// Enabled by default.
    pub sequences: bool,
    /// Remove computations whose values are unused and that have no side
    /// effects, such as the expression statement `1;`, the `a` in `(a, b())`
    /// for a declared `a`, and `x` in `void x`. Unknown properties, calls,
    /// and reads that can throw are kept. Enabled by default.
    pub side_effects: bool,
    /// Shorten undefined typeof comparisons for local var bindings. Enabled by default.
    pub typeofs: bool,
    /// Remove unused local variables whose initializers have no side effects
    /// and unused local function declarations. Declarations whose only uses
    /// are in removed code are removed too. Enabled by default.
    pub unused: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            blocks: true,
            booleans: true,
            comparisons: true,
            conditionals: true,
            dead_code: true,
            evaluate: true,
            global_defs: true,
            if_return: true,
            join_vars: true,
            negate_iife: true,
            properties: true,
            reduce_vars: true,
            sequences: true,
            side_effects: true,
            typeofs: true,
            unused: true,
        }
    }
}

impl CompressOptions {
    /// Settings that compress nothing, for a traversal that only mangles.
    pub(crate) fn disabled() -> Self {
        Self {
            blocks: false,
            booleans: false,
            comparisons: false,
            conditionals: false,
            dead_code: false,
            evaluate: false,
            global_defs: false,
            if_return: false,
            join_vars: false,
            negate_iife: false,
            properties: false,
            reduce_vars: false,
            sequences: false,
            side_effects: false,
            typeofs: false,
            unused: false,
        }
    }
}
