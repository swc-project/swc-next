#[derive(Clone, Copy, Default)]
struct Context {
    in_constructor: bool,
    in_derived_class: bool,
}

pub(super) struct JsxSelfCtx {
    ctx_stack: Vec<Context>,
}

impl Default for JsxSelfCtx {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L33>.
    fn default() -> Self {
        Self {
            ctx_stack: vec![Context::default()],
        }
    }
}

impl JsxSelfCtx {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L39>.
    fn current(&self) -> Context {
        *self
            .ctx_stack
            .last()
            .expect("context stack should never be empty")
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L46>.
    fn push(&mut self, ctx: Context) {
        self.ctx_stack.push(ctx);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L50>.
    pub(super) fn pop(&mut self) {
        if self.ctx_stack.len() > 1 {
            self.ctx_stack.pop();
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L56>.
    pub(super) fn enter_class(&mut self, derived: bool) {
        let mut ctx = self.current();
        ctx.in_derived_class = derived;
        self.push(ctx);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L62>.
    pub(super) fn enter_constructor(&mut self) {
        let mut ctx = self.current();
        ctx.in_constructor = true;
        self.push(ctx);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L70>.
    pub(super) fn enter_non_constructor_scope(&mut self) {
        let mut ctx = self.current();
        ctx.in_constructor = false;
        self.push(ctx);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L78>.
    pub(super) fn can_add_self(&self) -> bool {
        let ctx = self.current();
        !(ctx.in_constructor && ctx.in_derived_class)
    }
}
