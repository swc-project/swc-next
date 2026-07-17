use crate::{ExtraData, NodeData, NodePayload};

const _: () = {
    assert!(size_of::<NodePayload>() == 4);
    assert!(size_of::<NodeData>() == 16);
    assert!(size_of::<ExtraData<'_>>() == 8);
};
