#![allow(unused)]

/// crates/swc_ecma_transforms_base/src/resolver/mod.rs
use oxc_index::IndexVec;
use rustc_hash::FxHashMap;
use swc_experimental_ecma_ast::*;
use swc_experimental_ecma_visit::{Visit, VisitWith};

mod legacy;

pub use legacy::{ScopeId, resolver};
