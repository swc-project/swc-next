//! Identity keys for shared instruction fragments.

use super::Instruction;

pub(super) type FragmentKey = (usize, usize);

pub(super) fn key(elements: &[Instruction<'_>]) -> FragmentKey {
    (elements.as_ptr() as usize, elements.len())
}
