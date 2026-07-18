use std::{
    borrow::{Borrow, Cow},
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
};

use crate::{
    CloneIn, FromIn,
    wtf8::{Wtf8, Wtf8Buf},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom<'a>(&'a str);

impl Atom<'static> {
    #[inline]
    pub fn new_const(s: &'static str) -> Self {
        Self(s)
    }

    #[inline]
    pub fn empty() -> Self {
        Self::new_const("")
    }
}

impl<'a> Atom<'a> {
    #[inline]
    pub fn new<S>(s: S) -> Self
    where
        Self: From<S>,
    {
        Self::from(s)
    }

    #[inline]
    pub fn new_in<S>(s: S, allocator: &'a crate::Allocator) -> Atom<'a>
    where
        Atom<'a>: FromIn<'a, S>,
    {
        Atom::from_in(s, allocator)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl<'a> From<&'a str> for Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}

impl<'a, 'b> FromIn<'a, &'b str> for Atom<'a> {
    #[inline]
    fn from_in(s: &'b str, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_str(s))
    }
}

impl<'a> FromIn<'a, String> for Atom<'a> {
    #[inline]
    fn from_in(s: String, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_str(s.as_str()))
    }
}

impl<'a, 'b> FromIn<'a, &'b String> for Atom<'a> {
    #[inline]
    fn from_in(s: &'b String, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_str(s.as_str()))
    }
}

impl<'a> FromIn<'a, Cow<'_, str>> for Atom<'a> {
    #[inline]
    fn from_in(s: Cow<'_, str>, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_str(s.as_ref()))
    }
}

impl<'a> CloneIn<'a> for Atom<'_> {
    type Cloned = Atom<'a>;

    #[inline]
    fn clone_in(&self, allocator: &'a crate::Allocator) -> Self::Cloned {
        Atom(allocator.alloc_str(self.as_str()))
    }
}

impl AsRef<str> for Atom<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Atom<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Default for Atom<'_> {
    #[inline]
    fn default() -> Self {
        Atom::empty()
    }
}

impl Hash for Atom<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Display for Atom<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Atom<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl PartialEq<str> for Atom<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Atom<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl Borrow<str> for Atom<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wtf8Atom<'a>(&'a Wtf8);

impl Wtf8Atom<'static> {
    #[inline]
    pub fn new_const(s: &'static str) -> Self {
        Self(Wtf8::from_str(s))
    }

    #[inline]
    pub fn empty() -> Self {
        Self::new_const("")
    }
}

impl<'a> Wtf8Atom<'a> {
    #[inline]
    pub fn new<S>(s: S) -> Self
    where
        Self: From<S>,
    {
        Self::from(s)
    }

    #[inline]
    pub fn new_in<S>(s: S, allocator: &'a crate::Allocator) -> Wtf8Atom<'a>
    where
        Wtf8Atom<'a>: FromIn<'a, S>,
    {
        Wtf8Atom::from_in(s, allocator)
    }

    #[inline]
    pub fn as_wtf8(&self) -> &Wtf8 {
        self.0
    }
}

impl<'a> From<&'a str> for Wtf8Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        Self(s.into())
    }
}

impl<'a> From<&'a Wtf8> for Wtf8Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a Wtf8) -> Self {
        Self(s)
    }
}

impl<'a, 'b> FromIn<'a, &'b str> for Wtf8Atom<'a> {
    #[inline]
    fn from_in(s: &'b str, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_wtf8(Wtf8::from_str(s)))
    }
}

impl<'a, 'b> FromIn<'a, &'b Wtf8> for Wtf8Atom<'a> {
    #[inline]
    fn from_in(s: &'b Wtf8, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_wtf8(s))
    }
}

impl<'a> FromIn<'a, String> for Wtf8Atom<'a> {
    #[inline]
    fn from_in(s: String, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_wtf8(Wtf8::from_str(s.as_str())))
    }
}

impl<'a> FromIn<'a, Wtf8Buf> for Wtf8Atom<'a> {
    #[inline]
    fn from_in(s: Wtf8Buf, allocator: &'a crate::Allocator) -> Self {
        Self(allocator.alloc_wtf8(&s))
    }
}

impl<'a> CloneIn<'a> for Wtf8Atom<'_> {
    type Cloned = Wtf8Atom<'a>;

    #[inline]
    fn clone_in(&self, allocator: &'a crate::Allocator) -> Self::Cloned {
        Wtf8Atom(allocator.alloc_wtf8(self.as_wtf8()))
    }
}

impl Default for Wtf8Atom<'_> {
    #[inline]
    fn default() -> Self {
        Wtf8Atom::empty()
    }
}

impl AsRef<Wtf8> for Wtf8Atom<'_> {
    #[inline]
    fn as_ref(&self) -> &Wtf8 {
        self.as_wtf8()
    }
}

impl Deref for Wtf8Atom<'_> {
    type Target = Wtf8;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_wtf8()
    }
}

impl Debug for Wtf8Atom<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_wtf8(), f)
    }
}

impl PartialEq<Wtf8> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &Wtf8) -> bool {
        self.as_wtf8() == other
    }
}

impl PartialEq<&Wtf8> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &&Wtf8) -> bool {
        self.as_wtf8() == *other
    }
}

impl PartialEq<str> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_wtf8().as_str() == Some(other)
    }
}

impl PartialEq<&str> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_wtf8().as_str() == Some(*other)
    }
}
