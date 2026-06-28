use convert_case::{Case, Casing};

pub(crate) mod decode;
pub(crate) mod encode;
pub(crate) mod measure;
pub(crate) mod node_type;
pub(crate) mod visitor;

pub(super) fn js_field_name(name: &str) -> String {
    name.to_case(Case::Camel)
}
