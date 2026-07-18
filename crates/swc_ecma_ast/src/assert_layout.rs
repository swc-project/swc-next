use crate::{ExtraData, NodePayload};

const _: () = {
    assert!(size_of::<NodePayload>() == 16);
    assert!(align_of::<NodePayload>() == align_of::<u128>());
    assert!(size_of::<ExtraData>() == 8);
};
