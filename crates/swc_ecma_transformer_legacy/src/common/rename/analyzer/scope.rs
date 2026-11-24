use std::collections::{HashMap, HashSet};

use crate::common::resolver::context::Id;

#[derive(Default)]
pub(in crate::common::rename) struct Scope {
    pub(in crate::common::rename) function: bool,
    pub(in crate::common::rename) all: HashSet<Id>,
    pub(in crate::common::rename) queue: Vec<Id>,
    pub(in crate::common::rename) queued: HashSet<Id>,
    pub(in crate::common::rename) reserved: HashSet<String>,
    pub(in crate::common::rename) children: Vec<Scope>,
}

impl Scope {
    /// SWC declaration queue: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L52>.
    /// SWC var hoisting: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L53>.
    pub(in crate::common::rename) fn add_decl(&mut self, id: Id) {
        if id.name == "arguments" {
            return;
        }
        self.all.insert(id.clone());
        if self.queued.insert(id.clone()) {
            self.queue.push(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L74>.
    pub(in crate::common::rename) fn add_usage(&mut self, id: Id) {
        if id.name != "arguments" {
            self.all.insert(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L91>.
    pub(in crate::common::rename) fn prepare_renaming(&mut self) {
        for child in &mut self.children {
            child.prepare_renaming();
            self.all.extend(child.all.iter().cloned());
            self.reserved.extend(child.reserved.iter().cloned());
        }
    }

    /// SWC scope traversal: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L102>.
    /// SWC name assignment: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L140>.
    /// SWC conflict check: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L199>.
    pub(in crate::common::rename) fn rename_in_normal_mode(
        &self,
        unresolved: &HashSet<String>,
        reverse: &mut HashMap<String, Vec<Id>>,
        renames: &mut HashMap<Id, String>,
    ) {
        let mut latest = HashMap::<&str, usize>::new();
        for id in &self.queue {
            if id.name == "eval" || renames.contains_key(id) {
                continue;
            }
            let mut index = latest.get(id.name.as_str()).copied().unwrap_or_default();
            loop {
                let name = if index == 0 {
                    id.name.clone()
                } else {
                    format!("{}{index}", id.name)
                };
                index += 1;
                if unresolved.contains(&name) || self.reserved.contains(&name) {
                    continue;
                }
                let conflict = reverse.get(&name).is_some_and(|ids| {
                    ids.iter()
                        .any(|other| other != id && self.all.contains(other))
                });
                if conflict {
                    continue;
                }
                latest.insert(&id.name, index);
                reverse.entry(name.clone()).or_default().push(id.clone());
                renames.insert(id.clone(), name);
                break;
            }
        }
        for child in &self.children {
            child.rename_in_normal_mode(unresolved, reverse, renames);
        }
    }
}
