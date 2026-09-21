#![allow(unused)]
use crate::*;
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union NodePayload {
    pub(crate) raw: u128,
    pub(crate) program: ProgramPayload,
    pub(crate) string_literal: StringLiteralPayload,
    pub(crate) numeric_literal: NumericLiteralPayload,
    pub(crate) big_int_literal: BigIntLiteralPayload,
    pub(crate) boolean_literal: BooleanLiteralPayload,
    pub(crate) reg_exp_literal: RegExpLiteralPayload,
    pub(crate) template_literal: TemplateLiteralPayload,
    pub(crate) template_element: TemplateElementPayload,
    pub(crate) identifier_reference: IdentifierReferencePayload,
    pub(crate) private_identifier: PrivateIdentifierPayload,
    pub(crate) binding_identifier: BindingIdentifierPayload,
    pub(crate) identifier_name: IdentifierNamePayload,
    pub(crate) label_identifier: LabelIdentifierPayload,
    pub(crate) assignment_pattern: AssignmentPatternPayload,
    pub(crate) binding_rest_element: BindingRestElementPayload,
    pub(crate) array_pattern: ArrayPatternPayload,
    pub(crate) object_pattern: ObjectPatternPayload,
    pub(crate) binding_property: BindingPropertyPayload,
    pub(crate) binary_expression: BinaryExpressionPayload,
    pub(crate) logical_expression: LogicalExpressionPayload,
    pub(crate) conditional_expression: ConditionalExpressionPayload,
    pub(crate) unary_expression: UnaryExpressionPayload,
    pub(crate) update_expression: UpdateExpressionPayload,
    pub(crate) assignment_expression: AssignmentExpressionPayload,
    pub(crate) variable_declaration: VariableDeclarationPayload,
    pub(crate) variable_declarator: VariableDeclaratorPayload,
    pub(crate) if_statement: IfStatementPayload,
    pub(crate) switch_statement: SwitchStatementPayload,
    pub(crate) switch_case: SwitchCasePayload,
    pub(crate) for_statement: ForStatementPayload,
    pub(crate) for_in_statement: ForInStatementPayload,
    pub(crate) for_of_statement: ForOfStatementPayload,
    pub(crate) try_statement: TryStatementPayload,
    pub(crate) catch_clause: CatchClausePayload,
    pub(crate) break_statement: BreakStatementPayload,
    pub(crate) continue_statement: ContinueStatementPayload,
    pub(crate) labeled_statement: LabeledStatementPayload,
    pub(crate) expression_statement: ExpressionStatementPayload,
    pub(crate) return_statement: ReturnStatementPayload,
    pub(crate) throw_statement: ThrowStatementPayload,
    pub(crate) directive: DirectivePayload,
    pub(crate) decorator: DecoratorPayload,
    pub(crate) class: ClassPayload,
    pub(crate) class_body: ClassBodyPayload,
    pub(crate) method_definition: MethodDefinitionPayload,
    pub(crate) property_definition: PropertyDefinitionPayload,
    pub(crate) static_block: StaticBlockPayload,
    pub(crate) function: FunctionPayload,
    pub(crate) function_body: FunctionBodyPayload,
    pub(crate) block_statement: BlockStatementPayload,
    pub(crate) while_statement: WhileStatementPayload,
    pub(crate) do_while_statement: DoWhileStatementPayload,
    pub(crate) with_statement: WithStatementPayload,
    pub(crate) formal_parameters: FormalParametersPayload,
    pub(crate) formal_parameter: FormalParameterPayload,
    pub(crate) formal_parameter_rest: FormalParameterRestPayload,
    pub(crate) array_expression: ArrayExpressionPayload,
    pub(crate) object_expression: ObjectExpressionPayload,
    pub(crate) spread_element: SpreadElementPayload,
    pub(crate) object_property: ObjectPropertyPayload,
    pub(crate) parenthesized_expression: ParenthesizedExpressionPayload,
    pub(crate) arrow_function_expression: ArrowFunctionExpressionPayload,
    pub(crate) sequence_expression: SequenceExpressionPayload,
    pub(crate) member_expression: MemberExpressionPayload,
    pub(crate) call_expression: CallExpressionPayload,
    pub(crate) chain_expression: ChainExpressionPayload,
    pub(crate) tagged_template_expression: TaggedTemplateExpressionPayload,
    pub(crate) new_expression: NewExpressionPayload,
    pub(crate) await_expression: AwaitExpressionPayload,
    pub(crate) yield_expression: YieldExpressionPayload,
    pub(crate) meta_property: MetaPropertyPayload,
    pub(crate) import_expression: ImportExpressionPayload,
    pub(crate) import_declaration: ImportDeclarationPayload,
    pub(crate) import_specifier: ImportSpecifierPayload,
    pub(crate) import_default_specifier: ImportDefaultSpecifierPayload,
    pub(crate) import_namespace_specifier: ImportNamespaceSpecifierPayload,
    pub(crate) import_attribute: ImportAttributePayload,
    pub(crate) export_named_declaration: ExportNamedDeclarationPayload,
    pub(crate) export_default_declaration: ExportDefaultDeclarationPayload,
    pub(crate) export_all_declaration: ExportAllDeclarationPayload,
    pub(crate) export_specifier: ExportSpecifierPayload,
    pub(crate) hashbang: HashbangPayload,
    pub(crate) ts_function: TsFunctionPayload,
    pub(crate) ts_method_definition: TsMethodDefinitionPayload,
    pub(crate) ts_type_annotation: TsTypeAnnotationPayload,
    pub(crate) ts_type_reference: TsTypeReferencePayload,
    pub(crate) ts_qualified_name: TsQualifiedNamePayload,
    pub(crate) ts_type_query: TsTypeQueryPayload,
    pub(crate) ts_import_type: TsImportTypePayload,
    pub(crate) ts_type_parameter: TsTypeParameterPayload,
    pub(crate) ts_type_parameter_declaration: TsTypeParameterDeclarationPayload,
    pub(crate) ts_type_parameter_instantiation: TsTypeParameterInstantiationPayload,
    pub(crate) ts_literal_type: TsLiteralTypePayload,
    pub(crate) ts_template_literal_type: TsTemplateLiteralTypePayload,
    pub(crate) ts_array_type: TsArrayTypePayload,
    pub(crate) ts_indexed_access_type: TsIndexedAccessTypePayload,
    pub(crate) ts_tuple_type: TsTupleTypePayload,
    pub(crate) ts_named_tuple_member: TsNamedTupleMemberPayload,
    pub(crate) ts_optional_type: TsOptionalTypePayload,
    pub(crate) ts_rest_type: TsRestTypePayload,
    pub(crate) ts_js_doc_nullable_type: TsJsDocNullableTypePayload,
    pub(crate) ts_js_doc_non_nullable_type: TsJsDocNonNullableTypePayload,
    pub(crate) ts_union_type: TsUnionTypePayload,
    pub(crate) ts_intersection_type: TsIntersectionTypePayload,
    pub(crate) ts_conditional_type: TsConditionalTypePayload,
    pub(crate) ts_infer_type: TsInferTypePayload,
    pub(crate) ts_type_operator: TsTypeOperatorPayload,
    pub(crate) ts_parenthesized_type: TsParenthesizedTypePayload,
    pub(crate) ts_function_type: TsFunctionTypePayload,
    pub(crate) ts_constructor_type: TsConstructorTypePayload,
    pub(crate) ts_type_predicate: TsTypePredicatePayload,
    pub(crate) ts_type_literal: TsTypeLiteralPayload,
    pub(crate) ts_mapped_type: TsMappedTypePayload,
    pub(crate) ts_property_signature: TsPropertySignaturePayload,
    pub(crate) ts_method_signature: TsMethodSignaturePayload,
    pub(crate) ts_call_signature_declaration: TsCallSignatureDeclarationPayload,
    pub(crate) ts_construct_signature_declaration: TsConstructSignatureDeclarationPayload,
    pub(crate) ts_index_signature: TsIndexSignaturePayload,
    pub(crate) ts_type_alias_declaration: TsTypeAliasDeclarationPayload,
    pub(crate) ts_interface_declaration: TsInterfaceDeclarationPayload,
    pub(crate) ts_interface_body: TsInterfaceBodyPayload,
    pub(crate) ts_interface_heritage: TsInterfaceHeritagePayload,
    pub(crate) ts_class_implements: TsClassImplementsPayload,
    pub(crate) ts_enum_declaration: TsEnumDeclarationPayload,
    pub(crate) ts_enum_body: TsEnumBodyPayload,
    pub(crate) ts_enum_member: TsEnumMemberPayload,
    pub(crate) ts_module_declaration: TsModuleDeclarationPayload,
    pub(crate) ts_module_block: TsModuleBlockPayload,
    pub(crate) ts_global_declaration: TsGlobalDeclarationPayload,
    pub(crate) ts_parameter_property: TsParameterPropertyPayload,
    pub(crate) ts_this_parameter: TsThisParameterPayload,
    pub(crate) ts_as_expression: TsAsExpressionPayload,
    pub(crate) ts_satisfies_expression: TsSatisfiesExpressionPayload,
    pub(crate) ts_type_assertion: TsTypeAssertionPayload,
    pub(crate) ts_non_null_expression: TsNonNullExpressionPayload,
    pub(crate) ts_instantiation_expression: TsInstantiationExpressionPayload,
    pub(crate) ts_export_assignment: TsExportAssignmentPayload,
    pub(crate) ts_namespace_export_declaration: TsNamespaceExportDeclarationPayload,
    pub(crate) ts_import_equals_declaration: TsImportEqualsDeclarationPayload,
    pub(crate) ts_external_module_reference: TsExternalModuleReferencePayload,
    pub(crate) ts_index_signature_parameter: TsIndexSignatureParameterPayload,
    pub(crate) jsx_element: JsxElementPayload,
    pub(crate) jsx_opening_element: JsxOpeningElementPayload,
    pub(crate) jsx_closing_element: JsxClosingElementPayload,
    pub(crate) jsx_fragment: JsxFragmentPayload,
    pub(crate) jsx_identifier: JsxIdentifierPayload,
    pub(crate) jsx_namespaced_name: JsxNamespacedNamePayload,
    pub(crate) jsx_member_expression: JsxMemberExpressionPayload,
    pub(crate) jsx_attribute: JsxAttributePayload,
    pub(crate) jsx_spread_attribute: JsxSpreadAttributePayload,
    pub(crate) jsx_expression_container: JsxExpressionContainerPayload,
    pub(crate) jsx_text: JsxTextPayload,
    pub(crate) jsx_spread_child: JsxSpreadChildPayload,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ProgramPayload {
    pub(crate) directives: SubRange<Directive>,
    pub(crate) hashbang: u32,
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct StringLiteralPayload {
    pub(crate) value: AstWtf8,
    pub(crate) raw: AstUtf8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NumericLiteralPayload {
    pub(crate) value: f64,
    pub(crate) raw: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BigIntLiteralPayload {
    pub(crate) raw: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BooleanLiteralPayload {
    pub(crate) value: bool,
    pub(crate) _pad0: [u8; 15usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RegExpLiteralPayload {
    pub(crate) pattern: AstUtf8,
    pub(crate) flags: AstUtf8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TemplateLiteralPayload {
    pub(crate) quasis: SubRange<TemplateElement>,
    pub(crate) expressions: SubRange<Expr>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TemplateElementPayload {
    pub(crate) cooked: AstWtf8,
    pub(crate) tail: bool,
    pub(crate) is_cooked_undefined: bool,
    pub(crate) _pad0: [u8; 2usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct IdentifierReferencePayload {
    pub(crate) name: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct PrivateIdentifierPayload {
    pub(crate) name: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BindingIdentifierPayload {
    pub(crate) name: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct IdentifierNamePayload {
    pub(crate) name: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct LabelIdentifierPayload {
    pub(crate) name: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct AssignmentPatternPayload {
    pub(crate) left: BindingPattern,
    pub(crate) right: Expr,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BindingRestElementPayload {
    pub(crate) argument: BindingPattern,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ArrayPatternPayload {
    pub(crate) elements: SubRange<Option<BindingPattern>>,
    pub(crate) rest: u32,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ObjectPatternPayload {
    pub(crate) properties: SubRange<BindingProperty>,
    pub(crate) rest: u32,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BindingPropertyPayload {
    pub(crate) key: PropertyKey,
    pub(crate) value: BindingPattern,
    pub(crate) shorthand: bool,
    pub(crate) computed: bool,
    pub(crate) _pad0: [u8; 6usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BinaryExpressionPayload {
    pub(crate) left: Expr,
    pub(crate) right: Expr,
    pub(crate) operator: BinaryOperator,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct LogicalExpressionPayload {
    pub(crate) left: Expr,
    pub(crate) right: Expr,
    pub(crate) operator: LogicalOperator,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ConditionalExpressionPayload {
    pub(crate) test: Expr,
    pub(crate) consequent: Expr,
    pub(crate) alternate: Expr,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct UnaryExpressionPayload {
    pub(crate) argument: Expr,
    pub(crate) operator: UnaryOperator,
    pub(crate) _pad0: [u8; 11usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct UpdateExpressionPayload {
    pub(crate) argument: SimpleAssignmentTarget,
    pub(crate) operator: UpdateOperator,
    pub(crate) prefix: bool,
    pub(crate) _pad0: [u8; 10usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct AssignmentExpressionPayload {
    pub(crate) left: AssignmentTarget,
    pub(crate) right: Expr,
    pub(crate) operator: AssignmentOperator,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct VariableDeclarationPayload {
    pub(crate) declarators: SubRange<VariableDeclarator>,
    pub(crate) kind: VariableKind,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 6usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct VariableDeclaratorPayload {
    pub(crate) id: BindingPattern,
    pub(crate) type_annotation: u32,
    pub(crate) init: u32,
    pub(crate) definite: bool,
    pub(crate) _pad0: [u8; 3usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct IfStatementPayload {
    pub(crate) test: Expr,
    pub(crate) consequent: Stmt,
    pub(crate) alternate: u32,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SwitchStatementPayload {
    pub(crate) cases: SubRange<SwitchCase>,
    pub(crate) discriminant: Expr,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SwitchCasePayload {
    pub(crate) consequent: SubRange<Stmt>,
    pub(crate) test: u32,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ForStatementPayload {
    pub(crate) init: u32,
    pub(crate) test: u32,
    pub(crate) update: u32,
    pub(crate) body: Stmt,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ForInStatementPayload {
    pub(crate) left: ForStatementLeft,
    pub(crate) right: Expr,
    pub(crate) body: Stmt,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ForOfStatementPayload {
    pub(crate) left: ForStatementLeft,
    pub(crate) right: Expr,
    pub(crate) body: Stmt,
    pub(crate) r#await: bool,
    pub(crate) _pad0: [u8; 3usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TryStatementPayload {
    pub(crate) block: BlockStatement,
    pub(crate) handler: u32,
    pub(crate) finalizer: u32,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct CatchClausePayload {
    pub(crate) param: u32,
    pub(crate) type_annotation: u32,
    pub(crate) body: BlockStatement,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BreakStatementPayload {
    pub(crate) label: u32,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ContinueStatementPayload {
    pub(crate) label: u32,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct LabeledStatementPayload {
    pub(crate) label: LabelIdentifier,
    pub(crate) body: Stmt,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ExpressionStatementPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ReturnStatementPayload {
    pub(crate) argument: u32,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ThrowStatementPayload {
    pub(crate) argument: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct DirectivePayload {
    pub(crate) value: AstUtf8,
    pub(crate) expression: StringLiteral,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct DecoratorPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ClassPayload {
    pub(crate) id: u32,
    pub(crate) type_parameters: u32,
    pub(crate) class_type: ClassType,
    pub(crate) r#abstract: bool,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 1usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ClassBodyPayload {
    pub(crate) body: SubRange<ClassElement>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct MethodDefinitionPayload {
    pub(crate) key: PropertyKey,
    pub(crate) kind: MethodDefinitionKind,
    pub(crate) computed: bool,
    pub(crate) r#static: bool,
    pub(crate) r#override: bool,
    pub(crate) optional: bool,
    pub(crate) r#abstract: bool,
    pub(crate) accessibility: Accessibility,
    pub(crate) _pad0: [u8; 1usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct PropertyDefinitionPayload {
    pub(crate) computed: bool,
    pub(crate) r#static: bool,
    pub(crate) accessor: bool,
    pub(crate) declare: bool,
    pub(crate) r#override: bool,
    pub(crate) optional: bool,
    pub(crate) definite: bool,
    pub(crate) readonly: bool,
    pub(crate) r#abstract: bool,
    pub(crate) accessibility: Accessibility,
    pub(crate) _pad0: [u8; 2usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct StaticBlockPayload {
    pub(crate) body: SubRange<Stmt>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct FunctionPayload {
    pub(crate) id: u32,
    pub(crate) type_parameters: u32,
    pub(crate) function_type: FunctionType,
    pub(crate) generator: bool,
    pub(crate) r#async: bool,
    pub(crate) _pad0: [u8; 1usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct FunctionBodyPayload {
    pub(crate) directives: SubRange<Directive>,
    pub(crate) body: SubRange<Stmt>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BlockStatementPayload {
    pub(crate) body: SubRange<Stmt>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct WhileStatementPayload {
    pub(crate) test: Expr,
    pub(crate) body: Stmt,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct DoWhileStatementPayload {
    pub(crate) body: Stmt,
    pub(crate) test: Expr,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct WithStatementPayload {
    pub(crate) object: Expr,
    pub(crate) body: Stmt,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct FormalParametersPayload {
    pub(crate) items: SubRange<FormalParameterItem>,
    pub(crate) rest: u32,
    pub(crate) kind: FormalParameterKind,
    pub(crate) _pad0: [u8; 3usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct FormalParameterPayload {
    pub(crate) pattern: FormalParameterPattern,
    pub(crate) type_annotation: u32,
    pub(crate) optional: bool,
    pub(crate) _pad0: [u8; 3usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct FormalParameterRestPayload {
    pub(crate) decorators: SubRange<Decorator>,
    pub(crate) argument: FormalParameterPattern,
    pub(crate) type_annotation: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ArrayExpressionPayload {
    pub(crate) elements: SubRange<Option<Argument>>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ObjectExpressionPayload {
    pub(crate) properties: SubRange<ObjectPropertyKind>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SpreadElementPayload {
    pub(crate) argument: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ObjectPropertyPayload {
    pub(crate) key: PropertyKey,
    pub(crate) value: Expr,
    pub(crate) kind: PropertyKind,
    pub(crate) method: bool,
    pub(crate) shorthand: bool,
    pub(crate) computed: bool,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ParenthesizedExpressionPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ArrowFunctionExpressionPayload {
    pub(crate) type_parameters: u32,
    pub(crate) params: FormalParameters,
    pub(crate) expression: bool,
    pub(crate) r#async: bool,
    pub(crate) _pad0: [u8; 2usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SequenceExpressionPayload {
    pub(crate) expressions: SubRange<Expr>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct MemberExpressionPayload {
    pub(crate) object: Expr,
    pub(crate) property: PropertyKey,
    pub(crate) computed: bool,
    pub(crate) optional: bool,
    pub(crate) _pad0: [u8; 6usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct CallExpressionPayload {
    pub(crate) callee: Expr,
    pub(crate) type_arguments: u32,
    pub(crate) optional: bool,
    pub(crate) pure: bool,
    pub(crate) _pad0: [u8; 2usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ChainExpressionPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TaggedTemplateExpressionPayload {
    pub(crate) tag: Expr,
    pub(crate) type_arguments: u32,
    pub(crate) quasi: TemplateLiteral,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NewExpressionPayload {
    pub(crate) arguments: SubRange<Argument>,
    pub(crate) callee: Expr,
    pub(crate) type_arguments: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct AwaitExpressionPayload {
    pub(crate) argument: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct YieldExpressionPayload {
    pub(crate) argument: u32,
    pub(crate) delegate: bool,
    pub(crate) _pad0: [u8; 11usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct MetaPropertyPayload {
    pub(crate) meta: IdentifierName,
    pub(crate) property: IdentifierName,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImportExpressionPayload {
    pub(crate) source: Expr,
    pub(crate) options: u32,
    pub(crate) sync: bool,
    pub(crate) _pad0: [u8; 3usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImportDeclarationPayload {
    pub(crate) source: StringLiteral,
    pub(crate) import_kind: ImportOrExportKind,
    pub(crate) _pad0: [u8; 7usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImportSpecifierPayload {
    pub(crate) imported: ModuleExportName,
    pub(crate) local: BindingIdentifier,
    pub(crate) import_kind: ImportOrExportKind,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImportDefaultSpecifierPayload {
    pub(crate) local: BindingIdentifier,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImportNamespaceSpecifierPayload {
    pub(crate) local: BindingIdentifier,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImportAttributePayload {
    pub(crate) key: ModuleExportName,
    pub(crate) value: StringLiteral,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ExportNamedDeclarationPayload {
    pub(crate) declaration: u32,
    pub(crate) source: u32,
    pub(crate) export_kind: ImportOrExportKind,
    pub(crate) _pad0: [u8; 3usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ExportDefaultDeclarationPayload {
    pub(crate) declaration: ExportDefaultDeclarationKind,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ExportAllDeclarationPayload {
    pub(crate) exported: u32,
    pub(crate) source: StringLiteral,
    pub(crate) export_kind: ImportOrExportKind,
    pub(crate) _pad0: [u8; 3usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ExportSpecifierPayload {
    pub(crate) local: ModuleExportName,
    pub(crate) exported: ModuleExportName,
    pub(crate) export_kind: ImportOrExportKind,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct HashbangPayload {
    pub(crate) value: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsFunctionPayload {
    pub(crate) id: u32,
    pub(crate) type_parameters: u32,
    pub(crate) generator: bool,
    pub(crate) r#async: bool,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 1usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsMethodDefinitionPayload {
    pub(crate) key: PropertyKey,
    pub(crate) kind: MethodDefinitionKind,
    pub(crate) computed: bool,
    pub(crate) r#static: bool,
    pub(crate) r#override: bool,
    pub(crate) optional: bool,
    pub(crate) r#abstract: bool,
    pub(crate) accessibility: Accessibility,
    pub(crate) _pad0: [u8; 1usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeAnnotationPayload {
    pub(crate) type_annotation: TsType,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeReferencePayload {
    pub(crate) type_name: TsTypeName,
    pub(crate) type_arguments: u32,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsQualifiedNamePayload {
    pub(crate) left: TsQualifiedNameLeft,
    pub(crate) right: IdentifierName,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeQueryPayload {
    pub(crate) expr_name: TsTypeQueryExprName,
    pub(crate) type_arguments: u32,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsImportTypePayload {
    pub(crate) source: StringLiteral,
    pub(crate) options: u32,
    pub(crate) qualifier: u32,
    pub(crate) type_arguments: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeParameterPayload {
    pub(crate) name: BindingIdentifier,
    pub(crate) constraint: u32,
    pub(crate) default: u32,
    pub(crate) r#in: bool,
    pub(crate) out: bool,
    pub(crate) r#const: bool,
    pub(crate) _pad0: [u8; 1usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeParameterDeclarationPayload {
    pub(crate) params: SubRange<TsTypeParameter>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeParameterInstantiationPayload {
    pub(crate) params: SubRange<TsType>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsLiteralTypePayload {
    pub(crate) literal: TsLiteral,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTemplateLiteralTypePayload {
    pub(crate) quasis: SubRange<TemplateElement>,
    pub(crate) types: SubRange<TsType>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsArrayTypePayload {
    pub(crate) element_type: TsType,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsIndexedAccessTypePayload {
    pub(crate) object_type: TsType,
    pub(crate) index_type: TsType,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTupleTypePayload {
    pub(crate) element_types: SubRange<TsTupleElement>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsNamedTupleMemberPayload {
    pub(crate) label: IdentifierName,
    pub(crate) element_type: TsType,
    pub(crate) optional: bool,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsOptionalTypePayload {
    pub(crate) type_annotation: TsType,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsRestTypePayload {
    pub(crate) type_annotation: TsTupleElement,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsJsDocNullableTypePayload {
    pub(crate) type_annotation: TsType,
    pub(crate) postfix: bool,
    pub(crate) _pad0: [u8; 11usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsJsDocNonNullableTypePayload {
    pub(crate) type_annotation: TsType,
    pub(crate) postfix: bool,
    pub(crate) _pad0: [u8; 11usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsUnionTypePayload {
    pub(crate) types: SubRange<TsType>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsIntersectionTypePayload {
    pub(crate) types: SubRange<TsType>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsConditionalTypePayload {
    pub(crate) check_type: TsType,
    pub(crate) extends_type: TsType,
    pub(crate) true_type: TsType,
    pub(crate) false_type: TsType,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsInferTypePayload {
    pub(crate) type_parameter: TsTypeParameter,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeOperatorPayload {
    pub(crate) type_annotation: TsType,
    pub(crate) operator: TsTypeOperatorKind,
    pub(crate) _pad0: [u8; 11usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsParenthesizedTypePayload {
    pub(crate) type_annotation: TsType,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsFunctionTypePayload {
    pub(crate) type_parameters: u32,
    pub(crate) this_param: u32,
    pub(crate) params: FormalParameters,
    pub(crate) return_type: TsTypeAnnotation,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsConstructorTypePayload {
    pub(crate) type_parameters: u32,
    pub(crate) params: FormalParameters,
    pub(crate) return_type: TsTypeAnnotation,
    pub(crate) r#abstract: bool,
    pub(crate) _pad0: [u8; 3usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypePredicatePayload {
    pub(crate) parameter_name: TsTypePredicateName,
    pub(crate) type_annotation: u32,
    pub(crate) asserts: bool,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeLiteralPayload {
    pub(crate) members: SubRange<TsTypeElement>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsMappedTypePayload {
    pub(crate) key: BindingIdentifier,
    pub(crate) constraint: TsType,
    pub(crate) optional: TsMappedTypeModifier,
    pub(crate) readonly: TsMappedTypeModifier,
    pub(crate) _pad0: [u8; 2usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsPropertySignaturePayload {
    pub(crate) key: PropertyKey,
    pub(crate) type_annotation: u32,
    pub(crate) computed: bool,
    pub(crate) optional: bool,
    pub(crate) readonly: bool,
    pub(crate) _pad0: [u8; 5usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsMethodSignaturePayload {
    pub(crate) key: PropertyKey,
    pub(crate) type_parameters: u32,
    pub(crate) kind: TsMethodSignatureKind,
    pub(crate) computed: bool,
    pub(crate) optional: bool,
    pub(crate) _pad0: [u8; 1usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsCallSignatureDeclarationPayload {
    pub(crate) type_parameters: u32,
    pub(crate) this_param: u32,
    pub(crate) params: FormalParameters,
    pub(crate) return_type: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsConstructSignatureDeclarationPayload {
    pub(crate) type_parameters: u32,
    pub(crate) params: FormalParameters,
    pub(crate) return_type: u32,
    pub(crate) _pad0: [u8; 4usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsIndexSignaturePayload {
    pub(crate) parameters: SubRange<TsIndexSignatureParameter>,
    pub(crate) type_annotation: TsTypeAnnotation,
    pub(crate) readonly: bool,
    pub(crate) r#static: bool,
    pub(crate) _pad0: [u8; 2usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeAliasDeclarationPayload {
    pub(crate) id: BindingIdentifier,
    pub(crate) type_parameters: u32,
    pub(crate) type_annotation: TsType,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 3usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsInterfaceDeclarationPayload {
    pub(crate) id: BindingIdentifier,
    pub(crate) type_parameters: u32,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 3usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsInterfaceBodyPayload {
    pub(crate) body: SubRange<TsTypeElement>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsInterfaceHeritagePayload {
    pub(crate) expression: Expr,
    pub(crate) type_arguments: u32,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsClassImplementsPayload {
    pub(crate) expression: Expr,
    pub(crate) type_arguments: u32,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsEnumDeclarationPayload {
    pub(crate) id: BindingIdentifier,
    pub(crate) body: TsEnumBody,
    pub(crate) is_const: bool,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 6usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsEnumBodyPayload {
    pub(crate) members: SubRange<TsEnumMember>,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsEnumMemberPayload {
    pub(crate) id: TsEnumMemberName,
    pub(crate) initializer: u32,
    pub(crate) computed: bool,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsModuleDeclarationPayload {
    pub(crate) id: TsModuleName,
    pub(crate) body: u32,
    pub(crate) kind: TsModuleDeclarationKind,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 6usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsModuleBlockPayload {
    pub(crate) directives: SubRange<Directive>,
    pub(crate) body: SubRange<Stmt>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsGlobalDeclarationPayload {
    pub(crate) id: IdentifierName,
    pub(crate) body: TsModuleBlock,
    pub(crate) declare: bool,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsParameterPropertyPayload {
    pub(crate) decorators: SubRange<Decorator>,
    pub(crate) parameter: FormalParameter,
    pub(crate) r#override: bool,
    pub(crate) readonly: bool,
    pub(crate) accessibility: Accessibility,
    pub(crate) _pad0: [u8; 1usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsThisParameterPayload {
    pub(crate) type_annotation: u32,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsAsExpressionPayload {
    pub(crate) expression: Expr,
    pub(crate) type_annotation: TsType,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsSatisfiesExpressionPayload {
    pub(crate) expression: Expr,
    pub(crate) type_annotation: TsType,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsTypeAssertionPayload {
    pub(crate) type_annotation: TsType,
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsNonNullExpressionPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsInstantiationExpressionPayload {
    pub(crate) expression: Expr,
    pub(crate) type_arguments: TsTypeParameterInstantiation,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsExportAssignmentPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsNamespaceExportDeclarationPayload {
    pub(crate) id: IdentifierName,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsImportEqualsDeclarationPayload {
    pub(crate) id: BindingIdentifier,
    pub(crate) module_reference: TsModuleReference,
    pub(crate) import_kind: ImportOrExportKind,
    pub(crate) _pad0: [u8; 7usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsExternalModuleReferencePayload {
    pub(crate) expression: StringLiteral,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct TsIndexSignatureParameterPayload {
    pub(crate) name: BindingIdentifier,
    pub(crate) type_annotation: TsTypeAnnotation,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxElementPayload {
    pub(crate) children: SubRange<JsxChild>,
    pub(crate) opening_element: JsxOpeningElement,
    pub(crate) closing_element: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxOpeningElementPayload {
    pub(crate) name: JsxElementName,
    pub(crate) type_arguments: u32,
    pub(crate) self_closing: bool,
    pub(crate) _pad0: [u8; 3usize],
    pub(crate) extra_data_start: ExtraDataId,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxClosingElementPayload {
    pub(crate) name: JsxElementName,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxFragmentPayload {
    pub(crate) children: SubRange<JsxChild>,
    pub(crate) opening_fragment: JsxOpeningFragment,
    pub(crate) closing_fragment: JsxClosingFragment,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxIdentifierPayload {
    pub(crate) name: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxNamespacedNamePayload {
    pub(crate) namespace: JsxIdentifier,
    pub(crate) name: JsxIdentifier,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxMemberExpressionPayload {
    pub(crate) object: JsxMemberExpressionObject,
    pub(crate) property: JsxIdentifier,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxAttributePayload {
    pub(crate) name: JsxAttributeName,
    pub(crate) value: u32,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxSpreadAttributePayload {
    pub(crate) argument: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxExpressionContainerPayload {
    pub(crate) expression: JsxExpression,
    pub(crate) _pad0: [u8; 12usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxTextPayload {
    pub(crate) value: AstUtf8,
    pub(crate) _pad0: [u8; 8usize],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JsxSpreadChildPayload {
    pub(crate) expression: Expr,
    pub(crate) _pad0: [u8; 12usize],
}
const _: () = {
    assert!(std::mem::size_of::<ProgramPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Directive>>() == 8usize);
    assert!(std::mem::offset_of!(ProgramPayload, directives) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ProgramPayload, hashbang) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ProgramPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<StringLiteralPayload>() == 16);
    assert!(std::mem::size_of::<AstWtf8>() == 8usize);
    assert!(std::mem::offset_of!(StringLiteralPayload, value) == 0usize);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(StringLiteralPayload, raw) == 8usize);
    assert!(std::mem::size_of::<NumericLiteralPayload>() == 16);
    assert!(std::mem::size_of::<f64>() == 8usize);
    assert!(std::mem::offset_of!(NumericLiteralPayload, value) == 0usize);
    assert!(std::mem::size_of::<u64>() == 8usize);
    assert!(std::mem::offset_of!(NumericLiteralPayload, raw) == 8usize);
    assert!(std::mem::size_of::<BigIntLiteralPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(BigIntLiteralPayload, raw) == 0usize);
    assert!(std::mem::size_of::<BooleanLiteralPayload>() == 16);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(BooleanLiteralPayload, value) == 0usize);
    assert!(std::mem::size_of::<RegExpLiteralPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(RegExpLiteralPayload, pattern) == 0usize);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(RegExpLiteralPayload, flags) == 8usize);
    assert!(std::mem::size_of::<TemplateLiteralPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TemplateElement>>() == 8usize);
    assert!(std::mem::offset_of!(TemplateLiteralPayload, quasis) == 0usize);
    assert!(std::mem::size_of::<SubRange<Expr>>() == 8usize);
    assert!(std::mem::offset_of!(TemplateLiteralPayload, expressions) == 8usize);
    assert!(std::mem::size_of::<TemplateElementPayload>() == 16);
    assert!(std::mem::size_of::<AstWtf8>() == 8usize);
    assert!(std::mem::offset_of!(TemplateElementPayload, cooked) == 0usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TemplateElementPayload, tail) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TemplateElementPayload, is_cooked_undefined) == 9usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(TemplateElementPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<IdentifierReferencePayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(IdentifierReferencePayload, name) == 0usize);
    assert!(std::mem::size_of::<PrivateIdentifierPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(PrivateIdentifierPayload, name) == 0usize);
    assert!(std::mem::size_of::<BindingIdentifierPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(BindingIdentifierPayload, name) == 0usize);
    assert!(std::mem::size_of::<IdentifierNamePayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(IdentifierNamePayload, name) == 0usize);
    assert!(std::mem::size_of::<LabelIdentifierPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(LabelIdentifierPayload, name) == 0usize);
    assert!(std::mem::size_of::<AssignmentPatternPayload>() == 16);
    assert!(std::mem::size_of::<BindingPattern>() == 4usize);
    assert!(std::mem::offset_of!(AssignmentPatternPayload, left) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(AssignmentPatternPayload, right) == 4usize);
    assert!(std::mem::size_of::<BindingRestElementPayload>() == 16);
    assert!(std::mem::size_of::<BindingPattern>() == 4usize);
    assert!(std::mem::offset_of!(BindingRestElementPayload, argument) == 0usize);
    assert!(std::mem::size_of::<ArrayPatternPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Option<BindingPattern>>>() == 8usize);
    assert!(std::mem::offset_of!(ArrayPatternPayload, elements) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ArrayPatternPayload, rest) == 8usize);
    assert!(std::mem::size_of::<ObjectPatternPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<BindingProperty>>() == 8usize);
    assert!(std::mem::offset_of!(ObjectPatternPayload, properties) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ObjectPatternPayload, rest) == 8usize);
    assert!(std::mem::size_of::<BindingPropertyPayload>() == 16);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(BindingPropertyPayload, key) == 0usize);
    assert!(std::mem::size_of::<BindingPattern>() == 4usize);
    assert!(std::mem::offset_of!(BindingPropertyPayload, value) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(BindingPropertyPayload, shorthand) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(BindingPropertyPayload, computed) == 9usize);
    assert!(std::mem::size_of::<BinaryExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(BinaryExpressionPayload, left) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(BinaryExpressionPayload, right) == 4usize);
    assert!(std::mem::size_of::<BinaryOperator>() == 1usize);
    assert!(std::mem::offset_of!(BinaryExpressionPayload, operator) == 8usize);
    assert!(std::mem::size_of::<LogicalExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(LogicalExpressionPayload, left) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(LogicalExpressionPayload, right) == 4usize);
    assert!(std::mem::size_of::<LogicalOperator>() == 1usize);
    assert!(std::mem::offset_of!(LogicalExpressionPayload, operator) == 8usize);
    assert!(std::mem::size_of::<ConditionalExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ConditionalExpressionPayload, test) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ConditionalExpressionPayload, consequent) == 4usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ConditionalExpressionPayload, alternate) == 8usize);
    assert!(std::mem::size_of::<UnaryExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(UnaryExpressionPayload, argument) == 0usize);
    assert!(std::mem::size_of::<UnaryOperator>() == 1usize);
    assert!(std::mem::offset_of!(UnaryExpressionPayload, operator) == 4usize);
    assert!(std::mem::size_of::<UpdateExpressionPayload>() == 16);
    assert!(std::mem::size_of::<SimpleAssignmentTarget>() == 4usize);
    assert!(std::mem::offset_of!(UpdateExpressionPayload, argument) == 0usize);
    assert!(std::mem::size_of::<UpdateOperator>() == 1usize);
    assert!(std::mem::offset_of!(UpdateExpressionPayload, operator) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(UpdateExpressionPayload, prefix) == 5usize);
    assert!(std::mem::size_of::<AssignmentExpressionPayload>() == 16);
    assert!(std::mem::size_of::<AssignmentTarget>() == 4usize);
    assert!(std::mem::offset_of!(AssignmentExpressionPayload, left) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(AssignmentExpressionPayload, right) == 4usize);
    assert!(std::mem::size_of::<AssignmentOperator>() == 1usize);
    assert!(std::mem::offset_of!(AssignmentExpressionPayload, operator) == 8usize);
    assert!(std::mem::size_of::<VariableDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<VariableDeclarator>>() == 8usize);
    assert!(std::mem::offset_of!(VariableDeclarationPayload, declarators) == 0usize);
    assert!(std::mem::size_of::<VariableKind>() == 1usize);
    assert!(std::mem::offset_of!(VariableDeclarationPayload, kind) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(VariableDeclarationPayload, declare) == 9usize);
    assert!(std::mem::size_of::<VariableDeclaratorPayload>() == 16);
    assert!(std::mem::size_of::<BindingPattern>() == 4usize);
    assert!(std::mem::offset_of!(VariableDeclaratorPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(VariableDeclaratorPayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(VariableDeclaratorPayload, init) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(VariableDeclaratorPayload, definite) == 12usize);
    assert!(std::mem::size_of::<IfStatementPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(IfStatementPayload, test) == 0usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(IfStatementPayload, consequent) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(IfStatementPayload, alternate) == 8usize);
    assert!(std::mem::size_of::<SwitchStatementPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<SwitchCase>>() == 8usize);
    assert!(std::mem::offset_of!(SwitchStatementPayload, cases) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(SwitchStatementPayload, discriminant) == 8usize);
    assert!(std::mem::size_of::<SwitchCasePayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Stmt>>() == 8usize);
    assert!(std::mem::offset_of!(SwitchCasePayload, consequent) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(SwitchCasePayload, test) == 8usize);
    assert!(std::mem::size_of::<ForStatementPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ForStatementPayload, init) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ForStatementPayload, test) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ForStatementPayload, update) == 8usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(ForStatementPayload, body) == 12usize);
    assert!(std::mem::size_of::<ForInStatementPayload>() == 16);
    assert!(std::mem::size_of::<ForStatementLeft>() == 4usize);
    assert!(std::mem::offset_of!(ForInStatementPayload, left) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ForInStatementPayload, right) == 4usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(ForInStatementPayload, body) == 8usize);
    assert!(std::mem::size_of::<ForOfStatementPayload>() == 16);
    assert!(std::mem::size_of::<ForStatementLeft>() == 4usize);
    assert!(std::mem::offset_of!(ForOfStatementPayload, left) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ForOfStatementPayload, right) == 4usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(ForOfStatementPayload, body) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ForOfStatementPayload, r#await) == 12usize);
    assert!(std::mem::size_of::<TryStatementPayload>() == 16);
    assert!(std::mem::size_of::<BlockStatement>() == 4usize);
    assert!(std::mem::offset_of!(TryStatementPayload, block) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TryStatementPayload, handler) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TryStatementPayload, finalizer) == 8usize);
    assert!(std::mem::size_of::<CatchClausePayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(CatchClausePayload, param) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(CatchClausePayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<BlockStatement>() == 4usize);
    assert!(std::mem::offset_of!(CatchClausePayload, body) == 8usize);
    assert!(std::mem::size_of::<BreakStatementPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(BreakStatementPayload, label) == 0usize);
    assert!(std::mem::size_of::<ContinueStatementPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ContinueStatementPayload, label) == 0usize);
    assert!(std::mem::size_of::<LabeledStatementPayload>() == 16);
    assert!(std::mem::size_of::<LabelIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(LabeledStatementPayload, label) == 0usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(LabeledStatementPayload, body) == 4usize);
    assert!(std::mem::size_of::<ExpressionStatementPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ExpressionStatementPayload, expression) == 0usize);
    assert!(std::mem::size_of::<ReturnStatementPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ReturnStatementPayload, argument) == 0usize);
    assert!(std::mem::size_of::<ThrowStatementPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ThrowStatementPayload, argument) == 0usize);
    assert!(std::mem::size_of::<DirectivePayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(DirectivePayload, value) == 0usize);
    assert!(std::mem::size_of::<StringLiteral>() == 4usize);
    assert!(std::mem::offset_of!(DirectivePayload, expression) == 8usize);
    assert!(std::mem::size_of::<DecoratorPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(DecoratorPayload, expression) == 0usize);
    assert!(std::mem::size_of::<ClassPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ClassPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ClassPayload, type_parameters) == 4usize);
    assert!(std::mem::size_of::<ClassType>() == 1usize);
    assert!(std::mem::offset_of!(ClassPayload, class_type) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ClassPayload, r#abstract) == 9usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ClassPayload, declare) == 10usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ClassPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<ClassBodyPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<ClassElement>>() == 8usize);
    assert!(std::mem::offset_of!(ClassBodyPayload, body) == 0usize);
    assert!(std::mem::size_of::<MethodDefinitionPayload>() == 16);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, key) == 0usize);
    assert!(std::mem::size_of::<MethodDefinitionKind>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, kind) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, computed) == 5usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, r#static) == 6usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, r#override) == 7usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, optional) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, r#abstract) == 9usize);
    assert!(std::mem::size_of::<Accessibility>() == 1usize);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, accessibility) == 10usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(MethodDefinitionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<PropertyDefinitionPayload>() == 16);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, computed) == 0usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, r#static) == 1usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, accessor) == 2usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, declare) == 3usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, r#override) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, optional) == 5usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, definite) == 6usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, readonly) == 7usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, r#abstract) == 8usize);
    assert!(std::mem::size_of::<Accessibility>() == 1usize);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, accessibility) == 9usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(PropertyDefinitionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<StaticBlockPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Stmt>>() == 8usize);
    assert!(std::mem::offset_of!(StaticBlockPayload, body) == 0usize);
    assert!(std::mem::size_of::<FunctionPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(FunctionPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(FunctionPayload, type_parameters) == 4usize);
    assert!(std::mem::size_of::<FunctionType>() == 1usize);
    assert!(std::mem::offset_of!(FunctionPayload, function_type) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(FunctionPayload, generator) == 9usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(FunctionPayload, r#async) == 10usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(FunctionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<FunctionBodyPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Directive>>() == 8usize);
    assert!(std::mem::offset_of!(FunctionBodyPayload, directives) == 0usize);
    assert!(std::mem::size_of::<SubRange<Stmt>>() == 8usize);
    assert!(std::mem::offset_of!(FunctionBodyPayload, body) == 8usize);
    assert!(std::mem::size_of::<BlockStatementPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Stmt>>() == 8usize);
    assert!(std::mem::offset_of!(BlockStatementPayload, body) == 0usize);
    assert!(std::mem::size_of::<WhileStatementPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(WhileStatementPayload, test) == 0usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(WhileStatementPayload, body) == 4usize);
    assert!(std::mem::size_of::<DoWhileStatementPayload>() == 16);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(DoWhileStatementPayload, body) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(DoWhileStatementPayload, test) == 4usize);
    assert!(std::mem::size_of::<WithStatementPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(WithStatementPayload, object) == 0usize);
    assert!(std::mem::size_of::<Stmt>() == 4usize);
    assert!(std::mem::offset_of!(WithStatementPayload, body) == 4usize);
    assert!(std::mem::size_of::<FormalParametersPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<FormalParameterItem>>() == 8usize);
    assert!(std::mem::offset_of!(FormalParametersPayload, items) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(FormalParametersPayload, rest) == 8usize);
    assert!(std::mem::size_of::<FormalParameterKind>() == 1usize);
    assert!(std::mem::offset_of!(FormalParametersPayload, kind) == 12usize);
    assert!(std::mem::size_of::<FormalParameterPayload>() == 16);
    assert!(std::mem::size_of::<FormalParameterPattern>() == 4usize);
    assert!(std::mem::offset_of!(FormalParameterPayload, pattern) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(FormalParameterPayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(FormalParameterPayload, optional) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(FormalParameterPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<FormalParameterRestPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Decorator>>() == 8usize);
    assert!(std::mem::offset_of!(FormalParameterRestPayload, decorators) == 0usize);
    assert!(std::mem::size_of::<FormalParameterPattern>() == 4usize);
    assert!(std::mem::offset_of!(FormalParameterRestPayload, argument) == 8usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(FormalParameterRestPayload, type_annotation) == 12usize);
    assert!(std::mem::size_of::<ArrayExpressionPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Option<Argument>>>() == 8usize);
    assert!(std::mem::offset_of!(ArrayExpressionPayload, elements) == 0usize);
    assert!(std::mem::size_of::<ObjectExpressionPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<ObjectPropertyKind>>() == 8usize);
    assert!(std::mem::offset_of!(ObjectExpressionPayload, properties) == 0usize);
    assert!(std::mem::size_of::<SpreadElementPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(SpreadElementPayload, argument) == 0usize);
    assert!(std::mem::size_of::<ObjectPropertyPayload>() == 16);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(ObjectPropertyPayload, key) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ObjectPropertyPayload, value) == 4usize);
    assert!(std::mem::size_of::<PropertyKind>() == 1usize);
    assert!(std::mem::offset_of!(ObjectPropertyPayload, kind) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ObjectPropertyPayload, method) == 9usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ObjectPropertyPayload, shorthand) == 10usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ObjectPropertyPayload, computed) == 11usize);
    assert!(std::mem::size_of::<ParenthesizedExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ParenthesizedExpressionPayload, expression) == 0usize);
    assert!(std::mem::size_of::<ArrowFunctionExpressionPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ArrowFunctionExpressionPayload, type_parameters) == 0usize);
    assert!(std::mem::size_of::<FormalParameters>() == 4usize);
    assert!(std::mem::offset_of!(ArrowFunctionExpressionPayload, params) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ArrowFunctionExpressionPayload, expression) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ArrowFunctionExpressionPayload, r#async) == 9usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ArrowFunctionExpressionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<SequenceExpressionPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Expr>>() == 8usize);
    assert!(std::mem::offset_of!(SequenceExpressionPayload, expressions) == 0usize);
    assert!(std::mem::size_of::<MemberExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(MemberExpressionPayload, object) == 0usize);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(MemberExpressionPayload, property) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MemberExpressionPayload, computed) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(MemberExpressionPayload, optional) == 9usize);
    assert!(std::mem::size_of::<CallExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(CallExpressionPayload, callee) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(CallExpressionPayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(CallExpressionPayload, optional) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(CallExpressionPayload, pure) == 9usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(CallExpressionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<ChainExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ChainExpressionPayload, expression) == 0usize);
    assert!(std::mem::size_of::<TaggedTemplateExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TaggedTemplateExpressionPayload, tag) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TaggedTemplateExpressionPayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<TemplateLiteral>() == 4usize);
    assert!(std::mem::offset_of!(TaggedTemplateExpressionPayload, quasi) == 8usize);
    assert!(std::mem::size_of::<NewExpressionPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Argument>>() == 8usize);
    assert!(std::mem::offset_of!(NewExpressionPayload, arguments) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(NewExpressionPayload, callee) == 8usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(NewExpressionPayload, type_arguments) == 12usize);
    assert!(std::mem::size_of::<AwaitExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(AwaitExpressionPayload, argument) == 0usize);
    assert!(std::mem::size_of::<YieldExpressionPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(YieldExpressionPayload, argument) == 0usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(YieldExpressionPayload, delegate) == 4usize);
    assert!(std::mem::size_of::<MetaPropertyPayload>() == 16);
    assert!(std::mem::size_of::<IdentifierName>() == 4usize);
    assert!(std::mem::offset_of!(MetaPropertyPayload, meta) == 0usize);
    assert!(std::mem::size_of::<IdentifierName>() == 4usize);
    assert!(std::mem::offset_of!(MetaPropertyPayload, property) == 4usize);
    assert!(std::mem::size_of::<ImportExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(ImportExpressionPayload, source) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ImportExpressionPayload, options) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(ImportExpressionPayload, sync) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ImportExpressionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<ImportDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<StringLiteral>() == 4usize);
    assert!(std::mem::offset_of!(ImportDeclarationPayload, source) == 0usize);
    assert!(std::mem::size_of::<ImportOrExportKind>() == 1usize);
    assert!(std::mem::offset_of!(ImportDeclarationPayload, import_kind) == 4usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ImportDeclarationPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<ImportSpecifierPayload>() == 16);
    assert!(std::mem::size_of::<ModuleExportName>() == 4usize);
    assert!(std::mem::offset_of!(ImportSpecifierPayload, imported) == 0usize);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(ImportSpecifierPayload, local) == 4usize);
    assert!(std::mem::size_of::<ImportOrExportKind>() == 1usize);
    assert!(std::mem::offset_of!(ImportSpecifierPayload, import_kind) == 8usize);
    assert!(std::mem::size_of::<ImportDefaultSpecifierPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(ImportDefaultSpecifierPayload, local) == 0usize);
    assert!(std::mem::size_of::<ImportNamespaceSpecifierPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(ImportNamespaceSpecifierPayload, local) == 0usize);
    assert!(std::mem::size_of::<ImportAttributePayload>() == 16);
    assert!(std::mem::size_of::<ModuleExportName>() == 4usize);
    assert!(std::mem::offset_of!(ImportAttributePayload, key) == 0usize);
    assert!(std::mem::size_of::<StringLiteral>() == 4usize);
    assert!(std::mem::offset_of!(ImportAttributePayload, value) == 4usize);
    assert!(std::mem::size_of::<ExportNamedDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ExportNamedDeclarationPayload, declaration) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ExportNamedDeclarationPayload, source) == 4usize);
    assert!(std::mem::size_of::<ImportOrExportKind>() == 1usize);
    assert!(std::mem::offset_of!(ExportNamedDeclarationPayload, export_kind) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ExportNamedDeclarationPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<ExportDefaultDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<ExportDefaultDeclarationKind>() == 4usize);
    assert!(std::mem::offset_of!(ExportDefaultDeclarationPayload, declaration) == 0usize);
    assert!(std::mem::size_of::<ExportAllDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(ExportAllDeclarationPayload, exported) == 0usize);
    assert!(std::mem::size_of::<StringLiteral>() == 4usize);
    assert!(std::mem::offset_of!(ExportAllDeclarationPayload, source) == 4usize);
    assert!(std::mem::size_of::<ImportOrExportKind>() == 1usize);
    assert!(std::mem::offset_of!(ExportAllDeclarationPayload, export_kind) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(ExportAllDeclarationPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<ExportSpecifierPayload>() == 16);
    assert!(std::mem::size_of::<ModuleExportName>() == 4usize);
    assert!(std::mem::offset_of!(ExportSpecifierPayload, local) == 0usize);
    assert!(std::mem::size_of::<ModuleExportName>() == 4usize);
    assert!(std::mem::offset_of!(ExportSpecifierPayload, exported) == 4usize);
    assert!(std::mem::size_of::<ImportOrExportKind>() == 1usize);
    assert!(std::mem::offset_of!(ExportSpecifierPayload, export_kind) == 8usize);
    assert!(std::mem::size_of::<HashbangPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(HashbangPayload, value) == 0usize);
    assert!(std::mem::size_of::<TsFunctionPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsFunctionPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsFunctionPayload, type_parameters) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsFunctionPayload, generator) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsFunctionPayload, r#async) == 9usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsFunctionPayload, declare) == 10usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(TsFunctionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<TsMethodDefinitionPayload>() == 16);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, key) == 0usize);
    assert!(std::mem::size_of::<MethodDefinitionKind>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, kind) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, computed) == 5usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, r#static) == 6usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, r#override) == 7usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, optional) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, r#abstract) == 9usize);
    assert!(std::mem::size_of::<Accessibility>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, accessibility) == 10usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(TsMethodDefinitionPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<TsTypeAnnotationPayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeAnnotationPayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<TsTypeReferencePayload>() == 16);
    assert!(std::mem::size_of::<TsTypeName>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeReferencePayload, type_name) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeReferencePayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<TsQualifiedNamePayload>() == 16);
    assert!(std::mem::size_of::<TsQualifiedNameLeft>() == 4usize);
    assert!(std::mem::offset_of!(TsQualifiedNamePayload, left) == 0usize);
    assert!(std::mem::size_of::<IdentifierName>() == 4usize);
    assert!(std::mem::offset_of!(TsQualifiedNamePayload, right) == 4usize);
    assert!(std::mem::size_of::<TsTypeQueryPayload>() == 16);
    assert!(std::mem::size_of::<TsTypeQueryExprName>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeQueryPayload, expr_name) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeQueryPayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<TsImportTypePayload>() == 16);
    assert!(std::mem::size_of::<StringLiteral>() == 4usize);
    assert!(std::mem::offset_of!(TsImportTypePayload, source) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsImportTypePayload, options) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsImportTypePayload, qualifier) == 8usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsImportTypePayload, type_arguments) == 12usize);
    assert!(std::mem::size_of::<TsTypeParameterPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeParameterPayload, name) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeParameterPayload, constraint) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeParameterPayload, default) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsTypeParameterPayload, r#in) == 12usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsTypeParameterPayload, out) == 13usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsTypeParameterPayload, r#const) == 14usize);
    assert!(std::mem::size_of::<TsTypeParameterDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsTypeParameter>>() == 8usize);
    assert!(std::mem::offset_of!(TsTypeParameterDeclarationPayload, params) == 0usize);
    assert!(std::mem::size_of::<TsTypeParameterInstantiationPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsType>>() == 8usize);
    assert!(std::mem::offset_of!(TsTypeParameterInstantiationPayload, params) == 0usize);
    assert!(std::mem::size_of::<TsLiteralTypePayload>() == 16);
    assert!(std::mem::size_of::<TsLiteral>() == 4usize);
    assert!(std::mem::offset_of!(TsLiteralTypePayload, literal) == 0usize);
    assert!(std::mem::size_of::<TsTemplateLiteralTypePayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TemplateElement>>() == 8usize);
    assert!(std::mem::offset_of!(TsTemplateLiteralTypePayload, quasis) == 0usize);
    assert!(std::mem::size_of::<SubRange<TsType>>() == 8usize);
    assert!(std::mem::offset_of!(TsTemplateLiteralTypePayload, types) == 8usize);
    assert!(std::mem::size_of::<TsArrayTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsArrayTypePayload, element_type) == 0usize);
    assert!(std::mem::size_of::<TsIndexedAccessTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsIndexedAccessTypePayload, object_type) == 0usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsIndexedAccessTypePayload, index_type) == 4usize);
    assert!(std::mem::size_of::<TsTupleTypePayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsTupleElement>>() == 8usize);
    assert!(std::mem::offset_of!(TsTupleTypePayload, element_types) == 0usize);
    assert!(std::mem::size_of::<TsNamedTupleMemberPayload>() == 16);
    assert!(std::mem::size_of::<IdentifierName>() == 4usize);
    assert!(std::mem::offset_of!(TsNamedTupleMemberPayload, label) == 0usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsNamedTupleMemberPayload, element_type) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsNamedTupleMemberPayload, optional) == 8usize);
    assert!(std::mem::size_of::<TsOptionalTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsOptionalTypePayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<TsRestTypePayload>() == 16);
    assert!(std::mem::size_of::<TsTupleElement>() == 4usize);
    assert!(std::mem::offset_of!(TsRestTypePayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<TsJsDocNullableTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsJsDocNullableTypePayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsJsDocNullableTypePayload, postfix) == 4usize);
    assert!(std::mem::size_of::<TsJsDocNonNullableTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsJsDocNonNullableTypePayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsJsDocNonNullableTypePayload, postfix) == 4usize);
    assert!(std::mem::size_of::<TsUnionTypePayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsType>>() == 8usize);
    assert!(std::mem::offset_of!(TsUnionTypePayload, types) == 0usize);
    assert!(std::mem::size_of::<TsIntersectionTypePayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsType>>() == 8usize);
    assert!(std::mem::offset_of!(TsIntersectionTypePayload, types) == 0usize);
    assert!(std::mem::size_of::<TsConditionalTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsConditionalTypePayload, check_type) == 0usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsConditionalTypePayload, extends_type) == 4usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsConditionalTypePayload, true_type) == 8usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsConditionalTypePayload, false_type) == 12usize);
    assert!(std::mem::size_of::<TsInferTypePayload>() == 16);
    assert!(std::mem::size_of::<TsTypeParameter>() == 4usize);
    assert!(std::mem::offset_of!(TsInferTypePayload, type_parameter) == 0usize);
    assert!(std::mem::size_of::<TsTypeOperatorPayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeOperatorPayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<TsTypeOperatorKind>() == 1usize);
    assert!(std::mem::offset_of!(TsTypeOperatorPayload, operator) == 4usize);
    assert!(std::mem::size_of::<TsParenthesizedTypePayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsParenthesizedTypePayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<TsFunctionTypePayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsFunctionTypePayload, type_parameters) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsFunctionTypePayload, this_param) == 4usize);
    assert!(std::mem::size_of::<FormalParameters>() == 4usize);
    assert!(std::mem::offset_of!(TsFunctionTypePayload, params) == 8usize);
    assert!(std::mem::size_of::<TsTypeAnnotation>() == 4usize);
    assert!(std::mem::offset_of!(TsFunctionTypePayload, return_type) == 12usize);
    assert!(std::mem::size_of::<TsConstructorTypePayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsConstructorTypePayload, type_parameters) == 0usize);
    assert!(std::mem::size_of::<FormalParameters>() == 4usize);
    assert!(std::mem::offset_of!(TsConstructorTypePayload, params) == 4usize);
    assert!(std::mem::size_of::<TsTypeAnnotation>() == 4usize);
    assert!(std::mem::offset_of!(TsConstructorTypePayload, return_type) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsConstructorTypePayload, r#abstract) == 12usize);
    assert!(std::mem::size_of::<TsTypePredicatePayload>() == 16);
    assert!(std::mem::size_of::<TsTypePredicateName>() == 4usize);
    assert!(std::mem::offset_of!(TsTypePredicatePayload, parameter_name) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsTypePredicatePayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsTypePredicatePayload, asserts) == 8usize);
    assert!(std::mem::size_of::<TsTypeLiteralPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsTypeElement>>() == 8usize);
    assert!(std::mem::offset_of!(TsTypeLiteralPayload, members) == 0usize);
    assert!(std::mem::size_of::<TsMappedTypePayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsMappedTypePayload, key) == 0usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsMappedTypePayload, constraint) == 4usize);
    assert!(std::mem::size_of::<TsMappedTypeModifier>() == 1usize);
    assert!(std::mem::offset_of!(TsMappedTypePayload, optional) == 8usize);
    assert!(std::mem::size_of::<TsMappedTypeModifier>() == 1usize);
    assert!(std::mem::offset_of!(TsMappedTypePayload, readonly) == 9usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(TsMappedTypePayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<TsPropertySignaturePayload>() == 16);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(TsPropertySignaturePayload, key) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsPropertySignaturePayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsPropertySignaturePayload, computed) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsPropertySignaturePayload, optional) == 9usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsPropertySignaturePayload, readonly) == 10usize);
    assert!(std::mem::size_of::<TsMethodSignaturePayload>() == 16);
    assert!(std::mem::size_of::<PropertyKey>() == 4usize);
    assert!(std::mem::offset_of!(TsMethodSignaturePayload, key) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsMethodSignaturePayload, type_parameters) == 4usize);
    assert!(std::mem::size_of::<TsMethodSignatureKind>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodSignaturePayload, kind) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodSignaturePayload, computed) == 9usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsMethodSignaturePayload, optional) == 10usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(TsMethodSignaturePayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<TsCallSignatureDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsCallSignatureDeclarationPayload, type_parameters) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsCallSignatureDeclarationPayload, this_param) == 4usize);
    assert!(std::mem::size_of::<FormalParameters>() == 4usize);
    assert!(std::mem::offset_of!(TsCallSignatureDeclarationPayload, params) == 8usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsCallSignatureDeclarationPayload, return_type) == 12usize);
    assert!(std::mem::size_of::<TsConstructSignatureDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(
        std::mem::offset_of!(TsConstructSignatureDeclarationPayload, type_parameters) == 0usize
    );
    assert!(std::mem::size_of::<FormalParameters>() == 4usize);
    assert!(std::mem::offset_of!(TsConstructSignatureDeclarationPayload, params) == 4usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsConstructSignatureDeclarationPayload, return_type) == 8usize);
    assert!(std::mem::size_of::<TsIndexSignaturePayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsIndexSignatureParameter>>() == 8usize);
    assert!(std::mem::offset_of!(TsIndexSignaturePayload, parameters) == 0usize);
    assert!(std::mem::size_of::<TsTypeAnnotation>() == 4usize);
    assert!(std::mem::offset_of!(TsIndexSignaturePayload, type_annotation) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsIndexSignaturePayload, readonly) == 12usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsIndexSignaturePayload, r#static) == 13usize);
    assert!(std::mem::size_of::<TsTypeAliasDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeAliasDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeAliasDeclarationPayload, type_parameters) == 4usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeAliasDeclarationPayload, type_annotation) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsTypeAliasDeclarationPayload, declare) == 12usize);
    assert!(std::mem::size_of::<TsInterfaceDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsInterfaceDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsInterfaceDeclarationPayload, type_parameters) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsInterfaceDeclarationPayload, declare) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(TsInterfaceDeclarationPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<TsInterfaceBodyPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsTypeElement>>() == 8usize);
    assert!(std::mem::offset_of!(TsInterfaceBodyPayload, body) == 0usize);
    assert!(std::mem::size_of::<TsInterfaceHeritagePayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsInterfaceHeritagePayload, expression) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsInterfaceHeritagePayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<TsClassImplementsPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsClassImplementsPayload, expression) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsClassImplementsPayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<TsEnumDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsEnumDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<TsEnumBody>() == 4usize);
    assert!(std::mem::offset_of!(TsEnumDeclarationPayload, body) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsEnumDeclarationPayload, is_const) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsEnumDeclarationPayload, declare) == 9usize);
    assert!(std::mem::size_of::<TsEnumBodyPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<TsEnumMember>>() == 8usize);
    assert!(std::mem::offset_of!(TsEnumBodyPayload, members) == 0usize);
    assert!(std::mem::size_of::<TsEnumMemberPayload>() == 16);
    assert!(std::mem::size_of::<TsEnumMemberName>() == 4usize);
    assert!(std::mem::offset_of!(TsEnumMemberPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsEnumMemberPayload, initializer) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsEnumMemberPayload, computed) == 8usize);
    assert!(std::mem::size_of::<TsModuleDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<TsModuleName>() == 4usize);
    assert!(std::mem::offset_of!(TsModuleDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsModuleDeclarationPayload, body) == 4usize);
    assert!(std::mem::size_of::<TsModuleDeclarationKind>() == 1usize);
    assert!(std::mem::offset_of!(TsModuleDeclarationPayload, kind) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsModuleDeclarationPayload, declare) == 9usize);
    assert!(std::mem::size_of::<TsModuleBlockPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Directive>>() == 8usize);
    assert!(std::mem::offset_of!(TsModuleBlockPayload, directives) == 0usize);
    assert!(std::mem::size_of::<SubRange<Stmt>>() == 8usize);
    assert!(std::mem::offset_of!(TsModuleBlockPayload, body) == 8usize);
    assert!(std::mem::size_of::<TsGlobalDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<IdentifierName>() == 4usize);
    assert!(std::mem::offset_of!(TsGlobalDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<TsModuleBlock>() == 4usize);
    assert!(std::mem::offset_of!(TsGlobalDeclarationPayload, body) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsGlobalDeclarationPayload, declare) == 8usize);
    assert!(std::mem::size_of::<TsParameterPropertyPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<Decorator>>() == 8usize);
    assert!(std::mem::offset_of!(TsParameterPropertyPayload, decorators) == 0usize);
    assert!(std::mem::size_of::<FormalParameter>() == 4usize);
    assert!(std::mem::offset_of!(TsParameterPropertyPayload, parameter) == 8usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsParameterPropertyPayload, r#override) == 12usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(TsParameterPropertyPayload, readonly) == 13usize);
    assert!(std::mem::size_of::<Accessibility>() == 1usize);
    assert!(std::mem::offset_of!(TsParameterPropertyPayload, accessibility) == 14usize);
    assert!(std::mem::size_of::<TsThisParameterPayload>() == 16);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(TsThisParameterPayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<TsAsExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsAsExpressionPayload, expression) == 0usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsAsExpressionPayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<TsSatisfiesExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsSatisfiesExpressionPayload, expression) == 0usize);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsSatisfiesExpressionPayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<TsTypeAssertionPayload>() == 16);
    assert!(std::mem::size_of::<TsType>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeAssertionPayload, type_annotation) == 0usize);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsTypeAssertionPayload, expression) == 4usize);
    assert!(std::mem::size_of::<TsNonNullExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsNonNullExpressionPayload, expression) == 0usize);
    assert!(std::mem::size_of::<TsInstantiationExpressionPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsInstantiationExpressionPayload, expression) == 0usize);
    assert!(std::mem::size_of::<TsTypeParameterInstantiation>() == 4usize);
    assert!(std::mem::offset_of!(TsInstantiationExpressionPayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<TsExportAssignmentPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(TsExportAssignmentPayload, expression) == 0usize);
    assert!(std::mem::size_of::<TsNamespaceExportDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<IdentifierName>() == 4usize);
    assert!(std::mem::offset_of!(TsNamespaceExportDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<TsImportEqualsDeclarationPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsImportEqualsDeclarationPayload, id) == 0usize);
    assert!(std::mem::size_of::<TsModuleReference>() == 4usize);
    assert!(std::mem::offset_of!(TsImportEqualsDeclarationPayload, module_reference) == 4usize);
    assert!(std::mem::size_of::<ImportOrExportKind>() == 1usize);
    assert!(std::mem::offset_of!(TsImportEqualsDeclarationPayload, import_kind) == 8usize);
    assert!(std::mem::size_of::<TsExternalModuleReferencePayload>() == 16);
    assert!(std::mem::size_of::<StringLiteral>() == 4usize);
    assert!(std::mem::offset_of!(TsExternalModuleReferencePayload, expression) == 0usize);
    assert!(std::mem::size_of::<TsIndexSignatureParameterPayload>() == 16);
    assert!(std::mem::size_of::<BindingIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(TsIndexSignatureParameterPayload, name) == 0usize);
    assert!(std::mem::size_of::<TsTypeAnnotation>() == 4usize);
    assert!(std::mem::offset_of!(TsIndexSignatureParameterPayload, type_annotation) == 4usize);
    assert!(std::mem::size_of::<JsxElementPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<JsxChild>>() == 8usize);
    assert!(std::mem::offset_of!(JsxElementPayload, children) == 0usize);
    assert!(std::mem::size_of::<JsxOpeningElement>() == 4usize);
    assert!(std::mem::offset_of!(JsxElementPayload, opening_element) == 8usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(JsxElementPayload, closing_element) == 12usize);
    assert!(std::mem::size_of::<JsxOpeningElementPayload>() == 16);
    assert!(std::mem::size_of::<JsxElementName>() == 4usize);
    assert!(std::mem::offset_of!(JsxOpeningElementPayload, name) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(JsxOpeningElementPayload, type_arguments) == 4usize);
    assert!(std::mem::size_of::<bool>() == 1usize);
    assert!(std::mem::offset_of!(JsxOpeningElementPayload, self_closing) == 8usize);
    assert!(std::mem::size_of::<ExtraDataId>() == 4);
    assert!(std::mem::offset_of!(JsxOpeningElementPayload, extra_data_start) == 12usize);
    assert!(std::mem::size_of::<JsxClosingElementPayload>() == 16);
    assert!(std::mem::size_of::<JsxElementName>() == 4usize);
    assert!(std::mem::offset_of!(JsxClosingElementPayload, name) == 0usize);
    assert!(std::mem::size_of::<JsxFragmentPayload>() == 16);
    assert!(std::mem::size_of::<SubRange<JsxChild>>() == 8usize);
    assert!(std::mem::offset_of!(JsxFragmentPayload, children) == 0usize);
    assert!(std::mem::size_of::<JsxOpeningFragment>() == 4usize);
    assert!(std::mem::offset_of!(JsxFragmentPayload, opening_fragment) == 8usize);
    assert!(std::mem::size_of::<JsxClosingFragment>() == 4usize);
    assert!(std::mem::offset_of!(JsxFragmentPayload, closing_fragment) == 12usize);
    assert!(std::mem::size_of::<JsxIdentifierPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(JsxIdentifierPayload, name) == 0usize);
    assert!(std::mem::size_of::<JsxNamespacedNamePayload>() == 16);
    assert!(std::mem::size_of::<JsxIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(JsxNamespacedNamePayload, namespace) == 0usize);
    assert!(std::mem::size_of::<JsxIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(JsxNamespacedNamePayload, name) == 4usize);
    assert!(std::mem::size_of::<JsxMemberExpressionPayload>() == 16);
    assert!(std::mem::size_of::<JsxMemberExpressionObject>() == 4usize);
    assert!(std::mem::offset_of!(JsxMemberExpressionPayload, object) == 0usize);
    assert!(std::mem::size_of::<JsxIdentifier>() == 4usize);
    assert!(std::mem::offset_of!(JsxMemberExpressionPayload, property) == 4usize);
    assert!(std::mem::size_of::<JsxAttributePayload>() == 16);
    assert!(std::mem::size_of::<JsxAttributeName>() == 4usize);
    assert!(std::mem::offset_of!(JsxAttributePayload, name) == 0usize);
    assert!(std::mem::size_of::<u32>() == 4usize);
    assert!(std::mem::offset_of!(JsxAttributePayload, value) == 4usize);
    assert!(std::mem::size_of::<JsxSpreadAttributePayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(JsxSpreadAttributePayload, argument) == 0usize);
    assert!(std::mem::size_of::<JsxExpressionContainerPayload>() == 16);
    assert!(std::mem::size_of::<JsxExpression>() == 4usize);
    assert!(std::mem::offset_of!(JsxExpressionContainerPayload, expression) == 0usize);
    assert!(std::mem::size_of::<JsxTextPayload>() == 16);
    assert!(std::mem::size_of::<AstUtf8>() == 8usize);
    assert!(std::mem::offset_of!(JsxTextPayload, value) == 0usize);
    assert!(std::mem::size_of::<JsxSpreadChildPayload>() == 16);
    assert!(std::mem::size_of::<Expr>() == 4usize);
    assert!(std::mem::offset_of!(JsxSpreadChildPayload, expression) == 0usize);
};
/// Convert each numeric field between native and little-endian bytes.
/// Swapping twice restores the original representation.
pub(crate) fn swap_payload_endian(bytes: &mut [u8; 16], kind: NodeKind) {
    match kind {
        NodeKind::Program => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::StringLiteral => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::NumericLiteral => {
            bytes[0usize..8usize].reverse();
            bytes[8usize..16usize].reverse();
        }
        NodeKind::BigIntLiteral => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::RegExpLiteral => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TemplateLiteral => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TemplateElement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::IdentifierReference => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::PrivateIdentifier => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::BindingIdentifier => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::IdentifierName => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::LabelIdentifier => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::AssignmentPattern => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::BindingRestElement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ArrayPattern => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::ObjectPattern => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::BindingProperty => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::BinaryExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::LogicalExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ConditionalExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::UnaryExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::UpdateExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::AssignmentExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::VariableDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::VariableDeclarator => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::IfStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::SwitchStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::SwitchCase => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::ForStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ForInStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::ForOfStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TryStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::CatchClause => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::BreakStatement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ContinueStatement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::LabeledStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ExpressionStatement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ReturnStatement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ThrowStatement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::Directive => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::Decorator => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::Class => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ClassBody => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::MethodDefinition => {
            bytes[0usize..4usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::PropertyDefinition => {
            bytes[12usize..16usize].reverse();
        }
        NodeKind::StaticBlock => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::Function => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::FunctionBody => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::BlockStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::WhileStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::DoWhileStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::WithStatement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::FormalParameters => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::FormalParameter => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::FormalParameterRest => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ArrayExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ObjectExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::SpreadElement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ObjectProperty => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ParenthesizedExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ArrowFunctionExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::SequenceExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::MemberExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::CallExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ChainExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TaggedTemplateExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::NewExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::AwaitExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::YieldExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::MetaProperty => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ImportExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ImportDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ImportSpecifier => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ImportDefaultSpecifier => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ImportNamespaceSpecifier => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ImportAttribute => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::ExportNamedDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ExportDefaultDeclaration => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::ExportAllDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::ExportSpecifier => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::Hashbang => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsFunction => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsMethodDefinition => {
            bytes[0usize..4usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsTypeAnnotation => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsTypeReference => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsQualifiedName => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsTypeQuery => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsImportType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsTypeParameter => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TsTypeParameterDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsTypeParameterInstantiation => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsLiteralType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsTemplateLiteralType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsArrayType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsIndexedAccessType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsTupleType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsNamedTupleMember => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsOptionalType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsRestType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsJsDocNullableType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsJsDocNonNullableType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsUnionType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsIntersectionType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsConditionalType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsInferType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsTypeOperator => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsParenthesizedType => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsFunctionType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsConstructorType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TsTypePredicate => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsTypeLiteral => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsMappedType => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsPropertySignature => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsMethodSignature => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsCallSignatureDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsConstructSignatureDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TsIndexSignature => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TsTypeAliasDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TsInterfaceDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsInterfaceBody => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsInterfaceHeritage => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsClassImplements => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsEnumDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsEnumBody => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsEnumMember => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsModuleDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsModuleBlock => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::TsGlobalDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsParameterProperty => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
        }
        NodeKind::TsThisParameter => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsAsExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsSatisfiesExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsTypeAssertion => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsNonNullExpression => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsInstantiationExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsExportAssignment => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsNamespaceExportDeclaration => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsImportEqualsDeclaration => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::TsExternalModuleReference => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::TsIndexSignatureParameter => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::JsxElement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::JsxOpeningElement => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::JsxClosingElement => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::JsxFragment => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
            bytes[8usize..12usize].reverse();
            bytes[12usize..16usize].reverse();
        }
        NodeKind::JsxIdentifier => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::JsxNamespacedName => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::JsxMemberExpression => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::JsxAttribute => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::JsxSpreadAttribute => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::JsxExpressionContainer => {
            bytes[0usize..4usize].reverse();
        }
        NodeKind::JsxText => {
            bytes[0usize..4usize].reverse();
            bytes[4usize..8usize].reverse();
        }
        NodeKind::JsxSpreadChild => {
            bytes[0usize..4usize].reverse();
        }
        _ => {}
    }
}
