use swc_next_ecma_ast::{Expr, NodeId, NodeKindData, ScopeId, is_identifier};
use swc_next_ecma_semantic::{SymbolFlags, scope::ScopeKind};

use super::{BoundIdentifier, Mutation};

impl Mutation<'_, '_, '_> {
    /// Babel: scope.maybeGenerateMemoised(node), with the default dontPush=false.
    /// Possibly generate a memoised identifier if it is not static and has consequences.
    pub fn maybe_generate_memoised(
        &mut self,
        scope: ScopeId,
        node: Expr,
    ) -> Option<BoundIdentifier> {
        if self.is_static(node) {
            return None;
        }
        let name = self.memoised_uid_base(node);
        Some(self.generate_declared_uid_identifier(scope, &name))
    }

    /// Babel: Scope.isStatic. Use Semantic's resolved references and use indexes
    /// instead of resolving identifiers again in the transform.
    fn is_static(&self, node: Expr) -> bool {
        match self.ast.kind_data(node.node_id()) {
            NodeKindData::ThisExpression(_) | NodeKindData::Super(_) => true,
            NodeKindData::IdentifierReference(id) => {
                let Some(reference) = self.semantic().reference_of(id.node_id()) else {
                    return false;
                };
                let reference = self.semantic().reference(reference);
                if reference.flags.is_dynamic() || reference.flags.is_uncertain() {
                    return false;
                }
                let Some(symbol) = reference.symbol else {
                    // Scope.hasBinding also accepts the names in Babel's
                    // helper-globals data and Scope.contextVariables.
                    return is_babel_global(self.ast.get_utf8(id.name(self.ast)));
                };
                let mut declarations = self.semantic().declarations(symbol);
                let declaration = declarations.next().expect("a binding has a declaration");
                if declarations.next().is_some() {
                    // Babel registerBinding treats redeclaration as reassignment.
                    return false;
                }
                let flags = declaration.flags;
                // TODO: Babel Binding.constant also accounts for isInitInLoop
                // (scope/binding.ts). ScopeModel does not retain initializer/loop
                // ownership for declarations. Until it does, conservatively
                // memoise vars and hoisted functions; parameters/catch bindings
                // and named function expressions are not loop-initialized vars.
                if (flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR)
                    && !flags.intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR))
                    || (flags.contains(SymbolFlags::FUNCTION)
                        && self.semantic().scope(declaration.scope).kind
                            != ScopeKind::ExpressionName)
                {
                    return false;
                }
                self.semantic()
                    .uses(symbol)
                    .iter()
                    .all(|&id| !self.semantic().reference(id).flags.is_write())
            }
            _ => false,
        }
    }

    /// The name-seed portion of Babel generateUidBasedOnNode. Reservation and
    /// declaration are delegated to generate_declared_uid_identifier.
    fn memoised_uid_base(&self, node: Expr) -> String {
        let mut parts = Vec::new();
        self.gather_node_parts(node.node_id(), &mut parts);
        let id = parts.join("$");
        let id = id.strip_prefix('_').unwrap_or(&id);
        let id = if id.is_empty() { "ref" } else { id };

        // Babel uses id.slice(0, 20), measured in UTF-16 code units. If slicing
        // splits a surrogate pair, toIdentifier discards the trailing surrogate.
        let mut units = 0;
        let id: String = id
            .chars()
            .take_while(|c| {
                units += c.len_utf16();
                units <= 20
            })
            .collect();
        // toIdentifier replaces invalid identifier characters with dashes,
        // removes leading dashes/digits and camel-cases the remaining parts.
        let dashed: String = id
            .chars()
            .map(|c| {
                if is_identifier(format!("_{c}").as_bytes()) {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let mut name = String::new();
        let mut uppercase = false;
        for c in dashed
            .trim_start_matches(|c: char| c == '-' || c.is_ascii_digit())
            .chars()
        {
            if c == '-' {
                uppercase = true;
            } else if uppercase {
                name.extend(c.to_uppercase());
                uppercase = false;
            } else {
                name.push(c);
            }
        }
        // generateUid strips leading underscores and trailing digits. Its
        // final underscore prefix also makes reserved words valid identifiers.
        name.trim_start_matches('_')
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_owned()
    }

    /// Babel's gatherNodeParts, restricted to expression trees used as keys.
    /// Recursively gathers the identifying names of a node.
    fn gather_node_parts(&self, node: NodeId, parts: &mut Vec<String>) {
        match self.ast.kind_data(node) {
            NodeKindData::IdentifierReference(id) => {
                parts.push(self.ast.get_utf8(id.name(self.ast)).into())
            }
            NodeKindData::IdentifierName(id) => {
                parts.push(self.ast.get_utf8(id.name(self.ast)).into())
            }
            NodeKindData::BindingIdentifier(id) => {
                parts.push(self.ast.get_utf8(id.name(self.ast)).into())
            }
            NodeKindData::PrivateIdentifier(id) => {
                parts.push(self.ast.get_utf8(id.name(self.ast)).into())
            }
            NodeKindData::StringLiteral(literal) => parts.push(
                self.ast
                    .get_wtf8(literal.value(self.ast))
                    .to_string_lossy()
                    .into_owned(),
            ),
            NodeKindData::BooleanLiteral(literal) => {
                parts.push(literal.value(self.ast).to_string())
            }
            NodeKindData::NumericLiteral(_) | NodeKindData::BigIntLiteral(_) => {
                // TODO: Babel pushes literal.value, then parts.join performs
                // ECMAScript Number/BigInt string coercion. Until the existing
                // literal-value conversion gap is filled, use an equivalent,
                // collision-checked ref seed; this does not alter the key value.
                parts.push("ref".into());
            }
            NodeKindData::ThisExpression(_) => parts.push("this".into()),
            NodeKindData::Super(_) => parts.push("super".into()),
            NodeKindData::ImportExpression(_) => parts.push("import".into()),
            NodeKindData::MemberExpression(member) => {
                self.gather_node_parts(member.object(self.ast).node_id(), parts);
                self.gather_node_parts(member.property(self.ast).node_id(), parts);
            }
            NodeKindData::CallExpression(call) => {
                self.gather_node_parts(call.callee(self.ast).node_id(), parts)
            }
            NodeKindData::NewExpression(call) => {
                self.gather_node_parts(call.callee(self.ast).node_id(), parts)
            }
            NodeKindData::ChainExpression(chain) => {
                self.gather_node_parts(chain.expression(self.ast).node_id(), parts)
            }
            NodeKindData::ObjectExpression(object) => {
                for slot in object.properties(self.ast).iter() {
                    self.gather_node_parts(self.ast.get_node_in_sub_range(slot).node_id(), parts);
                }
            }
            NodeKindData::ObjectProperty(property) => {
                self.gather_node_parts(property.key(self.ast).node_id(), parts)
            }
            NodeKindData::SpreadElement(spread) => {
                self.gather_node_parts(spread.argument(self.ast).node_id(), parts)
            }
            NodeKindData::AssignmentExpression(assignment) => {
                self.gather_node_parts(assignment.left(self.ast).node_id(), parts)
            }
            NodeKindData::ObjectPattern(target) => {
                for slot in target.properties(self.ast).iter() {
                    self.gather_node_parts(self.ast.get_node_in_sub_range(slot).node_id(), parts);
                }
                if let Some(rest) = target.rest(self.ast) {
                    self.gather_node_parts(rest.node_id(), parts);
                }
            }
            NodeKindData::BindingProperty(property) => {
                self.gather_node_parts(property.key(self.ast).node_id(), parts)
            }
            NodeKindData::BindingRestElement(rest) => {
                self.gather_node_parts(rest.argument(self.ast).node_id(), parts)
            }
            NodeKindData::AwaitExpression(await_expr) => {
                parts.push("await".into());
                self.gather_node_parts(await_expr.argument(self.ast).node_id(), parts);
            }
            NodeKindData::YieldExpression(yield_expr) => {
                parts.push("yield".into());
                if let Some(argument) = yield_expr.argument(self.ast) {
                    self.gather_node_parts(argument.node_id(), parts);
                }
            }
            NodeKindData::Function(function) => {
                if let Some(id) = function.id(self.ast) {
                    self.gather_node_parts(id.node_id(), parts);
                }
            }
            NodeKindData::Class(class) => {
                if let Some(id) = class.id(self.ast) {
                    self.gather_node_parts(id.node_id(), parts);
                }
            }
            NodeKindData::ParenthesizedExpression(expr) => {
                self.gather_node_parts(expr.expression(self.ast).node_id(), parts)
            }
            NodeKindData::UnaryExpression(expr) => {
                self.gather_node_parts(expr.argument(self.ast).node_id(), parts)
            }
            NodeKindData::UpdateExpression(expr) => {
                self.gather_node_parts(expr.argument(self.ast).node_id(), parts)
            }
            NodeKindData::MetaProperty(meta) => {
                self.gather_node_parts(meta.meta(self.ast).node_id(), parts);
                self.gather_node_parts(meta.property(self.ast).node_id(), parts);
            }
            NodeKindData::JsxElement(element) => {
                self.gather_node_parts(element.opening_element(self.ast).node_id(), parts)
            }
            NodeKindData::JsxOpeningElement(element) => {
                self.gather_node_parts(element.name(self.ast).node_id(), parts)
            }
            NodeKindData::JsxIdentifier(id) => {
                parts.push(self.ast.get_utf8(id.name(self.ast)).into())
            }
            NodeKindData::JsxFragment(_) => parts.push("Fragment".into()),
            NodeKindData::JsxNamespacedName(name) => {
                self.gather_node_parts(name.namespace(self.ast).node_id(), parts);
                self.gather_node_parts(name.name(self.ast).node_id(), parts);
            }
            NodeKindData::JsxMemberExpression(member) => {
                self.gather_node_parts(member.object(self.ast).node_id(), parts);
                self.gather_node_parts(member.property(self.ast).node_id(), parts);
            }
            // Like Babel, other expressions (including binary/sequence,
            // template, null and regexp expressions) contribute no name parts.
            _ => {}
        }
    }
}

// Babel helper-globals/{builtin-lower,builtin-upper}.json and contextVariables.
fn is_babel_global(name: &str) -> bool {
    matches!(
        name,
        "decodeURI"
            | "decodeURIComponent"
            | "encodeURI"
            | "encodeURIComponent"
            | "escape"
            | "eval"
            | "globalThis"
            | "isFinite"
            | "isNaN"
            | "parseFloat"
            | "parseInt"
            | "undefined"
            | "unescape"
            | "arguments"
            | "AggregateError"
            | "Array"
            | "ArrayBuffer"
            | "Atomics"
            | "BigInt"
            | "BigInt64Array"
            | "BigUint64Array"
            | "Boolean"
            | "DataView"
            | "Date"
            | "Error"
            | "EvalError"
            | "FinalizationRegistry"
            | "Float16Array"
            | "Float32Array"
            | "Float64Array"
            | "Function"
            | "Infinity"
            | "Int16Array"
            | "Int32Array"
            | "Int8Array"
            | "Intl"
            | "Iterator"
            | "JSON"
            | "Map"
            | "Math"
            | "NaN"
            | "Number"
            | "Object"
            | "Promise"
            | "Proxy"
            | "RangeError"
            | "ReferenceError"
            | "Reflect"
            | "RegExp"
            | "Set"
            | "SharedArrayBuffer"
            | "String"
            | "Symbol"
            | "SyntaxError"
            | "TypeError"
            | "Uint16Array"
            | "Uint32Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "URIError"
            | "WeakMap"
            | "WeakRef"
            | "WeakSet"
    )
}
