use swc_next_ecma_ast::*;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/static_check.rs#L8>.
pub(super) fn should_use_create_element(
    attributes: SubRange<JsxAttributeItem>,
    ast: &Ast<'_>,
) -> bool {
    let mut seen_spread = false;
    for slot in attributes.iter() {
        let attribute = ast.get_node_in_sub_range(slot);
        match ast.jsx_attribute_item_data(attribute) {
            JsxAttributeItemData::JsxAttribute(attribute) if seen_spread => {
                if let Some(identifier) = attribute.name(ast).as_jsx_identifier(ast)
                    && ast.get_utf8(identifier.name(ast)) == "key"
                {
                    return true;
                }
            }
            JsxAttributeItemData::JsxSpreadAttribute(_) => seen_spread = true,
            _ => {}
        }
    }
    false
}
