use std::collections::{HashMap, HashSet};

use crate::common::resolver::context::Resolver;
use swc_next_ecma_ast::{Ast, SymbolId};

#[derive(Default)]
pub(in crate::common::rename) struct Scope {
    pub(in crate::common::rename) function: bool,
    pub(in crate::common::rename) all: HashSet<SymbolId>,
    pub(in crate::common::rename) queue: Vec<SymbolId>,
    pub(in crate::common::rename) queued: HashSet<SymbolId>,
    pub(in crate::common::rename) reserved: HashSet<String>,
    pub(in crate::common::rename) children: Vec<Scope>,
}

impl Scope {
    /// SWC declaration queue: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L52>.
    /// SWC var hoisting: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L53>.
    pub(in crate::common::rename) fn add_decl(&mut self, id: SymbolId) {
        self.all.insert(id);
        if self.queued.insert(id) {
            self.queue.push(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L74>.
    pub(in crate::common::rename) fn add_usage(&mut self, id: SymbolId) {
        self.all.insert(id);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L91>.
    pub(in crate::common::rename) fn prepare_renaming(&mut self) {
        for child in &mut self.children {
            child.prepare_renaming();
            self.all.extend(child.all.iter().copied());
            self.reserved.extend(child.reserved.iter().cloned());
        }
    }

    /// SWC scope traversal: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L102>.
    /// SWC name assignment: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L140>.
    /// SWC conflict check: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L199>.
    pub(in crate::common::rename) fn rename_in_normal_mode(
        &self,
        ast: &Ast<'_>,
        resolver: &Resolver,
        unresolved: &HashSet<&str>,
        reverse: &mut HashMap<String, Vec<SymbolId>>,
        renames: &mut HashMap<SymbolId, String>,
    ) {
        let mut latest = HashMap::<&str, usize>::new();
        for id in &self.queue {
            let original = ast.get_utf8(resolver.symbol_name(*id));
            if original == "eval" || renames.contains_key(id) {
                continue;
            }
            let mut index = latest.get(original).copied().unwrap_or_default();
            loop {
                let name = if index == 0 {
                    original.to_owned()
                } else {
                    format!("{original}{index}")
                };
                index += 1;
                if unresolved.contains(name.as_str()) || self.reserved.contains(&name) {
                    continue;
                }
                let conflict = reverse.get(&name).is_some_and(|ids| {
                    ids.iter()
                        .any(|other| other != id && self.all.contains(other))
                });
                if conflict {
                    continue;
                }
                latest.insert(original, index);
                reverse.entry(name.clone()).or_default().push(*id);
                renames.insert(*id, name);
                break;
            }
        }
        for child in &self.children {
            child.rename_in_normal_mode(ast, resolver, unresolved, reverse, renames);
        }
    }
}
