use convert_case::{Case, Casing};

pub(super) fn estree_type(name: &str) -> String {
    match name {
        "BindingRestElement" | "FormalParameterRest" => "RestElement".into(),
        "ObjectProperty" => "Property".into(),
        "IdentifierReference" | "BindingIdentifier" | "IdentifierName" | "LabelIdentifier" => {
            "Identifier".into()
        }
        "TsBigIntKeyword" => "TSBigIntKeyword".into(),
        "TsJsDocNullableType" => "TSJSDocNullableType".into(),
        "TsJsDocNonNullableType" => "TSJSDocNonNullableType".into(),
        "TsJsDocUnknownType" => "TSJSDocUnknownType".into(),
        _ if name.starts_with("Jsx") => format!("JSX{}", &name[3..]),
        _ if name.starts_with("Ts") => format!("TS{}", &name[2..]),
        _ => name.into(),
    }
}

pub(super) fn is_ts_field(node: &str, field: &str) -> bool {
    matches!(
        (node, field),
        ("VariableDeclaration", "declare")
            | ("VariableDeclarator", "definite")
            | ("CallExpression", "typeArguments")
            | ("NewExpression", "typeArguments")
            | ("TaggedTemplateExpression", "typeArguments")
            | ("JsxOpeningElement", "typeArguments")
            | ("ImportDeclaration", "importKind")
            | ("ImportSpecifier", "importKind")
            | ("ExportNamedDeclaration", "exportKind")
            | ("ExportAllDeclaration", "exportKind")
            | ("ExportSpecifier", "exportKind")
            | (
                "BindingRestElement",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "BindingIdentifier",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "AssignmentPattern",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "ObjectPattern",
                "decorators" | "optional" | "typeAnnotation"
            )
            | ("ArrayPattern", "decorators" | "optional" | "typeAnnotation")
            | ("TsIndexSignature", "static")
    )
}

const IDENTIFIER_TS_EXTRAS: &[(&str, &str)] = &[
    ("decorators", "[]"),
    ("optional", "false"),
    ("typeAnnotation", "null"),
];
const OBJECT_PROPERTY_TS_EXTRAS: &[(&str, &str)] = &[("optional", "false")];
const EXPRESSION_STATEMENT_TS_EXTRAS: &[(&str, &str)] = &[("directive", "null")];
const BINDING_REST_TS_EXTRAS: &[(&str, &str)] = &[
    ("value", "null"),
    ("decorators", "[]"),
    ("optional", "false"),
    ("typeAnnotation", "null"),
];
const EXPORT_DEFAULT_TS_EXTRAS: &[(&str, &str)] = &[("exportKind", "\"value\"")];
const TS_PROPERTY_SIGNATURE_EXTRAS: &[(&str, &str)] =
    &[("accessibility", "null"), ("static", "false")];
const TS_INDEX_SIGNATURE_EXTRAS: &[(&str, &str)] = &[("accessibility", "null")];
const TS_PARAMETER_PROPERTY_EXTRAS: &[(&str, &str)] = &[("static", "false")];

pub(super) fn ts_extra_fields(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "IdentifierReference"
        | "IdentifierName"
        | "LabelIdentifier"
        | "BindingIdentifier"
        | "AssignmentPattern"
        | "ArrayPattern"
        | "ObjectPattern" => IDENTIFIER_TS_EXTRAS,
        "ObjectProperty" => OBJECT_PROPERTY_TS_EXTRAS,
        "ExpressionStatement" => EXPRESSION_STATEMENT_TS_EXTRAS,
        "BindingRestElement" => BINDING_REST_TS_EXTRAS,
        "ExportDefaultDeclaration" => EXPORT_DEFAULT_TS_EXTRAS,
        "TsPropertySignature" => TS_PROPERTY_SIGNATURE_EXTRAS,
        "TsIndexSignature" => TS_INDEX_SIGNATURE_EXTRAS,
        "TsParameterProperty" => TS_PARAMETER_PROPERTY_EXTRAS,
        _ => &[],
    }
}

pub(super) fn estree_field_name(node: &str, field: &str) -> String {
    let field = field.strip_prefix("r#").unwrap_or(field);
    if node == "VariableDeclaration" && field == "declarators" {
        return "declarations".into();
    }
    if node == "TsEnumDeclaration" && field == "is_const" {
        return "const".into();
    }
    field.to_case(Case::Camel)
}

/// JS enum values shared by the decoder and declaration generator.
pub(super) const ENUMS: &[(&str, &[&str])] = &[
    (
        "Lang",
        &["\"js\"", "\"ts\"", "\"jsx\"", "\"tsx\"", "\"dts\""],
    ),
    (
        "SourceType",
        &[
            "\"module\"",
            "\"script\"",
            "\"commonjs\"",
            "\"unambiguous\"",
        ],
    ),
    (
        "MethodDefinitionKind",
        &["\"constructor\"", "\"method\"", "\"get\"", "\"set\""],
    ),
    (
        "ClassType",
        &["\"ClassDeclaration\"", "\"ClassExpression\""],
    ),
    (
        "BinaryOperator",
        &[
            "\"==\"",
            "\"!=\"",
            "\"===\"",
            "\"!==\"",
            "\"<\"",
            "\"<=\"",
            "\">\"",
            "\">=\"",
            "\"+\"",
            "\"-\"",
            "\"*\"",
            "\"/\"",
            "\"%\"",
            "\"**\"",
            "\"|\"",
            "\"^\"",
            "\"&\"",
            "\"<<\"",
            "\">>\"",
            "\">>>\"",
            "\"in\"",
            "\"instanceof\"",
        ],
    ),
    ("LogicalOperator", &["\"&&\"", "\"||\"", "\"??\""]),
    (
        "UnaryOperator",
        &[
            "\"-\"",
            "\"+\"",
            "\"!\"",
            "\"~\"",
            "\"typeof\"",
            "\"void\"",
            "\"delete\"",
        ],
    ),
    ("UpdateOperator", &["\"++\"", "\"--\""]),
    (
        "AssignmentOperator",
        &[
            "\"=\"", "\"+=\"", "\"-=\"", "\"*=\"", "\"/=\"", "\"%=\"", "\"**=\"", "\"<<=\"",
            "\">>=\"", "\">>>=\"", "\"|=\"", "\"^=\"", "\"&=\"", "\"||=\"", "\"&&=\"", "\"??=\"",
        ],
    ),
    (
        "VariableKind",
        &[
            "\"var\"",
            "\"let\"",
            "\"const\"",
            "\"using\"",
            "\"await using\"",
        ],
    ),
    (
        "FunctionType",
        &["\"FunctionDeclaration\"", "\"FunctionExpression\""],
    ),
    (
        "FormalParameterKind",
        &["\"formal\"", "\"unique\"", "\"arrow\"", "\"signature\""],
    ),
    ("PropertyKind", &["\"init\"", "\"get\"", "\"set\""]),
    ("ImportPhase", &["\"source\"", "\"defer\""]),
    (
        "Accessibility",
        &["null", "\"public\"", "\"private\"", "\"protected\""],
    ),
    ("ImportOrExportKind", &["\"value\"", "\"type\""]),
    (
        "TsTypeOperatorKind",
        &["\"keyof\"", "\"unique\"", "\"readonly\""],
    ),
    (
        "TsMethodSignatureKind",
        &["\"method\"", "\"get\"", "\"set\""],
    ),
    ("TsModuleDeclarationKind", &["\"namespace\"", "\"module\""]),
    ("TsMappedTypeModifier", &["null", "true", "\"+\"", "\"-\""]),
];
