use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use swc_next_ecma_ast::*;

use super::super::module_record::{ExportName, ImportAttributeRecord};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L16>.
#[derive(Default)]
pub(super) struct ExportTable {
    pub announced: Vec<ExportName>,
    pub local_exports: LocalExports,
}

pub(super) type LocalExports = IndexMap<SymbolId, Vec<ExportName>, FxBuildHasher>;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L38>.
pub(super) struct ExportInit {
    pub export: ExportName,
    pub value: Expr,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L66>.
pub(super) struct DependencySlot {
    pub request: StringLiteral,
    pub attributes: Vec<ImportAttributeRecord>,
    pub namespace: BindingIdentifier,
    pub setter_ops: Vec<SetterOp>,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L50>.
pub(super) enum SetterOp {
    Import {
        local: BindingIdentifier,
        imported: Option<AstUtf8>,
    },
    ReExport {
        export: ExportName,
        imported: AstUtf8,
    },
    ReExportNamespace {
        export: ExportName,
    },
    StarExport,
}

/// Carries synthetic assignments separately so visitors only rewrite writes
/// from the source program. The final export calls are emitted afterward.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L75>.
pub(super) enum ExecuteStmt {
    Source(Stmt),
    Assign {
        left: AssignmentTarget,
        right: Expr,
    },
    ExportAssign {
        local: BindingIdentifier,
        right: Expr,
    },
    ExportValue {
        export: ExportName,
        value: Expr,
    },
    ExportBatch {
        value: Expr,
    },
}

impl ExecuteStmt {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L137>.
    pub fn visit_mut_with<'ast>(&mut self, visitor: &mut impl VisitMut<'ast>) {
        match self {
            Self::Source(stmt) => {
                if let Some(replacement) = stmt.visit_mut_with(visitor) {
                    *stmt = replacement;
                }
            }
            Self::Assign { left, right } => {
                if let Some(replacement) = left.visit_mut_with(visitor) {
                    *left = replacement;
                }
                if let Some(replacement) = right.visit_mut_with(visitor) {
                    *right = replacement;
                }
            }
            Self::ExportAssign { right: value, .. }
            | Self::ExportValue { value, .. }
            | Self::ExportBatch { value } => {
                if let Some(replacement) = value.visit_mut_with(visitor) {
                    *value = replacement;
                }
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L171>.
    pub fn visit_with<'ast>(&self, visitor: &mut impl Visit<'ast>) {
        match *self {
            Self::Source(stmt) => stmt.visit_with(visitor),
            Self::Assign { left, right } => {
                left.visit_with(visitor);
                right.visit_with(visitor);
            }
            Self::ExportAssign { right: value, .. }
            | Self::ExportValue { value, .. }
            | Self::ExportBatch { value } => {
                value.visit_with(visitor);
            }
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/ir.rs#L199>.
pub(super) struct SystemModule {
    pub dependencies: Vec<DependencySlot>,
    pub exports: ExportTable,
    pub export_inits: Vec<ExportInit>,
    pub wrapper_vars: Vec<BindingIdentifier>,
    pub wrapper_fns: Vec<Function>,
    pub execute_stmts: Vec<ExecuteStmt>,
    pub export_setters: BindingIdentifier,
    pub needs_export_setters: bool,
    pub async_execute: bool,
}

impl SystemModule {
    pub fn new(export_setters: BindingIdentifier) -> Self {
        Self {
            dependencies: Vec::new(),
            exports: ExportTable::default(),
            export_inits: Vec::new(),
            wrapper_vars: Vec::new(),
            wrapper_fns: Vec::new(),
            execute_stmts: Vec::new(),
            export_setters,
            needs_export_setters: false,
            async_execute: false,
        }
    }
}
