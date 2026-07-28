use crate::Ast;

pub fn buffer_size(_tree: &Ast) -> usize {
    todo!()
}

/// Serialize an AST into `buf` and return the required number of bytes.
///
/// When `buf` is too small, including when it is empty, this returns the
/// required size without writing a partial payload.
pub fn serialize_into(_tree: &Ast, _buf: &mut [u8]) {
    todo!()
}
