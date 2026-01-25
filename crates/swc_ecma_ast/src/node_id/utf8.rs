use crate::define_optional_index_type;

oxc_index::define_index_type! {
    pub struct Utf8Ref = u32;
}

define_optional_index_type!(OptionalUtf8Ref, Utf8Ref);
