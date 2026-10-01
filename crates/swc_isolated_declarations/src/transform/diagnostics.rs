//! Shared isolated-declaration diagnostic messages.

pub(super) const VARIABLE_TYPE_REQUIRED: &str =
    "Variable must have an explicit type annotation with --isolatedDeclarations.";
pub(super) const PROPERTY_TYPE_REQUIRED: &str =
    "Property must have an explicit type annotation with --isolatedDeclarations.";
pub(super) const FUNCTION_RETURN_TYPE_REQUIRED: &str =
    "Function must have an explicit return type annotation with --isolatedDeclarations.";
pub(super) const METHOD_RETURN_TYPE_REQUIRED: &str =
    "Method must have an explicit return type annotation with --isolatedDeclarations.";
pub(super) const ACCESSOR_TYPE_REQUIRED: &str =
    "At least one accessor must have an explicit type annotation with --isolatedDeclarations.";
pub(super) const PARAMETER_TYPE_REQUIRED: &str =
    "Parameter must have an explicit type annotation with --isolatedDeclarations.";
pub(super) const DEFAULT_EXPORT_TYPE_REQUIRED: &str =
    "Default exports can't be inferred with --isolatedDeclarations.";
pub(super) const EXTENDS_EXPRESSION_NOT_SUPPORTED: &str =
    "Extends clause can't contain an expression with --isolatedDeclarations.";
pub(super) const COMPUTED_PROPERTY_NOT_SUPPORTED: &str = "Computed property names on class or object literals cannot be inferred with --isolatedDeclarations.";
pub(super) const COMPUTED_SIGNATURE_NOT_SUPPORTED: &str = "Computed properties must be number or string literals, variables or dotted expressions with --isolatedDeclarations.";
pub(super) const EXPRESSION_TYPE_REQUIRED: &str =
    "Expression type can't be inferred with --isolatedDeclarations.";
pub(super) const OBJECT_SPREAD_NOT_SUPPORTED: &str =
    "Objects that contain spread assignments can't be inferred with --isolatedDeclarations.";
pub(super) const OBJECT_SHORTHAND_NOT_SUPPORTED: &str =
    "Objects that contain shorthand properties can't be inferred with --isolatedDeclarations.";
pub(super) const ARRAY_REQUIRES_CONST: &str =
    "Only const arrays can be inferred with --isolatedDeclarations.";
pub(super) const ARRAY_SPREAD_NOT_SUPPORTED: &str =
    "Arrays with spread elements can't be inferred with --isolatedDeclarations.";
pub(super) const CLASS_EXPRESSION_NOT_SUPPORTED: &str =
    "Inference from class expressions is not supported with --isolatedDeclarations.";
pub(super) const BINDING_ELEMENT_NOT_SUPPORTED: &str =
    "Binding elements can't be exported directly with --isolatedDeclarations.";
pub(super) const EXPANDO_FUNCTION_NOT_SUPPORTED: &str = "Assigning properties to functions without declaring them is not supported with --isolatedDeclarations. Add an explicit declaration for the properties assigned to this function.";
