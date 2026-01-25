mod extra_data;
mod node;
mod sub_range;

pub use extra_data::*;
pub use node::*;
pub use sub_range::*;

#[macro_export]
macro_rules! define_optional_index_type {
    ($name:ident, $index_type:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl oxc_index::Idx for $name {
            const MAX: usize = (u32::MAX - 1) as usize;

            unsafe fn from_usize_unchecked(idx: usize) -> Self {
                Self(idx as u32)
            }

            fn index(self) -> usize {
                self.0 as usize
            }
        }

        impl From<$index_type> for $name {
            fn from(value: $index_type) -> Self {
                Self(value.0)
            }
        }

        impl $name {
            pub fn none() -> Self {
                Self(u32::MAX)
            }

            pub fn to_option(self) -> Option<$index_type> {
                if self.0 == u32::MAX {
                    return None;
                }

                Some($index_type(self.0))
            }

            pub fn map<U, F>(self, f: F) -> Option<U>
            where
                F: FnOnce($index_type) -> U,
            {
                if self.0 == u32::MAX {
                    return None;
                }

                Some(f($index_type(self.0)))
            }
        }
    };
}

oxc_index::define_index_type! {
    pub struct BigIntId = u32;
}
