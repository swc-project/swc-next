use swc_next_ecma_ast::*;

pub(super) fn enum_member_name(
    ast: &Ast<'_>,
    name: TsEnumMemberName,
) -> swc_next_allocator::wtf8::Wtf8Buf {
    use swc_next_allocator::wtf8::Wtf8Buf;
    match ast.ts_enum_member_name_data(name) {
        TsEnumMemberNameData::IdentifierName(id) => Wtf8Buf::from_str(ast.get_utf8(id.name(ast))),
        TsEnumMemberNameData::StringLiteral(string) => ast.get_wtf8(string.value(ast)).to_owned(),
        TsEnumMemberNameData::TemplateLiteral(template) => {
            let quasi = ast.get_node_in_sub_range(template.quasis(ast).iter().next().unwrap());
            ast.get_wtf8(quasi.cooked(ast)).to_owned()
        }
    }
}

pub(super) fn get_module_ident(
    ast: &Ast<'_>,
    reference: TsModuleReference,
) -> Option<IdentifierReference> {
    let name = reference.as_ts_type_name(ast)?;
    if let Some(id) = name.as_identifier_reference(ast) {
        return Some(id);
    }
    let mut name = name.as_ts_qualified_name(ast)?;
    loop {
        match ast.ts_qualified_name_left_data(name.left(ast)) {
            TsQualifiedNameLeftData::IdentifierReference(id) => return Some(id),
            TsQualifiedNameLeftData::TsQualifiedName(qualified) => name = qualified,
            _ => return None,
        }
    }
}
