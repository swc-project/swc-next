use crate::{ExtraData, NodeData};

const _: () = {
    assert!(size_of::<NodeData>() == 4);
    assert!(size_of::<ExtraData>() == 8);
};
