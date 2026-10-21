use std::mem::{align_of, size_of};

use crate::{Expr, ExtraData, ExtraDataId, NodeId, NodePayload, Program, Stmt, SubRange};

const _: () = {
    assert!(size_of::<NodeId>() == size_of::<u32>());
    assert!(align_of::<NodeId>() == align_of::<u32>());
    assert!(size_of::<ExtraDataId>() == size_of::<u32>());
    assert!(align_of::<ExtraDataId>() == align_of::<u32>());
    assert!(size_of::<SubRange<()>>() == 8);
    assert!(align_of::<SubRange<()>>() == align_of::<u32>());
    assert!(size_of::<NodePayload>() == 16);
    assert!(align_of::<NodePayload>() == align_of::<f64>());
    assert!(size_of::<ExtraData>() == 8);
    assert!(size_of::<Program>() == size_of::<NodeId>());
    assert!(size_of::<Stmt>() == size_of::<NodeId>());
    assert!(size_of::<Expr>() == size_of::<NodeId>());
};
