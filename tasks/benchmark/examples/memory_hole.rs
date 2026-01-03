use std::{
    collections::{HashMap, hash_map::Entry},
    env,
    fs::read_to_string,
};

use swc_experimental_ecma_ast::{Ast, NodeId, NodeKind};
use swc_experimental_ecma_parser::{Parser, StringSource, Syntax, TsSyntax};
use swc_experimental_ecma_visit::{Visit, VisitWith};

fn main() {
    let source = env::args()
        .nth(1)
        .map(|p| read_to_string(p).unwrap())
        .unwrap_or(include_str!("../files/typescript.js").to_owned());
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: true,
        ..Default::default()
    });
    let input = StringSource::new(&source);

    let parser = Parser::new(syntax, input, None);
    let ret = parser.parse_program().unwrap();

    let mut used = Use {
        ast: &ret.ast,
        used: Default::default(),
    };
    ret.root.visit_with(&mut used);

    for (node_id, node) in ret.ast.nodes() {
        let source = &source[node.span().lo.0 as usize - 1..node.span().hi.0 as usize - 1];
        let Some(occur) = used.used.get(&node_id) else {
            println!("Unused node: {:?}, source: {}", node.kind(), source);
            continue;
        };
        if occur.len() > 1 {
            println!(
                "Shared node: {:?}, source: {}, kind: {:?}",
                node.kind(),
                source,
                occur
            );
        }
    }

    println!(
        "Total: {}, Used: {}",
        ret.ast.nodes().count(),
        used.used.len()
    )
}

struct Use<'a> {
    ast: &'a Ast,
    used: HashMap<NodeId, Vec<NodeKind>>,
}

impl Visit for Use<'_> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn enter_node(&mut self, node_id: NodeId) {
        let kind = self.ast.get_node(node_id).kind();
        match self.used.entry(node_id) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().push(kind),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![kind]);
            }
        }
    }
}
