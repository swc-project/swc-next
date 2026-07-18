use crate::{ExtraData, NodePayload};

const _: () = {
    assert!(size_of::<NodePayload>() == 12);
    assert!(align_of::<NodePayload>() == 4);
    assert!(size_of::<ExtraData>() == 8);
};
