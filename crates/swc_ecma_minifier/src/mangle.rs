//! Name assignment for local bindings.
//!
//! Each scope numbers its mangled symbols after every slot its ancestor scopes
//! use, so an inner name never captures a reference to an outer one, while
//! sibling scopes reuse the same slots. The most frequently used slots get the
//! shortest names. Generated names avoid every name that is not mangled, so
//! they never capture a global or an unmangled binding either.

use std::cmp::Reverse;

use rustc_hash::FxHashSet;
use swc_next_allocator::Idx;
use swc_next_ecma_ast::{Ast, NodeId, NodeKind, SymbolId};
use swc_next_ecma_semantic::{NameResolution, Semantic};

use crate::dynamic::DynamicScopes;

/// Sentinel for a node that keeps its name.
const KEPT: u32 = u32::MAX;

/// New names for the binding and reference nodes of mangled symbols.
#[derive(Default)]
pub(crate) struct Renames {
    /// Slot of each node, indexed by node id, or `KEPT`.
    nodes: Vec<u32>,
    /// Name of each slot.
    pub(crate) names: Vec<String>,
}

/// Words that cannot name a binding, and names whose bindings behave
/// differently.
const RESERVED: &str = "do if in for let new try var case else enum eval null this true void \
    with await break catch class const false super throw while yield delete export import \
    public return static switch typeof default extends finally package private continue \
    debugger function arguments interface protected implements instanceof";

const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_";
const REST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789";

/// Returns the `index`th name in order of length. The first character varies
/// fastest.
fn name(mut index: usize) -> String {
    let mut name = String::new();
    name.push(char::from(FIRST[index % FIRST.len()]));
    index /= FIRST.len();
    while index > 0 {
        index -= 1;
        name.push(char::from(REST[index % REST.len()]));
        index /= REST.len();
    }
    name
}

impl Renames {
    /// Plans names for the symbols of `semantic` that only local code can see.
    ///
    /// Symbols that dynamic name lookup can see keep their names.
    pub(crate) fn build(ast: &Ast<'_>, semantic: &Semantic<'_>, dynamic: &DynamicScopes) -> Self {
        let program = ast.root_program().node_id();
        let mut reserved: FxHashSet<&[u8]> = RESERVED
            .split_ascii_whitespace()
            .map(str::as_bytes)
            .collect();
        let scope_count = semantic
            .iter_scopes()
            .last()
            .map_or(0, |entry| entry.id.index() + 1);
        let mut offsets = vec![0usize; scope_count + 1];
        let mut symbols = Vec::new();
        for entry in semantic.iter_symbols() {
            match occurrences(ast, semantic, dynamic, entry.id, program) {
                Some(count) => {
                    let scope = semantic.scope_of_symbol(entry.id);
                    offsets[scope.index() + 1] += 1;
                    symbols.push((scope, Reverse(count), entry.id));
                }
                None => {
                    reserved.insert(ast.get_wtf8(entry.symbol.name).as_bytes());
                }
            }
        }
        for entry in semantic.iter_references() {
            if entry.reference.symbol.is_none() {
                reserved.insert(ast.get_utf8(entry.reference.name).as_bytes());
            }
        }
        // Group symbols by scope before sorting each scope by use count and id.
        // The write cursors become the end offsets of the scope ranges.
        let mut offset = 0;
        for start in &mut offsets[1..] {
            let count = *start;
            *start = offset;
            offset += count;
        }
        let mut by_scope = vec![(Reverse(0), SymbolId::from_usize(0)); symbols.len()];
        for (scope, count, id) in symbols {
            let end = &mut offsets[scope.index() + 1];
            by_scope[*end] = (count, id);
            *end += 1;
        }
        let mut ends = vec![0u32; scope_count];
        let mut frequencies: Vec<u32> = Vec::new();
        let mut nodes = vec![KEPT; ast.node_count()];
        // Scopes are created before their descendants, in id order.
        for entry in semantic.iter_scopes() {
            let start = entry.scope.parent.map_or(0, |parent| ends[parent.index()]);
            let scope = entry.id.index();
            let symbols = &mut by_scope[offsets[scope]..offsets[scope + 1]];
            symbols.sort_unstable();
            let mut slot = start;
            for &(Reverse(count), id) in symbols.iter() {
                if frequencies.len() <= slot as usize {
                    frequencies.push(0);
                }
                frequencies[slot as usize] += count;
                for declaration in semantic.declarations(id) {
                    nodes[declaration.node.index()] = slot;
                }
                for &reference in semantic.uses(id) {
                    nodes[semantic.reference(reference).node.index()] = slot;
                }
                slot += 1;
            }
            ends[entry.id.index()] = slot;
        }

        let mut order: Vec<u32> = (0..frequencies.len() as u32).collect();
        order.sort_by_key(|&slot| Reverse(frequencies[slot as usize]));
        let mut names = vec![String::new(); frequencies.len()];
        let mut generated = (0..)
            .map(name)
            .filter(|name| !reserved.contains(name.as_bytes()));
        for slot in order {
            names[slot as usize] = generated.next().unwrap();
        }
        Self { nodes, names }
    }

    /// The slot of a renamed node.
    pub(crate) fn slot(&self, node: NodeId) -> Option<usize> {
        let slot = *self.nodes.get(node.index())?;
        (slot != KEPT).then_some(slot as usize)
    }
}

/// Counts the declarations and uses of a symbol that can be renamed, or returns
/// `None` when its name must be kept.
fn occurrences(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    dynamic: &DynamicScopes,
    id: SymbolId,
    program: NodeId,
) -> Option<u32> {
    // Linked symbols share one name across scopes.
    if semantic.linked_symbols(id).nth(1).is_some()
        // Top-level names are visible to other scripts and modules.
        || semantic.scope(semantic.scope_of_symbol(id)).node == program
        // `eval` and `with` look names up at run time.
        || dynamic.contains(semantic, id)
        // `var arguments` keeps the arguments object, and the shorthand `{ __proto__ }`
        // defines an own property where `{ __proto__: a }` sets the prototype.
        || matches!(
            ast.get_wtf8(semantic.symbol(id).name).as_bytes(),
            b"arguments" | b"__proto__"
        )
    {
        return None;
    }
    let mut count = 0;
    for declaration in semantic.declarations(id) {
        if ast.node_kind(declaration.node) != NodeKind::BindingIdentifier {
            return None;
        }
        count += 1;
    }
    for &reference in semantic.uses(id) {
        let reference = semantic.reference(reference);
        if !matches!(reference.resolution(), NameResolution::Resolved(_))
            || ast.node_kind(reference.node) != NodeKind::IdentifierReference
        {
            return None;
        }
        count += 1;
    }
    Some(count)
}

#[cfg(test)]
mod tests {
    use super::name;

    #[test]
    fn names() {
        assert_eq!(name(0), "a");
        assert_eq!(name(53), "_");
        assert_eq!(name(54), "aa");
        assert_eq!(name(55), "ba");
        assert_eq!(name(54 + 54), "ab");
        assert_eq!(name(54 + 54 * 64 - 1), "_9");
        assert_eq!(name(54 + 54 * 64), "aaa");
    }
}
