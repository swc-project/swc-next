// ESTree projection declarations used to generate fixed protocol tables.

pub const READ_RULES: &[&str] = &[
    "Missing",
    "Native",
    "False",
    "True",
    "Null",
    "List",
    "Params",
    "Pattern",
    "Initializer",
    "SourceType",
    "Directive",
    "NumericRaw",
    "BigIntRaw",
    "BigInt",
    "BigIntText",
    "BooleanRaw",
    "NullRaw",
    "RegexRaw",
    "RegexFields",
    "RegexValue",
    "TemplateValue",
    "Text",
    "Init",
    "Optional",
    "ModuleBody",
    "Global",
    "This",
    "Value",
    "ParameterOptional",
    "ParameterAnnotation",
    "DelegateName",
    "DelegateList",
    "EmptyList",
];

pub const PATTERNS: &[&str] = &[
    "BindingIdentifier",
    "ArrayPattern",
    "ObjectPattern",
    "IdentifierName",
];

pub const FUNCTIONS: &[&str] = &[
    "Function",
    "TsFunction",
    "ArrowFunctionExpression",
    "TsFunctionType",
    "TsConstructorType",
    "TsMethodSignature",
    "TsCallSignatureDeclaration",
    "TsConstructSignatureDeclaration",
];

pub const DYNAMIC_TYPES: &[&str] = &[
    "Function",
    "TsFunction",
    "Class",
    "MethodDefinition",
    "TsMethodDefinition",
    "PropertyDefinition",
    "FormalParameter",
];

pub const PUBLIC_KEYS: &[(&str, [&[&str]; 2])] = &[
    ("Program", [&["sourceType", "hashbang", "body"], &[]]),
    ("FunctionBody", [&["body"], &[]]),
    ("TsModuleBlock", [&["body"], &[]]),
    (
        "FormalParameter",
        [
            &["left", "right", "name", "elements", "properties"],
            &["decorators", "optional", "typeAnnotation"],
        ],
    ),
    (
        "TsIndexSignatureParameter",
        [&["name"], &["decorators", "optional", "typeAnnotation"]],
    ),
    (
        "FormalParameterRest",
        [
            &["argument"],
            &["decorators", "optional", "typeAnnotation", "value"],
        ],
    ),
    ("VariableDeclarator", [&["id", "init"], &["definite"]]),
    ("CatchClause", [&["param", "body"], &[]]),
    (
        "Function",
        [
            &["id", "generator", "async", "params", "body", "expression"],
            &["typeParameters", "returnType", "declare"],
        ],
    ),
    (
        "TsFunction",
        [
            &[
                "id",
                "generator",
                "async",
                "params",
                "body",
                "expression",
                "typeParameters",
                "returnType",
                "declare",
            ],
            &[],
        ],
    ),
    (
        "ArrowFunctionExpression",
        [
            &["id", "generator", "async", "params", "body", "expression"],
            &["typeParameters", "returnType"],
        ],
    ),
    ("Directive", [&["expression", "directive"], &[]]),
    ("StringLiteral", [&["value", "raw"], &[]]),
    ("NumericLiteral", [&["value", "raw"], &[]]),
    ("BigIntLiteral", [&["value", "raw", "bigint"], &[]]),
    ("BooleanLiteral", [&["value", "raw"], &[]]),
    ("NullLiteral", [&["value", "raw"], &[]]),
    ("RegExpLiteral", [&["value", "raw", "regex"], &[]]),
    ("TemplateElement", [&["value", "tail"], &[]]),
    (
        "Class",
        [
            &["decorators", "id", "superClass", "body"],
            &[
                "typeParameters",
                "superTypeArguments",
                "implements",
                "abstract",
                "declare",
            ],
        ],
    ),
    (
        "MethodDefinition",
        [
            &["decorators", "key", "value", "kind", "computed", "static"],
            &["override", "optional", "accessibility"],
        ],
    ),
    (
        "TsMethodDefinition",
        [
            &["decorators", "key", "value", "kind", "computed", "static"],
            &["override", "optional", "accessibility"],
        ],
    ),
    (
        "PropertyDefinition",
        [
            &["decorators", "key", "value", "computed", "static"],
            &[
                "typeAnnotation",
                "declare",
                "override",
                "optional",
                "definite",
                "readonly",
                "accessibility",
            ],
        ],
    ),
    (
        "UnaryExpression",
        [&["operator", "prefix", "argument"], &[]],
    ),
    (
        "BindingProperty",
        [
            &["kind", "key", "value", "method", "shorthand", "computed"],
            &["optional"],
        ],
    ),
    (
        "ArrayPattern",
        [&["elements"], &["decorators", "optional", "typeAnnotation"]],
    ),
    (
        "ObjectPattern",
        [
            &["properties"],
            &["decorators", "optional", "typeAnnotation"],
        ],
    ),
    ("JsxText", [&["value", "raw"], &[]]),
    (
        "TsFunctionType",
        [&["typeParameters", "params", "returnType"], &[]],
    ),
    (
        "TsConstructorType",
        [&["abstract", "typeParameters", "params", "returnType"], &[]],
    ),
    (
        "TsMethodSignature",
        [
            &[
                "key",
                "computed",
                "optional",
                "kind",
                "typeParameters",
                "params",
                "returnType",
                "accessibility",
                "readonly",
                "static",
            ],
            &[],
        ],
    ),
    (
        "TsCallSignatureDeclaration",
        [&["typeParameters", "params", "returnType"], &[]],
    ),
    (
        "TsConstructSignatureDeclaration",
        [&["typeParameters", "params", "returnType"], &[]],
    ),
    (
        "TsMappedType",
        [
            &[
                "key",
                "constraint",
                "nameType",
                "typeAnnotation",
                "optional",
                "readonly",
            ],
            &[],
        ],
    ),
    (
        "TsModuleDeclaration",
        [
            &["id", "attributes", "kind", "declare", "global", "body"],
            &[],
        ],
    ),
    (
        "TsGlobalDeclaration",
        [&["id", "body", "kind", "declare", "global"], &[]],
    ),
    (
        "TsThisParameter",
        [&["decorators", "name", "optional", "typeAnnotation"], &[]],
    ),
];

pub const FIELD_RULES: &[(&str, &str, &str)] = &[
    ("Program", "sourceType", "SourceType"),
    ("Program", "body", "List"),
    ("FunctionBody", "body", "List"),
    ("TsModuleBlock", "body", "List"),
    ("ArrayPattern", "elements", "List"),
    ("ObjectPattern", "properties", "List"),
    ("FormalParameter", "left", "Pattern"),
    ("FormalParameter", "right", "Initializer"),
    ("FormalParameter", "name", "DelegateName"),
    ("FormalParameter", "elements", "DelegateList"),
    ("FormalParameter", "properties", "DelegateList"),
    ("FormalParameter", "optional", "ParameterOptional"),
    ("FormalParameter", "typeAnnotation", "ParameterAnnotation"),
    ("TsIndexSignatureParameter", "name", "DelegateName"),
    ("TsIndexSignatureParameter", "decorators", "EmptyList"),
    ("TsIndexSignatureParameter", "optional", "False"),
    ("FormalParameterRest", "optional", "False"),
    ("FormalParameterRest", "value", "Null"),
    ("Function", "expression", "False"),
    ("Function", "declare", "False"),
    ("TsFunction", "expression", "False"),
    ("TsFunction", "body", "Null"),
    ("ArrowFunctionExpression", "id", "Null"),
    ("ArrowFunctionExpression", "generator", "False"),
    ("Directive", "directive", "Directive"),
    ("NumericLiteral", "raw", "NumericRaw"),
    ("BigIntLiteral", "raw", "BigIntRaw"),
    ("BigIntLiteral", "value", "BigInt"),
    ("BigIntLiteral", "bigint", "BigIntText"),
    ("BooleanLiteral", "raw", "BooleanRaw"),
    ("NullLiteral", "raw", "NullRaw"),
    ("NullLiteral", "value", "Null"),
    ("RegExpLiteral", "raw", "RegexRaw"),
    ("RegExpLiteral", "regex", "RegexFields"),
    ("RegExpLiteral", "value", "RegexValue"),
    ("TemplateElement", "value", "TemplateValue"),
    ("UnaryExpression", "prefix", "True"),
    ("BindingProperty", "kind", "Init"),
    ("BindingProperty", "method", "False"),
    ("BindingProperty", "optional", "False"),
    ("JsxText", "raw", "Text"),
    ("TsMethodSignature", "accessibility", "Null"),
    ("TsMethodSignature", "readonly", "False"),
    ("TsMethodSignature", "static", "False"),
    ("TsMappedType", "optional", "Optional"),
    ("TsModuleDeclaration", "global", "False"),
    ("TsModuleDeclaration", "body", "ModuleBody"),
    ("TsGlobalDeclaration", "kind", "Global"),
    ("TsGlobalDeclaration", "global", "True"),
    ("TsThisParameter", "name", "This"),
    ("TsThisParameter", "decorators", "List"),
    ("TsThisParameter", "optional", "False"),
];

pub const TYPE_OVERRIDES: &[(&str, &str)] = &[
    ("FunctionBody", "BlockStatement"),
    ("FormalParameterRest", "RestElement"),
    ("TsIndexSignatureParameter", "Identifier"),
    ("Directive", "ExpressionStatement"),
    ("BindingProperty", "Property"),
    ("TsGlobalDeclaration", "TSModuleDeclaration"),
    ("TsThisParameter", "Identifier"),
];
