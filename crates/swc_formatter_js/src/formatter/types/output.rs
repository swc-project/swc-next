//! A rendered type retains its pending packet and actual right-edge proof.

use crate::formatter::comments::ExpressionTail;
use crate::formatter::expressions::{ExpressionOutput, ExpressionRightEdge};

pub(in crate::formatter) struct TypeOutput<'ast> {
    pub(in crate::formatter) pending: Option<ExpressionTail<'ast>>,
    pub(in crate::formatter) right_edge: ExpressionRightEdge,
}

impl<'ast> TypeOutput<'ast> {
    pub(in crate::formatter) fn from_pending(pending: Option<ExpressionTail<'ast>>) -> Self {
        Self {
            pending,
            right_edge: ExpressionRightEdge::Unreported,
        }
    }

    pub(in crate::formatter) fn closed() -> Self {
        Self::from_pending(None)
    }

    pub(in crate::formatter) fn from_expression(output: ExpressionOutput<'ast>) -> Self {
        Self {
            pending: output.pending,
            right_edge: output.right_edge,
        }
    }

    pub(in crate::formatter) fn into_expression(self) -> ExpressionOutput<'ast> {
        ExpressionOutput {
            pending: self.pending,
            right_edge: self.right_edge,
        }
    }
}
