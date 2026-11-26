//! Emit the formatter's lexer membership for each BMP code point.
//!
//! Bit 0 denotes an identifier start and bit 1 denotes an identifier continuation.

use std::io::{self, Write};

use swc_next_ecma_ast::{
    is_identifier_continue_es5_and_esnext, is_identifier_start_es5_and_esnext,
};

fn main() -> io::Result<()> {
    let flags: Vec<u8> = (0..=u16::MAX)
        .map(|point| {
            char::from_u32(u32::from(point)).map_or(0, |ch| {
                u8::from(is_identifier_start_es5_and_esnext(ch))
                    | (u8::from(is_identifier_continue_es5_and_esnext(ch)) << 1)
            })
        })
        .collect();
    io::stdout().lock().write_all(&flags)
}
