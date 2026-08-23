pub mod resolver;
mod scope;
mod utils;

pub use resolver::{JsNameResolver, ScopeOwner, SymbolNode, resolver};
pub use scope::{DeclKind, ScopeKind};
