use bumpalo::Bump;
use std::cell::Cell;

mod allocator_api;
pub mod atom;
pub mod boxed;
pub mod hash_map;
pub mod hash_set;
pub mod raw_vec;
pub mod vec;
pub mod wtf8;

pub trait CloneIn<'a>: Sized {
    type Cloned;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned;
}

macro_rules! impl_clone_in_trivial {
    ($i:ty) => {
        impl<'a> CloneIn<'a> for $i {
            type Cloned = $i;

            fn clone_in(&self, _allocator: &'a Allocator) -> Self::Cloned {
                self.clone()
            }
        }
    };
}

impl_clone_in_trivial!(bool);
impl_clone_in_trivial!(f64);

impl<'a, 'src, C: 'a, T> CloneIn<'a> for boxed::Box<'src, T>
where
    T: CloneIn<'a, Cloned = C>,
{
    type Cloned = boxed::Box<'a, C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        allocator.boxed((**self).clone_in(allocator))
    }
}

impl<'a, 'src, C: 'a, T> CloneIn<'a> for vec::Vec<'src, T>
where
    T: CloneIn<'a, Cloned = C>,
{
    type Cloned = vec::Vec<'a, C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        let mut cloned = vec::Vec::with_capacity_in(self.len(), allocator);
        for item in self {
            cloned.push(item.clone_in(allocator));
        }
        cloned
    }
}

impl<'a, C, T> CloneIn<'a> for Option<T>
where
    T: CloneIn<'a, Cloned = C>,
{
    type Cloned = Option<C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(allocator))
    }
}

impl<'a, C, T> CloneIn<'a> for Cell<T>
where
    T: Copy + CloneIn<'a, Cloned = C>,
{
    type Cloned = Cell<C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Cell::new(self.get().clone_in(allocator))
    }
}

/// This trait works similarly to the standard library [`From`] trait.
pub trait FromIn<'a, T>: Sized {
    /// Converts to this type from the input type within the given `allocator`.
    fn from_in(value: T, allocator: &'a Allocator) -> Self;
}

/// This trait works similarly to the standard library [`Into`] trait.
pub trait IntoIn<'a, T>: Sized {
    /// Converts this type into the inferred target type within the given `allocator`.
    fn into_in(self, allocator: &'a Allocator) -> T;
}

impl<'a, T> FromIn<'a, T> for T {
    #[inline(always)]
    fn from_in(t: T, _: &'a Allocator) -> T {
        t
    }
}

impl<'a, T, U> IntoIn<'a, U> for T
where
    U: FromIn<'a, T>,
{
    #[inline]
    fn into_in(self, allocator: &'a Allocator) -> U {
        U::from_in(self, allocator)
    }
}

impl<'a> FromIn<'a, String> for &'a str {
    #[inline(always)]
    fn from_in(value: String, allocator: &'a Allocator) -> Self {
        allocator.alloc_str(value.as_str())
    }
}

impl<'a, T> FromIn<'a, T> for boxed::Box<'a, T> {
    #[inline(always)]
    fn from_in(value: T, allocator: &'a Allocator) -> Self {
        boxed::Box::new_in(value, allocator)
    }
}

impl<'a, T> FromIn<'a, Option<T>> for Option<boxed::Box<'a, T>> {
    #[inline(always)]
    fn from_in(value: Option<T>, allocator: &'a Allocator) -> Self {
        value.map(|it| boxed::Box::new_in(it, allocator))
    }
}

pub struct Allocator {
    arena: Bump,
}

impl Allocator {
    pub fn new() -> Self {
        Self { arena: Bump::new() }
    }

    pub fn reset(&mut self) {
        self.arena.reset();
    }

    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.arena.alloc(value)
    }

    pub fn alloc_str(&self, s: &str) -> &str {
        self.arena.alloc_str(s)
    }

    pub fn alloc_wtf8(&self, s: &wtf8::Wtf8) -> &wtf8::Wtf8 {
        let bytes = self.arena.alloc_slice_copy(s.as_bytes());
        unsafe { wtf8::Wtf8::from_bytes_unchecked(bytes) }
    }

    pub fn boxed<T>(&self, value: T) -> boxed::Box<'_, T> {
        boxed::Box::new_in(value, self)
    }

    pub fn vec<T>(&self) -> vec::Vec<'_, T> {
        vec::Vec::new_in(self)
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}
