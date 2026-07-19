use crate::{Expr, ExtraData, NodeId, NodePayload, Program, Stmt};

const _: () = {
    assert!(size_of::<NodePayload>() == 16);
    assert!(align_of::<NodePayload>() == align_of::<u128>());
    assert!(size_of::<ExtraData>() == 8);
    assert!(size_of::<Program>() == size_of::<NodeId>());
    assert!(size_of::<Stmt>() == size_of::<NodeId>());
    assert!(size_of::<Expr>() == size_of::<NodeId>());
};
