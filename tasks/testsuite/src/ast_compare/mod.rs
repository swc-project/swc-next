//! Structural comparison for parse/print round trips, independent of codegen.
//!
//! Compare reachable nodes and decoded strings, never arena IDs or source spans.
//! Compare the parsed structure directly, including parenthesis nodes when
//! present. Literal spellings may change; node kinds, flags, tagged-template raw
//! text, directive spellings, and optional-chain boundaries may not.

use swc_next_ecma_ast::*;

mod generated;

pub(crate) fn compare(left: &Ast<'_>, right: &Ast<'_>) -> Result<(), String> {
    let asts = AstPair { left, right };
    if left.source_type() != right.source_type() {
        return Err(format!(
            "source type: {:?} != {:?}",
            left.source_type(),
            right.source_type()
        ));
    }
    left.root_program().compare(right.root_program(), &asts)
}

struct AstPair<'a, 'b> {
    left: &'a Ast<'a>,
    right: &'b Ast<'b>,
}

impl AstPair<'_, '_> {
    fn nodes(&self, left: NodeId, right: NodeId) -> Result<(), String> {
        let left = self.left.kind_data(left);
        let right = self.right.kind_data(right);
        generated::compare_nodes(left, right, self)
    }
}

trait Compare: Sized {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String>;
}

fn compare_bigint(left: AstUtf8, right: AstUtf8, asts: &AstPair<'_, '_>) -> Result<(), String> {
    let left = asts.left.get_utf8(left);
    let right = asts.right.get_utf8(right);
    fn digits(text: &str) -> impl Iterator<Item = u8> + '_ {
        text.bytes().filter(|byte| *byte != b'_')
    }
    if digits(left).eq(digits(right)) {
        Ok(())
    } else {
        Err(format!("BigInt spelling: {left:?} != {right:?}"))
    }
}

macro_rules! compare_value {
    ($ty:ty) => {
        impl Compare for $ty {
            fn compare(self, other: Self, _: &AstPair<'_, '_>) -> Result<(), String> {
                if self == other {
                    Ok(())
                } else {
                    Err(format!("value: {self:?} != {other:?}"))
                }
            }
        }
    };
}
use compare_value;

impl Compare for AstUtf8 {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        let left = asts.left.get_utf8(self);
        let right = asts.right.get_utf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl Compare for AstInternedUtf8 {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        let left = asts.left.get_utf8(self);
        let right = asts.right.get_utf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl Compare for AstWtf8 {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        let left = asts.left.get_wtf8(self);
        let right = asts.right.get_wtf8(other);
        if left == right {
            Ok(())
        } else {
            Err(format!("value: {left:?} != {right:?}"))
        }
    }
}

impl Compare for f64 {
    fn compare(self, other: Self, _: &AstPair<'_, '_>) -> Result<(), String> {
        // Preserve negative zero and compare NaN payloads deterministically.
        if self.to_bits() == other.to_bits() {
            Ok(())
        } else {
            Err(format!("value: {self:?} != {other:?}"))
        }
    }
}

impl<T: Compare> Compare for Option<T> {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        match (self, other) {
            (Some(left), Some(right)) => left.compare(right, asts),
            (None, None) => Ok(()),
            _ => Err("presence changed".into()),
        }
    }
}

impl<T: Compare + ExtraDataValue> Compare for SubRange<T> {
    fn compare(self, other: Self, asts: &AstPair<'_, '_>) -> Result<(), String> {
        if self.len() != other.len() {
            return Err("list length changed".into());
        }
        for (index, (left, right)) in self.iter().zip(other.iter()).enumerate() {
            let left = asts.left.get_node_in_sub_range(left);
            let right = asts.right.get_node_in_sub_range(right);
            left.compare(right, asts)
                .map_err(|error| format!("[{index}].{error}"))?;
        }
        Ok(())
    }
}
