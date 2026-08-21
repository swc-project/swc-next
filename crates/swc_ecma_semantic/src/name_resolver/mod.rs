pub mod resolver;
mod scope;
mod utils;

pub use resolver::{JsNameResolver, resolver};
pub use scope::{DeclKind, ScopeKind};
