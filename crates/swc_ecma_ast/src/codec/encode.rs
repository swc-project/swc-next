use crate::Program;

pub enum EncodeError {
    BufferTooSmall,
}

pub struct Encoder {
    ptr: *mut u8,
    len: u32,
}

impl Encoder {
    pub fn new(buf: &mut [u8]) -> Self {
        Self {
            ptr: buf.as_mut_ptr(),
            len: buf.len() as u32,
        }
    }

    pub fn encode(&mut self, ast: &Program) -> Result<(), EncodeError> {
        let size = ast_size(ast);
        if self.len < size as u32 {
            return Err(EncodeError::BufferTooSmall);
        }
        ast.encode(self);
        Ok(())
    }
}

struct Header {}

fn ast_size(ast: &Program) -> usize {
    todo!()
}
