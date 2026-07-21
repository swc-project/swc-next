use swc_experimental_allocator::{Allocator, vec::Vec};
use swc_experimental_ecma_ast::{
    Ast, AstNode, AttachedComment, AttachedCommentPosition, CommentType, ExtraDataCompact, NodeId,
    NodeKind, NodeKindData, Program, TypedSubRange,
};

use crate::{sourcemap, utils};

pub use sourcemap::{Options as SourceMapOptions, SourceMap};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Format {
    #[default]
    Pretty,
    Compact,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Quotes {
    #[default]
    Preserve,
    Double,
    Single,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Comments {
    None,
    All,
    #[default]
    Some,
    Line,
    Block,
}

#[derive(Clone, Copy)]
pub struct Options<'a> {
    pub format: Format,
    pub indent: u8,
    pub quotes: Quotes,
    pub source_maps: Option<SourceMapOptions<'a>>,
    pub comments: Comments,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            format: Format::Pretty,
            indent: 2,
            quotes: Quotes::Preserve,
            source_maps: None,
            comments: Comments::Some,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Diagnostic<'a> {
    pub message: &'a [u8],
    pub start: u32,
    pub end: u32,
}

pub struct Result<'a> {
    pub code: &'a [u8],
    pub errors: &'a [Diagnostic<'a>],
    pub map: Option<SourceMap<'a>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    OutOfMemory,
}

#[derive(Clone, Copy, Default)]
pub struct Config {
    pub strip_ts: bool,
    pub minify: bool,
}

pub fn print<'a>(
    allocator: &'a Allocator,
    tree: &'a Ast<'a>,
    root: Program,
    options: Options<'a>,
) -> std::result::Result<Result<'a>, Error> {
    print_impl(Config::default(), allocator, tree, root, options)
}

pub fn strip<'a>(
    allocator: &'a Allocator,
    tree: &'a Ast<'a>,
    root: Program,
    options: Options<'a>,
) -> std::result::Result<Result<'a>, Error> {
    print_impl(
        Config {
            strip_ts: true,
            minify: false,
        },
        allocator,
        tree,
        root,
        options,
    )
}

pub fn minify<'a>(
    allocator: &'a Allocator,
    tree: &'a Ast<'a>,
    root: Program,
    options: Options<'a>,
) -> std::result::Result<Result<'a>, Error> {
    print_impl(
        Config {
            strip_ts: false,
            minify: true,
        },
        allocator,
        tree,
        root,
        options,
    )
}

pub fn print_impl<'a>(
    config: Config,
    allocator: &'a Allocator,
    tree: &'a Ast<'a>,
    root: Program,
    options: Options<'a>,
) -> std::result::Result<Result<'a>, Error> {
    let mut printer = Printer::init(allocator, tree, options, config);
    printer.print_root(root);
    let code = std::mem::replace(&mut printer.code, Vec::new_in(allocator)).into_bump_slice();
    let errors = std::mem::replace(&mut printer.errors, Vec::new_in(allocator)).into_bump_slice();
    let map = printer.sm.as_mut().map(|state| state.build(allocator));
    Ok(Result { code, errors, map })
}

fn fixed_string(tag: NodeKind) -> Option<&'static [u8]> {
    match tag {
        NodeKind::Super => Some(b"super"),
        NodeKind::ThisExpression | NodeKind::TsThisType => Some(b"this"),
        NodeKind::NullLiteral | NodeKind::TsNullKeyword => Some(b"null"),
        NodeKind::TsAnyKeyword => Some(b"any"),
        NodeKind::TsUnknownKeyword => Some(b"unknown"),
        NodeKind::TsNeverKeyword => Some(b"never"),
        NodeKind::TsVoidKeyword => Some(b"void"),
        NodeKind::TsUndefinedKeyword => Some(b"undefined"),
        NodeKind::TsStringKeyword => Some(b"string"),
        NodeKind::TsNumberKeyword => Some(b"number"),
        NodeKind::TsBigIntKeyword => Some(b"bigint"),
        NodeKind::TsBooleanKeyword => Some(b"boolean"),
        NodeKind::TsSymbolKeyword => Some(b"symbol"),
        NodeKind::TsObjectKeyword => Some(b"object"),
        NodeKind::TsIntrinsicKeyword => Some(b"intrinsic"),
        NodeKind::TsJsDocUnknownType => Some(b"?"),
        NodeKind::JsxOpeningFragment => Some(b"<>"),
        NodeKind::JsxClosingFragment => Some(b"</>"),
        _ => None,
    }
}

struct Precedence;

impl Precedence {
    const LOWEST: u8 = 0;
    const COMMA: u8 = 1;
    const ASSIGNMENT: u8 = 2;
    const LOGICAL_OR: u8 = 3;
    const LOGICAL_AND: u8 = 4;
    const BITWISE_OR: u8 = 5;
    const BITWISE_XOR: u8 = 6;
    const BITWISE_AND: u8 = 7;
    const EQUALITY: u8 = 8;
    const RELATIONAL: u8 = 9;
    const SHIFT: u8 = 10;
    const ADDITIVE: u8 = 11;
    const MULTIPLICATIVE: u8 = 12;
    const EXPONENTIATION: u8 = 13;
    const UNARY: u8 = 14;
    const POSTFIX: u8 = 15;
    const NEW: u8 = 16;
    const CALL: u8 = 17;
    const GROUPING: u8 = 18;
}

struct TypePrecedence;

impl TypePrecedence {
    const TRAILING: u8 = 1;
    const UNION: u8 = 2;
    const INTERSECTION: u8 = 3;
    const OPERATOR: u8 = 4;
    const PRIMARY: u8 = 5;
}

fn binary_precedence(operator: swc_experimental_ecma_ast::BinaryOperator) -> u8 {
    use swc_experimental_ecma_ast::BinaryOperator as Op;
    match operator {
        Op::BitwiseOr => Precedence::BITWISE_OR,
        Op::BitwiseXor => Precedence::BITWISE_XOR,
        Op::BitwiseAnd => Precedence::BITWISE_AND,
        Op::Equal | Op::NotEqual | Op::StrictEqual | Op::StrictNotEqual => Precedence::EQUALITY,
        Op::LessThan
        | Op::LessThanOrEqual
        | Op::GreaterThan
        | Op::GreaterThanOrEqual
        | Op::In
        | Op::Instanceof => Precedence::RELATIONAL,
        Op::LeftShift | Op::RightShift | Op::UnsignedRightShift => Precedence::SHIFT,
        Op::Add | Op::Subtract => Precedence::ADDITIVE,
        Op::Multiply | Op::Divide | Op::Modulo => Precedence::MULTIPLICATIVE,
        Op::Exponent => Precedence::EXPONENTIATION,
    }
}

fn logical_precedence(operator: swc_experimental_ecma_ast::LogicalOperator) -> u8 {
    use swc_experimental_ecma_ast::LogicalOperator as Op;
    match operator {
        Op::Or | Op::NullishCoalescing => Precedence::LOGICAL_OR,
        Op::And => Precedence::LOGICAL_AND,
    }
}

#[derive(Clone, Copy, Default)]
struct ExprContext {
    precedence: u8,
    no_in: bool,
    no_call: bool,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum Lead {
    #[default]
    None,
    Statement,
    Arrow,
}

struct Printer<'a> {
    tree: &'a Ast<'a>,
    code: Vec<'a, u8>,
    errors: Vec<'a, Diagnostic<'a>>,
    options: Options<'a>,
    config: Config,
    allocator: &'a Allocator,
    indent_depth: u32,
    in_assign_target: bool,
    pending_semi: bool,
    sm: Option<sourcemap::State<'a>>,
    current_idx: Option<NodeId>,
    definite_pending: bool,
    skip_leading_of: Option<NodeId>,
    at_lead: Lead,
    in_prologue: bool,
    declaration_no_in: bool,
}

#[derive(Clone, Copy)]
struct Cursor {
    code: usize,
    sm: Option<sourcemap::Snapshot>,
}

impl<'a> Printer<'a> {
    fn init(
        allocator: &'a Allocator,
        tree: &'a Ast<'a>,
        options: Options<'a>,
        config: Config,
    ) -> Self {
        let mut code = Vec::with_capacity_in(tree.source().len(), allocator);
        let sm = options.source_maps.map(|options| {
            let mut state = sourcemap::State::init(options, allocator);
            state.out.reserve(tree.node_count() * 8 + 64);
            state
        });
        code.reserve(0);
        Self {
            tree,
            code,
            errors: Vec::new_in(allocator),
            options,
            config,
            allocator,
            indent_depth: 0,
            in_assign_target: false,
            pending_semi: false,
            sm,
            current_idx: None,
            definite_pending: false,
            skip_leading_of: None,
            at_lead: Lead::None,
            in_prologue: false,
            declaration_no_in: false,
        }
    }

    #[inline]
    fn pretty(&self) -> bool {
        self.options.format == Format::Pretty
    }

    #[inline]
    fn node_data(&self, idx: NodeId) -> NodeKindData {
        self.tree.kind_data(idx)
    }

    #[inline]
    fn push_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    #[inline]
    fn push_slice(&mut self, slice: &[u8]) {
        self.code.extend_from_slice_copy(slice);
    }

    #[inline]
    fn write_byte(&mut self, byte: u8) {
        self.drop_pending_keyword_space(byte);
        self.at_lead = Lead::None;
        self.push_byte(byte);
        if let Some(sm) = &mut self.sm {
            if byte == b'\n' {
                sm.gen_line += 1;
                sm.gen_col = 0;
            } else {
                sm.gen_col += 1;
            }
        }
    }

    #[inline]
    fn write_str(&mut self, slice: &[u8]) {
        if slice.is_empty() {
            return;
        }
        self.drop_pending_keyword_space(slice[0]);
        self.at_lead = Lead::None;
        self.push_slice(slice);
        if let Some(sm) = &mut self.sm {
            sm.advance(slice);
        }
    }

    #[inline]
    fn drop_pending_keyword_space(&mut self, next: u8) {
        if self.pretty() || self.code.len() < 2 || self.code[self.code.len() - 1] != b' ' {
            return;
        }
        if utils::is_id_cont(self.code[self.code.len() - 2]) && !utils::is_id_cont(next) {
            self.code.pop();
            if let Some(sm) = &mut self.sm {
                sm.gen_col = sm.gen_col.saturating_sub(1);
            }
        }
    }

    #[inline]
    fn write_string(&mut self, id: swc_experimental_ecma_ast::String) {
        self.write_str(self.tree.string(id));
    }

    #[inline]
    fn space(&mut self) {
        if self.pretty() {
            self.write_byte(b' ');
        }
    }

    #[inline]
    fn comma(&mut self) {
        self.write_byte(b',');
        self.space();
    }

    #[inline]
    fn print_eq(&mut self) {
        self.space();
        self.write_byte(b'=');
        self.space();
    }

    fn emit_list<T>(&mut self, items: TypedSubRange<T>)
    where
        T: AstNode + ExtraDataCompact<'a>,
    {
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                self.comma();
            }
            let node = self.tree.get_node_in_sub_range(item);
            self.emit(node.node_id());
        }
    }

    fn print_binding_suffix(&mut self, optional: bool, definite: bool, annotation: Option<NodeId>) {
        if !self.config.strip_ts && optional {
            self.write_byte(b'?');
        }
        if definite {
            self.write_byte(b'!');
        }
        if let Some(annotation) = annotation {
            self.emit(annotation);
        }
    }

    #[inline]
    fn take_definite(&mut self) -> bool {
        if self.config.strip_ts {
            return false;
        }
        let definite = self.definite_pending;
        self.definite_pending = false;
        definite
    }

    fn newline(&mut self) {
        if self.pretty() {
            self.break_line();
        }
    }

    #[inline]
    fn mark(&self) -> usize {
        self.code.len()
    }

    #[inline]
    fn cursor(&self) -> Cursor {
        Cursor {
            code: self.code.len(),
            sm: self.sm.as_ref().map(sourcemap::State::snapshot),
        }
    }

    #[inline]
    fn restore(&mut self, cursor: Cursor) {
        assert!(cursor.code <= self.code.len());
        self.code.truncate(cursor.code);
        if let (Some(snapshot), Some(sm)) = (cursor.sm, &mut self.sm) {
            sm.restore(snapshot);
        }
    }

    fn try_emit(&mut self, idx: NodeId) -> bool {
        let start = self.mark();
        let tail = start.checked_sub(1).map_or(0, |index| self.code[index]);
        self.emit(idx);
        if self.code.len() > start {
            return true;
        }
        start > 0 && self.code[start - 1] != tail
    }

    fn emit_stmt(&mut self, idx: NodeId) {
        if !self.try_emit(idx) {
            self.write_byte(b';');
        }
    }

    fn print_root(&mut self, root: Program) {
        self.emit(root.node_id());
    }

    fn emit(&mut self, idx: NodeId) {
        self.emit_expr(idx, ExprContext::default());
    }

    fn emit_expr(&mut self, idx: NodeId, context: ExprContext) {
        if self.config.strip_ts {
            match self.node_data(idx) {
                NodeKindData::TsTypeAliasDeclaration(_)
                | NodeKindData::TsInterfaceDeclaration(_)
                | NodeKindData::TsGlobalDeclaration(_)
                | NodeKindData::TsNamespaceExportDeclaration(_)
                | NodeKindData::TsThisParameter(_) => return,
                NodeKindData::TsAsExpression(_)
                | NodeKindData::TsSatisfiesExpression(_)
                | NodeKindData::TsTypeAssertion(_)
                | NodeKindData::TsNonNullExpression(_)
                | NodeKindData::TsInstantiationExpression(_) => {
                    return self.emit_expr(self.stripped(idx), context);
                }
                NodeKindData::TsEnumDeclaration(declaration) => {
                    if declaration.declare(self.tree) {
                        return;
                    }
                    return self
                        .diagnose(idx, b"TypeScript enums cannot be stripped to JavaScript");
                }
                NodeKindData::TsModuleDeclaration(declaration) => {
                    if declaration.declare(self.tree) {
                        return;
                    }
                    return self.diagnose(
                        idx,
                        b"TypeScript namespaces cannot be stripped to JavaScript",
                    );
                }
                NodeKindData::TsImportEqualsDeclaration(declaration) => {
                    if declaration.import_kind(self.tree)
                        == swc_experimental_ecma_ast::ImportOrExportKind::Type
                    {
                        return;
                    }
                    return self.diagnose(
                        idx,
                        b"`import = require()` cannot be stripped to JavaScript",
                    );
                }
                NodeKindData::TsExportAssignment(_) => {
                    return self.diagnose(idx, b"`export =` cannot be stripped to JavaScript");
                }
                NodeKindData::TsParameterProperty(property) => {
                    self.diagnose(
                        idx,
                        b"parameter properties cannot be stripped to JavaScript",
                    );
                    return self.emit_expr(property.parameter(self.tree).node_id(), context);
                }
                _ if matches!(
                    self.tree.node_kind(idx),
                    NodeKind::TsTypeAnnotation
                        | NodeKind::TsAnyKeyword
                        | NodeKind::TsUnknownKeyword
                        | NodeKind::TsNeverKeyword
                        | NodeKind::TsVoidKeyword
                        | NodeKind::TsNullKeyword
                        | NodeKind::TsUndefinedKeyword
                        | NodeKind::TsStringKeyword
                        | NodeKind::TsNumberKeyword
                        | NodeKind::TsBigIntKeyword
                        | NodeKind::TsBooleanKeyword
                        | NodeKind::TsSymbolKeyword
                        | NodeKind::TsObjectKeyword
                        | NodeKind::TsIntrinsicKeyword
                        | NodeKind::TsThisType
                        | NodeKind::TsTypeReference
                        | NodeKind::TsQualifiedName
                        | NodeKind::TsTypeQuery
                        | NodeKind::TsImportType
                        | NodeKind::TsTypeParameter
                        | NodeKind::TsTypeParameterDeclaration
                        | NodeKind::TsTypeParameterInstantiation
                        | NodeKind::TsLiteralType
                        | NodeKind::TsTemplateLiteralType
                        | NodeKind::TsArrayType
                        | NodeKind::TsIndexedAccessType
                        | NodeKind::TsTupleType
                        | NodeKind::TsNamedTupleMember
                        | NodeKind::TsOptionalType
                        | NodeKind::TsRestType
                        | NodeKind::TsJsDocNullableType
                        | NodeKind::TsJsDocNonNullableType
                        | NodeKind::TsJsDocUnknownType
                        | NodeKind::TsUnionType
                        | NodeKind::TsIntersectionType
                        | NodeKind::TsConditionalType
                        | NodeKind::TsInferType
                        | NodeKind::TsTypeOperator
                        | NodeKind::TsParenthesizedType
                        | NodeKind::TsFunctionType
                        | NodeKind::TsConstructorType
                        | NodeKind::TsTypePredicate
                        | NodeKind::TsTypeLiteral
                        | NodeKind::TsMappedType
                        | NodeKind::TsPropertySignature
                        | NodeKind::TsMethodSignature
                        | NodeKind::TsCallSignatureDeclaration
                        | NodeKind::TsConstructSignatureDeclaration
                        | NodeKind::TsIndexSignature
                        | NodeKind::TsInterfaceBody
                        | NodeKind::TsInterfaceHeritage
                        | NodeKind::TsClassImplements
                        | NodeKind::TsEnumBody
                        | NodeKind::TsEnumMember
                        | NodeKind::TsModuleBlock
                        | NodeKind::TsExternalModuleReference
                ) =>
                {
                    return;
                }
                _ => {}
            }
        }

        let wrap = self.needs_parens(idx, context);
        let inner = if wrap {
            ExprContext::default()
        } else {
            context
        };
        if wrap {
            self.write_byte(b'(');
        }
        if self.options.comments != Comments::None {
            let previous = self.current_idx;
            self.current_idx = Some(idx);
            let saved_lead = self.at_lead;
            self.emit_leading_comments(idx);
            self.at_lead = saved_lead;
            self.emit_node(idx, inner);
            self.emit_trailing_comments(idx);
            self.current_idx = previous;
        } else {
            self.emit_node(idx, inner);
        }
        if wrap {
            self.write_byte(b')');
        }
    }

    fn precedence_of(&self, idx: NodeId) -> u8 {
        match self.node_data(idx) {
            NodeKindData::SequenceExpression(_) => Precedence::COMMA,
            NodeKindData::AssignmentExpression(_)
            | NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::YieldExpression(_)
            | NodeKindData::ConditionalExpression(_) => Precedence::ASSIGNMENT,
            NodeKindData::LogicalExpression(expression) => {
                logical_precedence(expression.operator(self.tree))
            }
            NodeKindData::BinaryExpression(expression) => {
                binary_precedence(expression.operator(self.tree))
            }
            NodeKindData::UnaryExpression(_)
            | NodeKindData::AwaitExpression(_)
            | NodeKindData::TsTypeAssertion(_) => Precedence::UNARY,
            NodeKindData::UpdateExpression(_) => Precedence::POSTFIX,
            NodeKindData::TsAsExpression(_) | NodeKindData::TsSatisfiesExpression(_) => {
                Precedence::RELATIONAL
            }
            NodeKindData::NewExpression(_)
            | NodeKindData::CallExpression(_)
            | NodeKindData::MemberExpression(_)
            | NodeKindData::ChainExpression(_)
            | NodeKindData::TaggedTemplateExpression(_)
            | NodeKindData::ImportExpression(_)
            | NodeKindData::TsNonNullExpression(_)
            | NodeKindData::TsInstantiationExpression(_) => Precedence::CALL,
            NodeKindData::BooleanLiteral(_) if self.config.minify => Precedence::UNARY,
            NodeKindData::IdentifierReference(identifier)
                if self.config.minify && !self.in_assign_target =>
            {
                match self.tree.string(identifier.name(self.tree)) {
                    b"undefined" => Precedence::UNARY,
                    b"Infinity" => Precedence::MULTIPLICATIVE,
                    _ => Precedence::GROUPING,
                }
            }
            _ => Precedence::GROUPING,
        }
    }

    fn needs_parens(&self, idx: NodeId, context: ExprContext) -> bool {
        let data = self.node_data(idx);
        if self.at_lead != Lead::None {
            match data {
                NodeKindData::ObjectExpression(_) => return true,
                NodeKindData::AssignmentExpression(expression)
                    if self.tree.node_kind(expression.left(self.tree).node_id())
                        == NodeKind::ObjectPattern =>
                {
                    return true;
                }
                _ => {}
            }
        }
        if self.at_lead == Lead::Statement {
            match data {
                NodeKindData::Function(function)
                    if matches!(
                        function.function_type(self.tree),
                        swc_experimental_ecma_ast::FunctionType::FunctionExpression
                            | swc_experimental_ecma_ast::FunctionType::TsEmptyBodyFunctionExpression
                    ) =>
                {
                    return true;
                }
                NodeKindData::Class(class)
                    if class.class_type(self.tree)
                        == swc_experimental_ecma_ast::ClassType::ClassExpression =>
                {
                    return true;
                }
                NodeKindData::MemberExpression(member)
                    if member.computed(self.tree)
                        && self.is_let_identifier(member.object(self.tree).node_id()) =>
                {
                    return true;
                }
                _ => {}
            }
        }
        if context.no_call
            && matches!(
                data,
                NodeKindData::CallExpression(_)
                    | NodeKindData::ImportExpression(_)
                    | NodeKindData::ChainExpression(_)
            )
        {
            return true;
        }
        if context.precedence >= Precedence::CALL
            && matches!(data, NodeKindData::ChainExpression(_))
        {
            return true;
        }
        if context.no_in
            && matches!(
                data,
                NodeKindData::BinaryExpression(expression)
                    if expression.operator(self.tree)
                        == swc_experimental_ecma_ast::BinaryOperator::In
            )
        {
            return true;
        }
        self.precedence_of(idx) < context.precedence
    }

    fn is_let_identifier(&self, idx: NodeId) -> bool {
        matches!(
            self.node_data(idx),
            NodeKindData::IdentifierReference(identifier)
                if self.tree.string(identifier.name(self.tree)) == b"let"
        )
    }

    fn stripped(&self, mut idx: NodeId) -> NodeId {
        if !self.config.strip_ts {
            return idx;
        }
        loop {
            idx = match self.node_data(idx) {
                NodeKindData::TsAsExpression(expression) => {
                    expression.expression(self.tree).node_id()
                }
                NodeKindData::TsSatisfiesExpression(expression) => {
                    expression.expression(self.tree).node_id()
                }
                NodeKindData::TsTypeAssertion(expression) => {
                    expression.expression(self.tree).node_id()
                }
                NodeKindData::TsNonNullExpression(expression) => {
                    expression.expression(self.tree).node_id()
                }
                NodeKindData::TsInstantiationExpression(expression) => {
                    expression.expression(self.tree).node_id()
                }
                _ => return idx,
            };
        }
    }

    #[inline]
    fn emit_node(&mut self, idx: NodeId, context: ExprContext) {
        if self.sm.is_some() {
            self.record_mapping(idx);
        }
        if let Some(fixed) = fixed_string(self.tree.node_kind(idx)) {
            self.write_str(fixed);
            return;
        }
        match self.tree.kind_data(idx) {
            NodeKindData::Program(node) => self.emit_program(node),
            NodeKindData::BlockStatement(node) => self.emit_block_statement(node),
            NodeKindData::FunctionBody(node) => self.emit_function_body(node),
            NodeKindData::StaticBlock(node) => self.emit_static_block(node),
            NodeKindData::Directive(node) => self.emit_directive(node),
            NodeKindData::EmptyStatement(node) => self.emit_empty_statement(node),
            NodeKindData::DebuggerStatement(node) => self.emit_debugger_statement(node),
            NodeKindData::ExpressionStatement(node) => self.emit_expression_statement(node),
            NodeKindData::IfStatement(node) => self.emit_if_statement(node),
            NodeKindData::ReturnStatement(node) => self.emit_return_statement(node),
            NodeKindData::ThrowStatement(node) => self.emit_throw_statement(node),
            NodeKindData::BreakStatement(node) => self.emit_break_statement(node),
            NodeKindData::ContinueStatement(node) => self.emit_continue_statement(node),
            NodeKindData::LabeledStatement(node) => self.emit_labeled_statement(node),
            NodeKindData::WithStatement(node) => self.emit_with_statement(node),
            NodeKindData::WhileStatement(node) => self.emit_while_statement(node),
            NodeKindData::DoWhileStatement(node) => self.emit_do_while_statement(node),
            NodeKindData::ForStatement(node) => self.emit_for_statement(node),
            NodeKindData::ForInStatement(node) => self.emit_for_in_statement(node),
            NodeKindData::ForOfStatement(node) => self.emit_for_of_statement(node),
            NodeKindData::SwitchStatement(node) => self.emit_switch_statement(node),
            NodeKindData::SwitchCase(node) => self.emit_switch_case(node),
            NodeKindData::TryStatement(node) => self.emit_try_statement(node),
            NodeKindData::CatchClause(node) => self.emit_catch_clause(node),
            NodeKindData::VariableDeclaration(node) => self.emit_variable_declaration(node),
            NodeKindData::VariableDeclarator(node) => self.emit_variable_declarator(node),
            NodeKindData::SequenceExpression(node) => self.emit_sequence_expression(node, context),
            NodeKindData::ParenthesizedExpression(node) => self.emit_parenthesized_expression(node),
            NodeKindData::BinaryExpression(node) => self.emit_binary_expression(node, context),
            NodeKindData::LogicalExpression(node) => self.emit_logical_expression(node, context),
            NodeKindData::ConditionalExpression(node) => {
                self.emit_conditional_expression(node, context)
            }
            NodeKindData::UnaryExpression(node) => self.emit_unary_expression(node),
            NodeKindData::UpdateExpression(node) => self.emit_update_expression(node),
            NodeKindData::AssignmentExpression(node) => {
                self.emit_assignment_expression(node, context)
            }
            NodeKindData::ArrayExpression(node) => self.emit_array_expression(node),
            NodeKindData::ObjectExpression(node) => self.emit_object_expression(node),
            NodeKindData::ObjectProperty(node) => self.emit_object_property(node),
            NodeKindData::SpreadElement(node) => self.emit_spread_element(node),
            NodeKindData::MemberExpression(node) => self.emit_member_expression(node, context),
            NodeKindData::CallExpression(node) => self.emit_call_expression(node),
            NodeKindData::ChainExpression(node) => self.emit_chain_expression(node, context),
            NodeKindData::NewExpression(node) => self.emit_new_expression(node),
            NodeKindData::TaggedTemplateExpression(node) => {
                self.emit_tagged_template_expression(node, context)
            }
            NodeKindData::AwaitExpression(node) => self.emit_await_expression(node),
            NodeKindData::YieldExpression(node) => self.emit_yield_expression(node),
            NodeKindData::MetaProperty(node) => self.emit_meta_property(node),
            NodeKindData::StringLiteral(node) => self.emit_string_literal(node),
            NodeKindData::NumericLiteral(node) => self.emit_numeric_literal(node),
            NodeKindData::BigIntLiteral(node) => self.emit_bigint_literal(node),
            NodeKindData::BooleanLiteral(node) => self.emit_boolean_literal(node),
            NodeKindData::RegExpLiteral(node) => self.emit_regexp_literal(node),
            NodeKindData::TemplateLiteral(node) => self.emit_template_literal(node),
            NodeKindData::TemplateElement(node) => self.emit_template_element(node),
            NodeKindData::IdentifierReference(node) => self.emit_identifier_reference(node),
            NodeKindData::IdentifierName(node) => self.emit_identifier_name(node),
            NodeKindData::BindingIdentifier(node) => self.emit_binding_identifier(node),
            NodeKindData::LabelIdentifier(node) => self.emit_label_identifier(node),
            NodeKindData::PrivateIdentifier(node) => self.emit_private_identifier(node),
            NodeKindData::AssignmentPattern(node) => self.emit_assignment_pattern(node),
            NodeKindData::BindingRestElement(node) => self.emit_binding_rest_element(node),
            NodeKindData::ArrayPattern(node) => self.emit_array_pattern(node),
            NodeKindData::ObjectPattern(node) => self.emit_object_pattern(node),
            NodeKindData::BindingProperty(node) => self.emit_binding_property(node),
            NodeKindData::Function(node) => self.emit_function(node),
            NodeKindData::ArrowFunctionExpression(node) => {
                self.emit_arrow_function_expression(node)
            }
            NodeKindData::FormalParameters(node) => self.emit_formal_parameters(node),
            NodeKindData::FormalParameter(node) => self.emit_formal_parameter(node),
            NodeKindData::Class(node) => self.emit_class(node),
            NodeKindData::ClassBody(node) => self.emit_class_body(node),
            NodeKindData::MethodDefinition(node) => self.emit_method_definition(node),
            NodeKindData::PropertyDefinition(node) => self.emit_property_definition(node),
            NodeKindData::Decorator(node) => self.emit_decorator(node),
            NodeKindData::ImportDeclaration(node) => self.emit_import_declaration(node),
            NodeKindData::ImportSpecifier(node) => self.emit_import_specifier(node),
            NodeKindData::ImportDefaultSpecifier(node) => self.emit_import_default_specifier(node),
            NodeKindData::ImportNamespaceSpecifier(node) => {
                self.emit_import_namespace_specifier(node)
            }
            NodeKindData::ImportAttribute(node) => self.emit_import_attribute(node),
            NodeKindData::ImportExpression(node) => self.emit_import_expression(node),
            NodeKindData::ExportNamedDeclaration(node) => self.emit_export_named_declaration(node),
            NodeKindData::ExportDefaultDeclaration(node) => {
                self.emit_export_default_declaration(node)
            }
            NodeKindData::ExportAllDeclaration(node) => self.emit_export_all_declaration(node),
            NodeKindData::ExportSpecifier(node) => self.emit_export_specifier(node),
            NodeKindData::TsTypeAnnotation(node) => self.emit_ts_type_annotation(node),
            NodeKindData::TsTypeReference(node) => self.emit_ts_type_reference(node),
            NodeKindData::TsQualifiedName(node) => self.emit_ts_qualified_name(node),
            NodeKindData::TsTypeQuery(node) => self.emit_ts_type_query(node),
            NodeKindData::TsImportType(node) => self.emit_ts_import_type(node),
            NodeKindData::TsTypeParameter(node) => self.emit_ts_type_parameter(node),
            NodeKindData::TsTypeParameterDeclaration(node) => {
                self.emit_ts_type_parameter_declaration(node)
            }
            NodeKindData::TsTypeParameterInstantiation(node) => {
                self.emit_ts_type_parameter_instantiation(node)
            }
            NodeKindData::TsLiteralType(node) => self.emit_ts_literal_type(node),
            NodeKindData::TsTemplateLiteralType(node) => self.emit_ts_template_literal_type(node),
            NodeKindData::TsArrayType(node) => self.emit_ts_array_type(node),
            NodeKindData::TsIndexedAccessType(node) => self.emit_ts_indexed_access_type(node),
            NodeKindData::TsTupleType(node) => self.emit_ts_tuple_type(node),
            NodeKindData::TsNamedTupleMember(node) => self.emit_ts_named_tuple_member(node),
            NodeKindData::TsOptionalType(node) => self.emit_ts_optional_type(node),
            NodeKindData::TsRestType(node) => self.emit_ts_rest_type(node),
            NodeKindData::TsJsDocNullableType(node) => self.emit_ts_jsdoc_nullable_type(node),
            NodeKindData::TsJsDocNonNullableType(node) => {
                self.emit_ts_jsdoc_non_nullable_type(node)
            }
            NodeKindData::TsUnionType(node) => self.emit_ts_union_type(node),
            NodeKindData::TsIntersectionType(node) => self.emit_ts_intersection_type(node),
            NodeKindData::TsConditionalType(node) => self.emit_ts_conditional_type(node),
            NodeKindData::TsInferType(node) => self.emit_ts_infer_type(node),
            NodeKindData::TsTypeOperator(node) => self.emit_ts_type_operator(node),
            NodeKindData::TsParenthesizedType(node) => self.emit_ts_parenthesized_type(node),
            NodeKindData::TsFunctionType(node) => self.emit_ts_function_type(node),
            NodeKindData::TsConstructorType(node) => self.emit_ts_constructor_type(node),
            NodeKindData::TsTypePredicate(node) => self.emit_ts_type_predicate(node),
            NodeKindData::TsTypeLiteral(node) => self.emit_ts_type_literal(node),
            NodeKindData::TsMappedType(node) => self.emit_ts_mapped_type(node),
            NodeKindData::TsPropertySignature(node) => self.emit_ts_property_signature(node),
            NodeKindData::TsMethodSignature(node) => self.emit_ts_method_signature(node),
            NodeKindData::TsCallSignatureDeclaration(node) => {
                self.emit_ts_call_signature_declaration(node)
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                self.emit_ts_construct_signature_declaration(node)
            }
            NodeKindData::TsIndexSignature(node) => self.emit_ts_index_signature(node),
            NodeKindData::TsTypeAliasDeclaration(node) => self.emit_ts_type_alias_declaration(node),
            NodeKindData::TsInterfaceDeclaration(node) => self.emit_ts_interface_declaration(node),
            NodeKindData::TsInterfaceBody(node) => self.emit_ts_interface_body(node),
            NodeKindData::TsInterfaceHeritage(node) => self.emit_ts_interface_heritage(node),
            NodeKindData::TsClassImplements(node) => self.emit_ts_class_implements(node),
            NodeKindData::TsEnumDeclaration(node) => self.emit_ts_enum_declaration(node),
            NodeKindData::TsEnumBody(node) => self.emit_ts_enum_body(node),
            NodeKindData::TsEnumMember(node) => self.emit_ts_enum_member(node),
            NodeKindData::TsModuleDeclaration(node) => self.emit_ts_module_declaration(node),
            NodeKindData::TsModuleBlock(node) => self.emit_ts_module_block(node),
            NodeKindData::TsGlobalDeclaration(node) => self.emit_ts_global_declaration(node),
            NodeKindData::TsAsExpression(node) => self.emit_ts_as_expression(node, context),
            NodeKindData::TsSatisfiesExpression(node) => {
                self.emit_ts_satisfies_expression(node, context)
            }
            NodeKindData::TsTypeAssertion(node) => self.emit_ts_type_assertion(node),
            NodeKindData::TsNonNullExpression(node) => {
                self.emit_ts_non_null_expression(node, context)
            }
            NodeKindData::TsInstantiationExpression(node) => {
                self.emit_ts_instantiation_expression(node, context)
            }
            NodeKindData::TsExportAssignment(node) => self.emit_ts_export_assignment(node),
            NodeKindData::TsNamespaceExportDeclaration(node) => {
                self.emit_ts_namespace_export_declaration(node)
            }
            NodeKindData::TsImportEqualsDeclaration(node) => {
                self.emit_ts_import_equals_declaration(node)
            }
            NodeKindData::TsExternalModuleReference(node) => {
                self.emit_ts_external_module_reference(node)
            }
            NodeKindData::TsParameterProperty(node) => self.emit_ts_parameter_property(node),
            NodeKindData::TsThisParameter(node) => self.emit_ts_this_parameter(node),
            NodeKindData::JsxElement(node) => self.emit_jsx_element(node),
            NodeKindData::JsxOpeningElement(node) => self.emit_jsx_opening_element(node),
            NodeKindData::JsxClosingElement(node) => self.emit_jsx_closing_element(node),
            NodeKindData::JsxFragment(node) => self.emit_jsx_fragment(node),
            NodeKindData::JsxIdentifier(node) => self.emit_jsx_identifier(node),
            NodeKindData::JsxNamespacedName(node) => self.emit_jsx_namespaced_name(node),
            NodeKindData::JsxMemberExpression(node) => self.emit_jsx_member_expression(node),
            NodeKindData::JsxAttribute(node) => self.emit_jsx_attribute(node),
            NodeKindData::JsxSpreadAttribute(node) => self.emit_jsx_spread_attribute(node),
            NodeKindData::JsxExpressionContainer(node) => self.emit_jsx_expression_container(node),
            NodeKindData::JsxEmptyExpression(node) => self.emit_jsx_empty_expression(node),
            NodeKindData::JsxText(node) => self.emit_jsx_text(node),
            NodeKindData::JsxSpreadChild(node) => self.emit_jsx_spread_child(node),
            other => panic!("codegen: not implemented for {:?}", other),
        }
    }

    #[inline]
    fn allow_comment(&self, comment: AttachedComment) -> bool {
        match self.options.comments {
            Comments::None => false,
            Comments::All => true,
            Comments::Some => {
                comment.comment_type == CommentType::Block
                    && utils::is_significant_block_comment(self.tree.string(comment.value))
            }
            Comments::Line => comment.comment_type == CommentType::Line,
            Comments::Block => comment.comment_type == CommentType::Block,
        }
    }

    #[inline]
    fn last_byte(&self) -> u8 {
        self.code.last().copied().unwrap_or(0)
    }

    fn emit_leading_comments(&mut self, idx: NodeId) {
        if Some(idx) == self.skip_leading_of {
            return;
        }
        for &comment in self.tree.comments_of(idx) {
            if comment.position == AttachedCommentPosition::Before && self.allow_comment(comment) {
                self.write_leading(comment);
            }
        }
    }

    fn hoist_key_comments(&mut self, key: Option<NodeId>) {
        if self.options.comments == Comments::None {
            return;
        }
        let Some(key) = key else {
            return;
        };
        self.emit_leading_comments(key);
        self.skip_leading_of = Some(key);
    }

    fn emit_trailing_comments(&mut self, idx: NodeId) {
        for &comment in self.tree.comments_of(idx) {
            if comment.position == AttachedCommentPosition::After && self.allow_comment(comment) {
                self.flush_semi();
                self.write_trailing(comment);
            }
        }
    }

    fn write_leading(&mut self, comment: AttachedComment) {
        if comment.comment_type == CommentType::Block && comment.same_line {
            let last = self.last_byte();
            if self.pretty() && !matches!(last, 0 | b' ' | b'\n') {
                self.write_byte(b' ');
            }
            self.write_comment_body(comment);
            if self.pretty() {
                self.write_byte(b' ');
            }
            return;
        }
        self.break_line();
        self.write_comment_body(comment);
        self.break_line();
    }

    fn write_trailing(&mut self, comment: AttachedComment) {
        if comment.same_line {
            if self.pretty() {
                self.write_byte(b' ');
            }
            self.write_comment_body(comment);
            if comment.comment_type == CommentType::Line {
                self.break_line();
            }
            return;
        }
        self.break_line();
        self.write_comment_body(comment);
        self.break_line();
    }

    #[inline]
    fn write_comment_body(&mut self, comment: AttachedComment) {
        let value = self.tree.string(comment.value);
        self.write_str(if comment.comment_type == CommentType::Line {
            b"//"
        } else {
            b"/*"
        });
        if comment.comment_type == CommentType::Block {
            self.write_block_body(value);
            self.push_slice(b"*/");
            if let Some(sm) = &mut self.sm {
                sm.advance(b"*/");
            }
        } else {
            self.push_slice(value);
            if let Some(sm) = &mut self.sm {
                sm.advance(value);
            }
        }
    }

    fn write_block_body(&mut self, value: &[u8]) {
        if !self.pretty() || !utils::is_jsdoc_body(value) {
            self.push_slice(value);
            if let Some(sm) = &mut self.sm {
                sm.advance(value);
            }
            return;
        }
        let mut lines = value.split(|&byte| byte == b'\n');
        self.write_str(lines.next().unwrap_or_default());
        for line in lines {
            self.break_line();
            self.write_byte(b' ');
            self.write_str(line.trim_ascii_start());
        }
    }

    fn break_line(&mut self) {
        let mut index = self.code.len();
        while index > 0 && self.code[index - 1] == b' ' {
            index -= 1;
        }
        if index == 0 {
            return;
        }
        self.code.truncate(index);
        let count = if self.pretty() {
            self.indent_depth * u32::from(self.options.indent)
        } else {
            0
        } as usize;
        if self.code[self.code.len() - 1] != b'\n' {
            self.code.push(b'\n');
            if let Some(sm) = &mut self.sm {
                sm.gen_line += 1;
            }
        }
        self.code.resize(self.code.len() + count, b' ');
        if let Some(sm) = &mut self.sm {
            sm.gen_col = count as u32;
        }
    }

    fn record_mapping(&mut self, idx: NodeId) {
        let span = self.tree.span(idx);
        if span.is_dummy() {
            return;
        }
        let starts = self.tree.line_starts();
        let line = match starts.binary_search(&span.start) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };
        let mut col = 0;
        let mut cursor = starts[line] as usize;
        while cursor < span.start as usize {
            let byte = self.tree.source()[cursor];
            let len = if byte < 0x80 {
                1
            } else if (0xc2..=0xdf).contains(&byte) {
                2
            } else if (0xe0..=0xef).contains(&byte) {
                3
            } else if (0xf0..=0xf4).contains(&byte) {
                4
            } else {
                cursor += 1;
                continue;
            };
            col += if len == 4 { 2 } else { 1 };
            cursor += len;
        }
        if let Some(sm) = &mut self.sm {
            sm.line_cursor = line as u32;
            sm.record(self.allocator, line as u32, col);
        }
    }

    fn diagnose(&mut self, idx: NodeId, message: &'a [u8]) {
        let span = self.tree.span(idx);
        self.errors.push(Diagnostic {
            message,
            start: span.start,
            end: span.end,
        });
    }

    fn emit_program(&mut self, program: Program) {
        if let Some(hashbang) = program.hashbang(self.tree) {
            self.write_str(b"#!");
            self.write_string(hashbang.value(self.tree));
            self.write_byte(b'\n');
        }
        self.print_stmt_list(program.body(self.tree), true);
        self.pending_semi = false;
        if self.options.comments != Comments::None {
            self.emit_inside_comments(self.current_idx.expect("program is current"));
        }
    }

    fn print_stmt_list<T>(&mut self, items: TypedSubRange<T>, prologue: bool)
    where
        T: AstNode + ExtraDataCompact<'a>,
    {
        let mut first = true;
        let mut in_prologue = prologue;
        for item in items.iter() {
            let statement = self.tree.get_node_in_sub_range(item);
            let statement = statement.node_id();
            let cursor = self.cursor();
            let saved_semi = self.pending_semi;
            if !first {
                self.newline();
            }
            self.flush_semi();
            self.in_prologue = in_prologue;
            if self.try_emit(statement) {
                first = false;
                if prologue && self.tree.node_kind(statement) != NodeKind::Directive {
                    in_prologue = false;
                }
            } else {
                self.restore(cursor);
                self.pending_semi = saved_semi;
            }
        }
        self.in_prologue = false;
    }

    fn print_block<T>(&mut self, items: TypedSubRange<T>, prologue: bool)
    where
        T: AstNode + ExtraDataCompact<'a>,
    {
        self.write_byte(b'{');
        if !items.is_empty() {
            let cursor = self.cursor();
            self.indent_depth += 1;
            self.newline();
            let after_indent = self.mark();
            self.print_stmt_list(items, prologue);
            self.indent_depth -= 1;
            if self.mark() == after_indent {
                self.restore(cursor);
            } else {
                self.pending_semi = false;
                self.newline();
            }
        } else if self.options.comments != Comments::None {
            self.emit_inside_comments(self.current_idx.expect("container is current"));
        }
        self.write_byte(b'}');
    }

    fn emit_inside_comments(&mut self, idx: NodeId) {
        let mut any = false;
        for &comment in self.tree.comments_of(idx) {
            if comment.position != AttachedCommentPosition::Inside || !self.allow_comment(comment) {
                continue;
            }
            if !any {
                any = true;
                self.indent_depth += 1;
            }
            self.break_line();
            self.write_comment_body(comment);
        }
        if any {
            self.indent_depth -= 1;
            self.break_line();
        }
    }

    #[inline]
    fn soft_semi(&mut self) {
        if self.pretty() {
            self.write_byte(b';');
        } else {
            self.pending_semi = true;
        }
    }

    #[inline]
    fn flush_semi(&mut self) {
        if self.pending_semi {
            self.pending_semi = false;
            self.write_byte(b';');
        }
    }

    fn emit_block_statement(&mut self, statement: swc_experimental_ecma_ast::BlockStatement) {
        self.print_block(statement.body(self.tree), false);
    }

    fn emit_function_body(&mut self, body: swc_experimental_ecma_ast::FunctionBody) {
        self.print_block(body.body(self.tree), true);
    }

    fn emit_static_block(&mut self, block: swc_experimental_ecma_ast::StaticBlock) {
        self.write_str(b"static");
        self.space();
        self.print_block(block.body(self.tree), false);
    }

    fn emit_directive(&mut self, directive: swc_experimental_ecma_ast::Directive) {
        let expression = directive.expression(self.tree);
        self.write_string(expression.raw(self.tree));
        self.soft_semi();
    }

    fn emit_empty_statement(&mut self, _: swc_experimental_ecma_ast::EmptyStatement) {
        self.write_byte(b';');
    }

    fn emit_debugger_statement(&mut self, _: swc_experimental_ecma_ast::DebuggerStatement) {
        self.write_str(b"debugger");
        self.soft_semi();
    }

    fn emit_expression_statement(
        &mut self,
        statement: swc_experimental_ecma_ast::ExpressionStatement,
    ) {
        let expression = statement.expression(self.tree).node_id();
        let as_directive =
            self.in_prologue && self.tree.node_kind(expression) == NodeKind::StringLiteral;
        self.in_prologue = false;
        if as_directive {
            self.write_byte(b'(');
            self.emit(expression);
            self.write_byte(b')');
            self.soft_semi();
            return;
        }
        self.at_lead = Lead::Statement;
        self.emit_expr(expression, ExprContext::default());
        self.soft_semi();
    }

    fn emit_if_statement(&mut self, statement: swc_experimental_ecma_ast::IfStatement) {
        self.write_str(b"if");
        self.space();
        self.write_byte(b'(');
        self.emit(statement.test(self.tree).node_id());
        self.write_byte(b')');
        self.space();
        self.emit_stmt(statement.consequent(self.tree).node_id());
        if let Some(alternate) = statement.alternate(self.tree) {
            self.flush_semi();
            self.space();
            self.write_str(b"else ");
            self.emit_stmt(alternate.node_id());
        }
    }

    fn emit_return_statement(&mut self, statement: swc_experimental_ecma_ast::ReturnStatement) {
        self.write_str(b"return");
        self.emit_restricted_arg(
            statement.argument(self.tree).map(AstNode::node_id),
            Precedence::LOWEST,
        );
        self.soft_semi();
    }

    fn emit_throw_statement(&mut self, statement: swc_experimental_ecma_ast::ThrowStatement) {
        self.write_str(b"throw");
        self.emit_restricted_arg(
            Some(statement.argument(self.tree).node_id()),
            Precedence::LOWEST,
        );
        self.soft_semi();
    }

    fn emit_restricted_arg(&mut self, idx: Option<NodeId>, precedence: u8) {
        let Some(idx) = idx else {
            return;
        };
        let cursor = self.cursor();
        let at = self.mark();
        self.write_byte(b' ');
        self.emit_expr(
            idx,
            ExprContext {
                precedence,
                ..ExprContext::default()
            },
        );
        if self.code[at] == b'\n' {
            self.restore(cursor);
            self.write_str(b" (");
            self.emit_expr(
                idx,
                ExprContext {
                    precedence,
                    ..ExprContext::default()
                },
            );
            self.write_byte(b')');
        }
    }

    fn emit_break_statement(&mut self, statement: swc_experimental_ecma_ast::BreakStatement) {
        self.write_str(b"break");
        if let Some(label) = statement.label(self.tree) {
            self.write_byte(b' ');
            self.emit(label.node_id());
        }
        self.soft_semi();
    }

    fn emit_continue_statement(&mut self, statement: swc_experimental_ecma_ast::ContinueStatement) {
        self.write_str(b"continue");
        if let Some(label) = statement.label(self.tree) {
            self.write_byte(b' ');
            self.emit(label.node_id());
        }
        self.soft_semi();
    }

    fn emit_labeled_statement(&mut self, statement: swc_experimental_ecma_ast::LabeledStatement) {
        self.emit(statement.label(self.tree).node_id());
        self.write_byte(b':');
        self.space();
        self.emit_stmt(statement.body(self.tree).node_id());
    }

    fn emit_with_statement(&mut self, statement: swc_experimental_ecma_ast::WithStatement) {
        self.write_str(b"with");
        self.space();
        self.write_byte(b'(');
        self.emit(statement.object(self.tree).node_id());
        self.write_byte(b')');
        self.space();
        self.emit_stmt(statement.body(self.tree).node_id());
    }

    fn emit_while_statement(&mut self, statement: swc_experimental_ecma_ast::WhileStatement) {
        self.write_str(b"while");
        self.space();
        self.write_byte(b'(');
        self.emit(statement.test(self.tree).node_id());
        self.write_byte(b')');
        self.space();
        self.emit_stmt(statement.body(self.tree).node_id());
    }

    fn emit_do_while_statement(&mut self, statement: swc_experimental_ecma_ast::DoWhileStatement) {
        self.write_str(b"do ");
        self.emit_stmt(statement.body(self.tree).node_id());
        self.flush_semi();
        self.space();
        self.write_str(b"while");
        self.space();
        self.write_byte(b'(');
        self.emit(statement.test(self.tree).node_id());
        self.write_str(b");");
    }

    fn emit_for_statement(&mut self, statement: swc_experimental_ecma_ast::ForStatement) {
        self.write_str(b"for");
        self.space();
        self.write_byte(b'(');
        if let Some(init) = statement.init(self.tree) {
            let init = init.node_id();
            if let NodeKindData::VariableDeclaration(declaration) = self.node_data(init) {
                self.print_variable_decl(declaration, false, true);
            } else {
                self.emit_expr(
                    init,
                    ExprContext {
                        no_in: true,
                        ..ExprContext::default()
                    },
                );
            }
        }
        self.write_byte(b';');
        if let Some(test) = statement.test(self.tree) {
            self.space();
            self.emit(test.node_id());
        }
        self.write_byte(b';');
        if let Some(update) = statement.update(self.tree) {
            self.space();
            self.emit(update.node_id());
        }
        self.write_byte(b')');
        self.space();
        self.emit_stmt(statement.body(self.tree).node_id());
    }

    fn emit_for_in_statement(&mut self, statement: swc_experimental_ecma_ast::ForInStatement) {
        self.write_str(b"for");
        self.space();
        self.write_byte(b'(');
        self.print_for_left(statement.left(self.tree).node_id());
        self.write_str(b" in ");
        self.emit(statement.right(self.tree).node_id());
        self.write_byte(b')');
        self.space();
        self.emit_stmt(statement.body(self.tree).node_id());
    }

    fn emit_for_of_statement(&mut self, statement: swc_experimental_ecma_ast::ForOfStatement) {
        self.write_str(b"for");
        if statement.r#await(self.tree) {
            self.write_str(b" await");
        }
        self.space();
        self.write_byte(b'(');
        let left = statement.left(self.tree).node_id();
        let wrap_async = !statement.r#await(self.tree) && is_bare_async_identifier(self.tree, left);
        if wrap_async {
            self.write_byte(b'(');
        }
        self.print_for_left(left);
        if wrap_async {
            self.write_byte(b')');
        }
        self.write_str(b" of ");
        self.emit_value(statement.right(self.tree).node_id());
        self.write_byte(b')');
        self.space();
        self.emit_stmt(statement.body(self.tree).node_id());
    }

    fn print_for_left(&mut self, idx: NodeId) {
        if let NodeKindData::VariableDeclaration(declaration) = self.node_data(idx) {
            self.print_variable_decl(declaration, false, false);
        } else {
            self.emit_assign_target(idx);
        }
    }

    fn emit_switch_statement(&mut self, statement: swc_experimental_ecma_ast::SwitchStatement) {
        self.write_str(b"switch");
        self.space();
        self.write_byte(b'(');
        self.emit(statement.discriminant(self.tree).node_id());
        self.write_byte(b')');
        self.space();
        self.write_byte(b'{');
        let cases = statement.cases(self.tree);
        if !cases.is_empty() {
            for case in cases.iter() {
                self.flush_semi();
                self.newline();
                let case = self.tree.get_node_in_sub_range(case);
                self.emit(case.node_id());
            }
            self.pending_semi = false;
            self.newline();
        }
        self.write_byte(b'}');
    }

    fn emit_switch_case(&mut self, case: swc_experimental_ecma_ast::SwitchCase) {
        if let Some(test) = case.test(self.tree) {
            self.write_str(b"case ");
            self.emit(test.node_id());
            self.write_byte(b':');
        } else {
            self.write_str(b"default:");
        }
        let consequent = case.consequent(self.tree);
        if consequent.is_empty() {
            return;
        }
        self.indent_depth += 1;
        self.print_stmt_list(consequent, false);
        self.indent_depth -= 1;
    }

    fn emit_try_statement(&mut self, statement: swc_experimental_ecma_ast::TryStatement) {
        self.write_str(b"try ");
        self.emit(statement.block(self.tree).node_id());
        if let Some(handler) = statement.handler(self.tree) {
            self.space();
            self.emit(handler.node_id());
        }
        if let Some(finalizer) = statement.finalizer(self.tree) {
            self.space();
            self.write_str(b"finally ");
            self.emit(finalizer.node_id());
        }
    }

    fn emit_catch_clause(&mut self, clause: swc_experimental_ecma_ast::CatchClause) {
        self.write_str(b"catch");
        if let Some(param) = clause.param(self.tree) {
            self.space();
            self.write_byte(b'(');
            self.emit(param.node_id());
            self.write_byte(b')');
        }
        self.space();
        self.emit(clause.body(self.tree).node_id());
    }

    fn emit_variable_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::VariableDeclaration,
    ) {
        if self.config.strip_ts && declaration.declare(self.tree) {
            return;
        }
        self.print_variable_decl(declaration, true, false);
    }

    fn print_variable_decl(
        &mut self,
        declaration: swc_experimental_ecma_ast::VariableDeclaration,
        with_semicolon: bool,
        no_in: bool,
    ) {
        if !self.config.strip_ts && declaration.declare(self.tree) {
            self.write_str(b"declare ");
        }
        self.write_str(declaration.kind(self.tree).as_str().as_bytes());
        self.write_byte(b' ');
        let previous = self.declaration_no_in;
        self.declaration_no_in = no_in;
        self.emit_list(declaration.declarators(self.tree));
        self.declaration_no_in = previous;
        if with_semicolon {
            self.soft_semi();
        }
    }

    fn emit_variable_declarator(
        &mut self,
        declarator: swc_experimental_ecma_ast::VariableDeclarator,
    ) {
        if !self.config.strip_ts {
            self.definite_pending = declarator.definite(self.tree);
        }
        self.emit(declarator.id(self.tree).node_id());
        assert!(!self.definite_pending);
        if let Some(init) = declarator.init(self.tree) {
            self.separate_bang_from_assign();
            self.print_eq();
            self.emit_expr(
                init.node_id(),
                ExprContext {
                    precedence: Precedence::ASSIGNMENT,
                    no_in: self.declaration_no_in,
                    ..ExprContext::default()
                },
            );
        }
    }

    fn emit_sequence_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::SequenceExpression,
        context: ExprContext,
    ) {
        for (index, item) in expression.expressions(self.tree).iter().enumerate() {
            if index > 0 {
                self.comma();
            }
            let item = self.tree.get_node_in_sub_range(item);
            self.emit_expr(
                item.node_id(),
                ExprContext {
                    precedence: Precedence::ASSIGNMENT,
                    no_in: context.no_in,
                    ..ExprContext::default()
                },
            );
        }
    }

    fn emit_parenthesized_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::ParenthesizedExpression,
    ) {
        self.write_byte(b'(');
        self.emit(expression.expression(self.tree).node_id());
        self.write_byte(b')');
    }

    fn emit_binary_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::BinaryExpression,
        context: ExprContext,
    ) {
        use swc_experimental_ecma_ast::BinaryOperator as Op;
        let operator = match expression.operator(self.tree) {
            Op::Equal => b"==" as &[u8],
            Op::NotEqual => b"!=",
            Op::StrictEqual => b"===",
            Op::StrictNotEqual => b"!==",
            Op::LessThan => b"<",
            Op::LessThanOrEqual => b"<=",
            Op::GreaterThan => b">",
            Op::GreaterThanOrEqual => b">=",
            Op::Add => b"+",
            Op::Subtract => b"-",
            Op::Multiply => b"*",
            Op::Divide => b"/",
            Op::Modulo => b"%",
            Op::Exponent => b"**",
            Op::BitwiseOr => b"|",
            Op::BitwiseXor => b"^",
            Op::BitwiseAnd => b"&",
            Op::LeftShift => b"<<",
            Op::RightShift => b">>",
            Op::UnsignedRightShift => b">>>",
            Op::In => b"in",
            Op::Instanceof => b"instanceof",
        };
        let operation = expression.operator(self.tree);
        let precedence = binary_precedence(operation);
        let right_associative = operation == Op::Exponent;
        let mut left_minimum = if right_associative {
            Precedence::POSTFIX
        } else {
            precedence
        };
        let right_minimum = if right_associative {
            precedence
        } else {
            precedence + 1
        };
        let left = expression.left(self.tree).node_id();
        let right = expression.right(self.tree).node_id();
        if operator[0] == b'<' && ends_with_ts_cast(self.tree, left) {
            left_minimum = Precedence::GROUPING;
        }
        self.emit_expr(
            left,
            ExprContext {
                precedence: left_minimum,
                no_in: context.no_in,
                ..ExprContext::default()
            },
        );
        let right_context = ExprContext {
            precedence: right_minimum,
            no_in: context.no_in,
            ..ExprContext::default()
        };
        if utils::is_word_op(operator) {
            self.write_byte(b' ');
            self.write_str(operator);
            self.write_byte(b' ');
        } else {
            if operator[0] == b'=' {
                self.separate_bang_from_assign();
            }
            self.space();
            self.write_str(operator);
            self.space();
            if !self.pretty() && operator.len() == 1 && !self.needs_parens(right, right_context) {
                match operator[0] {
                    b'+' | b'-' | b'/' if leftmost_byte_is(self.tree, right, operator[0]) => {
                        self.write_byte(b' ');
                    }
                    b'<' if leftmost_byte_is(self.tree, right, b'!') => self.write_byte(b' '),
                    _ => {}
                }
            }
        }
        self.emit_expr(right, right_context);
    }

    fn emit_logical_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::LogicalExpression,
        context: ExprContext,
    ) {
        use swc_experimental_ecma_ast::LogicalOperator as Op;
        let operator = match expression.operator(self.tree) {
            Op::And => b"&&" as &[u8],
            Op::Or => b"||",
            Op::NullishCoalescing => b"??",
        };
        let operation = expression.operator(self.tree);
        let precedence = logical_precedence(operation);
        self.emit_logical_operand(
            expression.left(self.tree).node_id(),
            precedence,
            operation,
            context.no_in,
        );
        self.space();
        self.write_str(operator);
        self.space();
        self.emit_logical_operand(
            expression.right(self.tree).node_id(),
            precedence + 1,
            operation,
            context.no_in,
        );
    }

    fn emit_logical_operand(
        &mut self,
        child: NodeId,
        minimum: u8,
        parent: swc_experimental_ecma_ast::LogicalOperator,
        no_in: bool,
    ) {
        let precedence = if logical_mismatch(self.tree, parent, self.stripped(child)) {
            Precedence::GROUPING
        } else {
            minimum
        };
        self.emit_expr(
            child,
            ExprContext {
                precedence,
                no_in,
                ..ExprContext::default()
            },
        );
    }

    fn emit_conditional_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::ConditionalExpression,
        context: ExprContext,
    ) {
        self.emit_expr(
            expression.test(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::LOGICAL_OR,
                no_in: context.no_in,
                ..ExprContext::default()
            },
        );
        self.space();
        self.write_byte(b'?');
        self.space();
        self.emit_expr(
            expression.consequent(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::ASSIGNMENT,
                ..ExprContext::default()
            },
        );
        self.space();
        self.write_byte(b':');
        self.space();
        self.emit_expr(
            expression.alternate(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::ASSIGNMENT,
                no_in: context.no_in,
                ..ExprContext::default()
            },
        );
    }

    fn emit_unary_expression(&mut self, expression: swc_experimental_ecma_ast::UnaryExpression) {
        use swc_experimental_ecma_ast::UnaryOperator as Op;
        let operator = match expression.operator(self.tree) {
            Op::Negate => b"-" as &[u8],
            Op::Positive => b"+",
            Op::LogicalNot => b"!",
            Op::BitwiseNot => b"~",
            Op::Typeof => b"typeof",
            Op::Void => b"void",
            Op::Delete => b"delete",
        };
        self.write_str(operator);
        if utils::is_word_op(operator) {
            self.write_byte(b' ');
        }
        let argument = expression.argument(self.tree).node_id();
        if operator.len() == 1
            && matches!(operator[0], b'+' | b'-')
            && leftmost_byte_is(self.tree, argument, operator[0])
        {
            self.write_byte(b' ');
        }
        self.emit_expr(
            argument,
            ExprContext {
                precedence: Precedence::UNARY,
                ..ExprContext::default()
            },
        );
    }

    fn emit_update_expression(&mut self, expression: swc_experimental_ecma_ast::UpdateExpression) {
        use swc_experimental_ecma_ast::UpdateOperator as Op;
        let operator = match expression.operator(self.tree) {
            Op::Increment => b"++" as &[u8],
            Op::Decrement => b"--",
        };
        let argument = expression.argument(self.tree).node_id();
        if expression.prefix(self.tree) {
            self.write_str(operator);
            self.emit_assign_target(argument);
        } else {
            self.emit_assign_target(argument);
            self.write_str(operator);
        }
    }

    fn emit_assignment_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::AssignmentExpression,
        context: ExprContext,
    ) {
        use swc_experimental_ecma_ast::AssignmentOperator as Op;
        let operator = match expression.operator(self.tree) {
            Op::Assign => b"=" as &[u8],
            Op::AddAssign => b"+=",
            Op::SubtractAssign => b"-=",
            Op::MultiplyAssign => b"*=",
            Op::DivideAssign => b"/=",
            Op::ModuloAssign => b"%=",
            Op::ExponentAssign => b"**=",
            Op::LeftShiftAssign => b"<<=",
            Op::RightShiftAssign => b">>=",
            Op::UnsignedRightShiftAssign => b">>>=",
            Op::BitwiseOrAssign => b"|=",
            Op::BitwiseXorAssign => b"^=",
            Op::BitwiseAndAssign => b"&=",
            Op::LogicalOrAssign => b"||=",
            Op::LogicalAndAssign => b"&&=",
            Op::NullishAssign => b"??=",
        };
        self.emit_assign_target(expression.left(self.tree).node_id());
        self.separate_bang_from_assign();
        self.space();
        self.write_str(operator);
        self.space();
        self.emit_expr(
            expression.right(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::ASSIGNMENT,
                no_in: context.no_in,
                ..ExprContext::default()
            },
        );
    }

    #[inline]
    fn separate_bang_from_assign(&mut self) {
        if !self.pretty() && self.last_byte() == b'!' {
            self.write_byte(b' ');
        }
    }

    fn emit_assign_target(&mut self, idx: NodeId) {
        let previous = self.in_assign_target;
        if needs_parens_as_assign_target(self.tree, idx) {
            self.in_assign_target = false;
            self.write_byte(b'(');
            self.emit(idx);
            self.write_byte(b')');
        } else {
            self.in_assign_target = true;
            self.emit(idx);
        }
        self.in_assign_target = previous;
    }

    #[inline]
    fn emit_child_of_assign_target(&mut self, idx: NodeId) {
        if self.in_assign_target {
            self.emit_assign_target(idx);
        } else {
            self.emit_value(idx);
        }
    }

    #[inline]
    fn emit_value(&mut self, idx: NodeId) {
        self.emit_expr(
            idx,
            ExprContext {
                precedence: Precedence::ASSIGNMENT,
                ..ExprContext::default()
            },
        );
    }

    fn emit_array_expression(&mut self, expression: swc_experimental_ecma_ast::ArrayExpression) {
        let in_target = self.in_assign_target;
        self.in_assign_target = false;
        self.write_byte(b'[');
        let elements = expression.elements(self.tree);
        for (index, element) in elements.iter().enumerate() {
            if index > 0 {
                self.comma();
            }
            if let Some(element) = self.tree.get_node_in_sub_range(element) {
                if in_target {
                    self.emit_assign_target(element.node_id());
                } else {
                    self.emit_value(element.node_id());
                }
            }
        }
        if !elements.is_empty()
            && elements
                .last()
                .is_some_and(|last| self.tree.get_node_in_sub_range(last).is_none())
        {
            self.write_byte(b',');
        }
        self.write_byte(b']');
        self.in_assign_target = in_target;
    }

    fn emit_object_expression(&mut self, expression: swc_experimental_ecma_ast::ObjectExpression) {
        let in_target = self.in_assign_target;
        self.write_byte(b'{');
        let properties = expression.properties(self.tree);
        if !properties.is_empty() {
            self.space();
            for (index, property) in properties.iter().enumerate() {
                if index > 0 {
                    self.comma();
                }
                self.in_assign_target = in_target;
                let property = self.tree.get_node_in_sub_range(property);
                self.emit(property.node_id());
            }
            self.space();
        }
        self.write_byte(b'}');
        self.in_assign_target = in_target;
    }

    fn emit_object_property(&mut self, property: swc_experimental_ecma_ast::ObjectProperty) {
        use swc_experimental_ecma_ast::PropertyKind;
        let value = property.value(self.tree).node_id();
        if property.method(self.tree) || property.kind(self.tree) != PropertyKind::Init {
            let NodeKindData::Function(function) = self.node_data(value) else {
                unreachable!("method property value must be a function");
            };
            match property.kind(self.tree) {
                PropertyKind::Get => self.write_str(b"get "),
                PropertyKind::Set => self.write_str(b"set "),
                PropertyKind::Init => {
                    if function.r#async(self.tree) {
                        self.write_str(b"async ");
                    }
                    if function.generator(self.tree) {
                        self.write_byte(b'*');
                    }
                }
            }
            self.print_object_key(
                property.key(self.tree).node_id(),
                property.computed(self.tree),
            );
            self.print_function_as_method(function);
            return;
        }
        let key = property.key(self.tree).node_id();
        if property.shorthand(self.tree) && shorthand_still_valid(self.tree, key, value) {
            self.emit_child_of_assign_target(value);
            return;
        }
        self.print_object_key(key, property.computed(self.tree));
        self.write_byte(b':');
        self.space();
        self.emit_child_of_assign_target(value);
    }

    fn emit_spread_element(&mut self, spread: swc_experimental_ecma_ast::SpreadElement) {
        self.write_str(b"...");
        self.emit_value(spread.argument(self.tree).node_id());
    }

    fn emit_member_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::MemberExpression,
        context: ExprContext,
    ) {
        let object = expression.object(self.tree).node_id();
        let property = expression.property(self.tree).node_id();
        let head_start = self.mark();
        self.emit_expr(
            object,
            ExprContext {
                precedence: Precedence::CALL,
                no_call: context.no_call,
                ..ExprContext::default()
            },
        );
        let static_key = if self.config.minify && expression.computed(self.tree) {
            simple_string_key(self.tree, property)
        } else {
            None
        };
        if expression.computed(self.tree) && static_key.is_none() {
            if expression.optional(self.tree) {
                self.write_str(b"?.");
            }
            self.write_byte(b'[');
            self.emit(property);
            self.write_byte(b']');
        } else {
            if !expression.optional(self.tree) && is_bare_integer_head(&self.code[head_start..]) {
                self.write_byte(if self.pretty() { b' ' } else { b'.' });
            }
            self.write_str(if expression.optional(self.tree) {
                b"?."
            } else {
                b"."
            });
            if let Some(key) = static_key {
                self.write_str(key);
            } else {
                self.emit(property);
            }
        }
    }

    fn emit_call_expression(&mut self, expression: swc_experimental_ecma_ast::CallExpression) {
        self.emit_expr(
            expression.callee(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::CALL,
                ..ExprContext::default()
            },
        );
        if expression.optional(self.tree) {
            self.write_str(b"?.");
        }
        if let Some(arguments) = expression.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
        self.print_arg_list(expression.arguments(self.tree));
    }

    fn emit_chain_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::ChainExpression,
        context: ExprContext,
    ) {
        self.emit_expr(expression.expression(self.tree).node_id(), context);
    }

    fn emit_new_expression(&mut self, expression: swc_experimental_ecma_ast::NewExpression) {
        self.write_str(b"new ");
        self.emit_expr(
            expression.callee(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::NEW,
                no_call: true,
                ..ExprContext::default()
            },
        );
        if let Some(arguments) = expression.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
        self.print_arg_list(expression.arguments(self.tree));
    }

    fn emit_tagged_template_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::TaggedTemplateExpression,
        context: ExprContext,
    ) {
        self.emit_expr(
            expression.tag(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::CALL,
                no_call: context.no_call,
                ..ExprContext::default()
            },
        );
        if let Some(arguments) = expression.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
        self.emit(expression.quasi(self.tree).node_id());
    }

    fn emit_await_expression(&mut self, expression: swc_experimental_ecma_ast::AwaitExpression) {
        self.write_str(b"await ");
        self.emit_expr(
            expression.argument(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::UNARY,
                ..ExprContext::default()
            },
        );
    }

    fn emit_yield_expression(&mut self, expression: swc_experimental_ecma_ast::YieldExpression) {
        self.write_str(b"yield");
        if expression.delegate(self.tree) {
            self.write_byte(b'*');
        }
        self.emit_restricted_arg(
            expression.argument(self.tree).map(AstNode::node_id),
            Precedence::ASSIGNMENT,
        );
    }

    fn emit_meta_property(&mut self, property: swc_experimental_ecma_ast::MetaProperty) {
        self.emit(property.meta(self.tree).node_id());
        self.write_byte(b'.');
        self.emit(property.property(self.tree).node_id());
    }

    fn print_arg_list(&mut self, arguments: TypedSubRange<swc_experimental_ecma_ast::Argument>) {
        self.write_byte(b'(');
        for (index, argument) in arguments.iter().enumerate() {
            if index > 0 {
                self.comma();
            }
            let argument = self.tree.get_node_in_sub_range(argument);
            self.emit_value(argument.node_id());
        }
        self.write_byte(b')');
    }

    fn emit_string_literal(&mut self, literal: swc_experimental_ecma_ast::StringLiteral) {
        let raw = self.tree.string(literal.raw(self.tree));
        self.emit_quoted(
            self.tree.string(literal.value(self.tree)),
            raw.first() == Some(&b'\''),
        );
    }

    fn emit_quoted(&mut self, value: &[u8], single_quoted: bool) {
        let quote = self.pick_quote(value, single_quoted);
        self.write_byte(quote);
        self.write_escaped_string(value, quote);
        self.write_byte(quote);
    }

    #[inline]
    fn pick_quote(&self, value: &[u8], single_quoted: bool) -> u8 {
        if self.config.minify {
            let single = value.iter().filter(|&&byte| byte == b'\'').count();
            let double = value.iter().filter(|&&byte| byte == b'"').count();
            if single == double {
                return if self.options.quotes == Quotes::Single {
                    b'\''
                } else {
                    b'"'
                };
            }
            return if double < single { b'"' } else { b'\'' };
        }
        match self.options.quotes {
            Quotes::Preserve if single_quoted => b'\'',
            Quotes::Preserve | Quotes::Double => b'"',
            Quotes::Single => b'\'',
        }
    }

    fn write_escaped_string(&mut self, value: &[u8], quote: u8) {
        let mut start = 0;
        let mut index = 0;
        while index < value.len() {
            let byte = value[index];
            if byte >= 0x80 {
                if byte == 0xed && index + 2 < value.len() {
                    let second = value[index + 1];
                    let third = value[index + 2];
                    if (0xa0..=0xbf).contains(&second) && (0x80..=0xbf).contains(&third) {
                        let code_point = (u32::from(byte & 0x0f) << 12)
                            | (u32::from(second & 0x3f) << 6)
                            | u32::from(third & 0x3f);
                        if index > start {
                            self.write_str(&value[start..index]);
                        }
                        self.write_unicode_escape(code_point);
                        index += 3;
                        start = index;
                        continue;
                    }
                }
                index += 1;
                continue;
            }
            let escape = if self.config.minify {
                utils::script_escape(value, index)
            } else {
                None
            }
            .or_else(|| match byte {
                b'\\' => Some(b"\\\\" as &[u8]),
                b'\n' => Some(b"\\n"),
                b'\r' => Some(b"\\r"),
                b'\t' => Some(b"\\t"),
                0x08 => Some(b"\\b"),
                0x0c => Some(b"\\f"),
                0x0b => Some(b"\\v"),
                0 if value.get(index + 1).is_some_and(u8::is_ascii_digit) => Some(b"\\x00"),
                0 => Some(b"\\0"),
                _ if byte == quote && quote == b'"' => Some(b"\\\""),
                _ if byte == quote => Some(b"\\'"),
                _ => None,
            });
            if let Some(escape) = escape {
                if index > start {
                    self.write_str(&value[start..index]);
                }
                self.write_str(escape);
                start = index + 1;
            }
            index += 1;
        }
        if start < value.len() {
            self.write_str(&value[start..]);
        }
    }

    fn write_unicode_escape(&mut self, code_point: u32) {
        let hex = b"0123456789abcdef";
        let buffer = [
            b'\\',
            b'u',
            hex[((code_point >> 12) & 0xf) as usize],
            hex[((code_point >> 8) & 0xf) as usize],
            hex[((code_point >> 4) & 0xf) as usize],
            hex[(code_point & 0xf) as usize],
        ];
        self.write_str(&buffer);
    }

    fn emit_numeric_literal(&mut self, literal: swc_experimental_ecma_ast::NumericLiteral) {
        if self.config.minify {
            self.write_shortest_number(literal);
        } else {
            self.write_string(literal.raw(self.tree));
        }
    }

    fn write_shortest_number(&mut self, literal: swc_experimental_ecma_ast::NumericLiteral) {
        let raw = self.tree.string(literal.raw(self.tree));
        let mut source = [0; 128];
        let Some(cleaned) = utils::strip_underscores(raw, &mut source) else {
            self.write_str(raw);
            return;
        };
        if literal.kind(self.tree) != swc_experimental_ecma_ast::NumericLiteralKind::Decimal {
            self.write_str(cleaned);
            return;
        }
        let mut destination = [0; 128];
        self.write_str(utils::shortest_decimal(cleaned, &mut destination));
    }

    fn emit_bigint_literal(&mut self, literal: swc_experimental_ecma_ast::BigIntLiteral) {
        self.write_string(literal.raw(self.tree));
        self.write_byte(b'n');
    }

    fn emit_boolean_literal(&mut self, literal: swc_experimental_ecma_ast::BooleanLiteral) {
        if self.config.minify {
            self.write_str(if literal.value(self.tree) {
                b"!0"
            } else {
                b"!1"
            });
        } else {
            self.write_str(if literal.value(self.tree) {
                b"true"
            } else {
                b"false"
            });
        }
    }

    fn emit_regexp_literal(&mut self, literal: swc_experimental_ecma_ast::RegExpLiteral) {
        self.write_byte(b'/');
        self.write_string(literal.pattern(self.tree));
        self.write_byte(b'/');
        self.write_string(literal.flags(self.tree));
    }

    fn emit_template_literal(&mut self, literal: swc_experimental_ecma_ast::TemplateLiteral) {
        self.print_template(literal.quasis(self.tree), literal.expressions(self.tree));
    }

    fn print_template<Q, S>(&mut self, quasis: TypedSubRange<Q>, substitutions: TypedSubRange<S>)
    where
        Q: AstNode + ExtraDataCompact<'a>,
        S: AstNode + ExtraDataCompact<'a>,
    {
        self.write_byte(b'`');
        for (index, quasi) in quasis.iter().enumerate() {
            let quasi = self.tree.get_node_in_sub_range(quasi);
            self.emit(quasi.node_id());
            if let Some(substitution) = substitutions.get(index) {
                self.write_str(b"${");
                let substitution = self.tree.get_node_in_sub_range(substitution);
                self.emit(substitution.node_id());
                self.write_byte(b'}');
            }
        }
        self.write_byte(b'`');
    }

    fn emit_template_element(&mut self, element: swc_experimental_ecma_ast::TemplateElement) {
        if !self.config.minify {
            let raw = self.tree.string(element.raw(self.tree));
            if !raw.is_empty() {
                self.write_str(raw);
                return;
            }
        }
        let value = self.tree.string(element.cooked(self.tree));
        let mut index = 0;
        let mut start = 0;
        while index < value.len() {
            let byte = value[index];
            if byte == 0xed && index + 2 < value.len() {
                let second = value[index + 1];
                let third = value[index + 2];
                if (0xa0..=0xbf).contains(&second) && (0x80..=0xbf).contains(&third) {
                    let code_point = (u32::from(byte & 0x0f) << 12)
                        | (u32::from(second & 0x3f) << 6)
                        | u32::from(third & 0x3f);
                    if index > start {
                        self.write_str(&value[start..index]);
                    }
                    self.write_unicode_escape(code_point);
                    index += 3;
                    start = index;
                    continue;
                }
            }
            let escape = if self.config.minify {
                utils::script_escape(value, index)
            } else {
                None
            }
            .or_else(|| match byte {
                b'\\' => Some(b"\\\\" as &[u8]),
                b'`' => Some(b"\\`"),
                b'$' if value.get(index + 1) == Some(&b'{') => Some(b"\\$"),
                b'\r' => Some(b"\\r"),
                0 if value.get(index + 1).is_some_and(u8::is_ascii_digit) => Some(b"\\x00"),
                0 => Some(b"\\0"),
                _ => None,
            });
            if let Some(escape) = escape {
                if index > start {
                    self.write_str(&value[start..index]);
                }
                self.write_str(escape);
                start = index + 1;
            }
            index += 1;
        }
        if start < value.len() {
            self.write_str(&value[start..]);
        }
    }

    fn emit_identifier_reference(
        &mut self,
        identifier: swc_experimental_ecma_ast::IdentifierReference,
    ) {
        if self.config.minify && !self.in_assign_target {
            match self.tree.string(identifier.name(self.tree)) {
                b"undefined" => return self.write_str(b"void 0"),
                b"Infinity" => return self.write_str(b"1/0"),
                _ => {}
            }
        }
        self.write_string(identifier.name(self.tree));
    }

    fn emit_identifier_name(&mut self, identifier: swc_experimental_ecma_ast::IdentifierName) {
        self.write_string(identifier.name(self.tree));
    }

    fn emit_binding_identifier(
        &mut self,
        identifier: swc_experimental_ecma_ast::BindingIdentifier,
    ) {
        let definite = self.take_definite();
        if !self.config.strip_ts {
            self.print_decorators(identifier.decorators(self.tree));
        }
        self.write_string(identifier.name(self.tree));
        self.print_binding_suffix(
            identifier.optional(self.tree),
            definite,
            identifier.type_annotation(self.tree).map(AstNode::node_id),
        );
    }

    fn emit_label_identifier(&mut self, identifier: swc_experimental_ecma_ast::LabelIdentifier) {
        self.write_string(identifier.name(self.tree));
    }

    fn emit_private_identifier(
        &mut self,
        identifier: swc_experimental_ecma_ast::PrivateIdentifier,
    ) {
        self.write_byte(b'#');
        self.write_string(identifier.name(self.tree));
    }

    fn emit_assignment_pattern(&mut self, pattern: swc_experimental_ecma_ast::AssignmentPattern) {
        if !self.config.strip_ts {
            self.print_decorators(pattern.decorators(self.tree));
        }
        self.emit(pattern.left(self.tree).node_id());
        if !self.config.strip_ts && pattern.optional(self.tree) {
            self.write_byte(b'?');
        }
        if let Some(annotation) = pattern.type_annotation(self.tree) {
            self.emit(annotation.node_id());
        }
        self.separate_bang_from_assign();
        self.print_eq();
        self.emit_value(pattern.right(self.tree).node_id());
    }

    fn emit_binding_rest_element(&mut self, rest: swc_experimental_ecma_ast::BindingRestElement) {
        if !self.config.strip_ts {
            self.print_decorators(rest.decorators(self.tree));
        }
        self.write_str(b"...");
        self.emit(rest.argument(self.tree).node_id());
        if !self.config.strip_ts && rest.optional(self.tree) {
            self.write_byte(b'?');
        }
        if let Some(annotation) = rest.type_annotation(self.tree) {
            self.emit(annotation.node_id());
        }
    }

    fn emit_array_pattern(&mut self, pattern: swc_experimental_ecma_ast::ArrayPattern) {
        let definite = self.take_definite();
        if !self.config.strip_ts {
            self.print_decorators(pattern.decorators(self.tree));
        }
        self.write_byte(b'[');
        let elements = pattern.elements(self.tree);
        for (index, element) in elements.iter().enumerate() {
            if index > 0 {
                self.comma();
            }
            if let Some(element) = self.tree.get_node_in_sub_range(element) {
                self.emit_assign_target(element.node_id());
            }
        }
        if let Some(rest) = pattern.rest(self.tree) {
            if !elements.is_empty() {
                self.comma();
            }
            self.emit(rest.node_id());
        } else if !elements.is_empty()
            && elements
                .last()
                .is_some_and(|last| self.tree.get_node_in_sub_range(last).is_none())
        {
            self.write_byte(b',');
        }
        self.write_byte(b']');
        self.print_binding_suffix(
            pattern.optional(self.tree),
            definite,
            pattern.type_annotation(self.tree).map(AstNode::node_id),
        );
    }

    fn emit_object_pattern(&mut self, pattern: swc_experimental_ecma_ast::ObjectPattern) {
        let definite = self.take_definite();
        if !self.config.strip_ts {
            self.print_decorators(pattern.decorators(self.tree));
        }
        self.write_byte(b'{');
        let properties = pattern.properties(self.tree);
        let has_any = !properties.is_empty() || pattern.rest(self.tree).is_some();
        if has_any {
            self.space();
        }
        for (index, property) in properties.iter().enumerate() {
            if index > 0 {
                self.comma();
            }
            let property = self.tree.get_node_in_sub_range(property);
            self.emit(property.node_id());
        }
        if let Some(rest) = pattern.rest(self.tree) {
            if !properties.is_empty() {
                self.comma();
            }
            self.emit(rest.node_id());
        }
        if has_any {
            self.space();
        }
        self.write_byte(b'}');
        self.print_binding_suffix(
            pattern.optional(self.tree),
            definite,
            pattern.type_annotation(self.tree).map(AstNode::node_id),
        );
    }

    fn emit_binding_property(&mut self, property: swc_experimental_ecma_ast::BindingProperty) {
        let key = property.key(self.tree).node_id();
        let value = property.value(self.tree).node_id();
        if property.shorthand(self.tree) && shorthand_still_valid(self.tree, key, value) {
            self.emit_assign_target(value);
            return;
        }
        self.print_object_key(key, property.computed(self.tree));
        self.write_byte(b':');
        self.space();
        self.emit_assign_target(value);
    }

    fn print_property_key(&mut self, key: NodeId, computed: bool) {
        if computed {
            self.write_byte(b'[');
            self.emit_value(key);
            self.write_byte(b']');
        } else {
            self.emit(key);
        }
    }

    fn print_object_key(&mut self, key: NodeId, computed: bool) {
        if self.config.minify
            && let Some(value) = simple_string_key(self.tree, key)
        {
            self.write_str(value);
            return;
        }
        self.print_property_key(key, computed);
    }

    fn print_class_key(&mut self, key: NodeId, computed: bool, r#static: bool, is_field: bool) {
        if self.config.minify
            && let Some(value) = simple_string_key(self.tree, key)
        {
            if !computed {
                self.write_str(value);
                return;
            }
            let constructor_clash = value == b"constructor" && (is_field || !r#static);
            let prototype_clash = r#static && value == b"prototype";
            if !constructor_clash && !prototype_clash {
                self.write_str(value);
                return;
            }
        }
        self.print_property_key(key, computed);
    }

    fn emit_function(&mut self, function: swc_experimental_ecma_ast::Function) {
        use swc_experimental_ecma_ast::FunctionType;
        if self.config.strip_ts
            && (function.declare(self.tree)
                || matches!(
                    function.function_type(self.tree),
                    FunctionType::TsDeclareFunction | FunctionType::TsEmptyBodyFunctionExpression
                ))
        {
            return;
        }
        if !self.config.strip_ts && function.declare(self.tree) {
            self.write_str(b"declare ");
        }
        if function.r#async(self.tree) {
            self.write_str(b"async ");
        }
        self.write_str(b"function");
        if function.generator(self.tree) {
            self.write_byte(b'*');
        }
        if let Some(identifier) = function.id(self.tree) {
            self.write_byte(b' ');
            self.emit(identifier.node_id());
        }
        self.print_function_as_method(function);
    }

    fn print_function_as_method(&mut self, function: swc_experimental_ecma_ast::Function) {
        if let Some(parameters) = function.type_parameters(self.tree) {
            self.emit(parameters.node_id());
        }
        self.emit(function.params(self.tree).node_id());
        if let Some(return_type) = function.return_type(self.tree) {
            self.emit(return_type.node_id());
        }
        if let Some(body) = function.body(self.tree) {
            self.space();
            self.emit(body.node_id());
        } else if !self.config.strip_ts {
            self.soft_semi();
        }
    }

    fn emit_arrow_function_expression(
        &mut self,
        arrow: swc_experimental_ecma_ast::ArrowFunctionExpression,
    ) {
        if arrow.r#async(self.tree) {
            self.write_str(b"async ");
        }
        if let Some(parameters) = arrow.type_parameters(self.tree) {
            self.emit(parameters.node_id());
        }
        self.emit(arrow.params(self.tree).node_id());
        if let Some(return_type) = arrow.return_type(self.tree) {
            self.emit(return_type.node_id());
        }
        self.space();
        self.write_str(b"=>");
        self.space();
        let body = arrow.body(self.tree).node_id();
        if arrow.expression(self.tree) {
            self.at_lead = Lead::Arrow;
            self.emit_expr(
                body,
                ExprContext {
                    precedence: Precedence::ASSIGNMENT,
                    ..ExprContext::default()
                },
            );
        } else {
            self.emit(body);
        }
    }

    fn emit_formal_parameters(&mut self, parameters: swc_experimental_ecma_ast::FormalParameters) {
        self.write_byte(b'(');
        let mut first = true;
        for parameter in parameters.items(self.tree).iter() {
            let parameter = self.tree.get_node_in_sub_range(parameter);
            first = self.emit_separated(parameter.node_id(), first);
        }
        if let Some(rest) = parameters.rest(self.tree) {
            let _ = self.emit_separated(rest.node_id(), first);
        }
        self.write_byte(b')');
    }

    fn emit_separated(&mut self, idx: NodeId, first: bool) -> bool {
        let cursor = self.cursor();
        if !first {
            self.write_byte(b',');
            self.space();
        }
        if self.try_emit(idx) {
            return false;
        }
        self.restore(cursor);
        first
    }

    fn emit_formal_parameter(&mut self, parameter: swc_experimental_ecma_ast::FormalParameter) {
        self.emit(parameter.pattern(self.tree).node_id());
    }

    fn emit_class(&mut self, class: swc_experimental_ecma_ast::Class) {
        if self.config.strip_ts && class.declare(self.tree) {
            return;
        }
        self.print_decorators(class.decorators(self.tree));
        if !self.config.strip_ts {
            if class.declare(self.tree) {
                self.write_str(b"declare ");
            }
            if class.r#abstract(self.tree) {
                self.write_str(b"abstract ");
            }
        }
        self.write_str(b"class");
        if let Some(identifier) = class.id(self.tree) {
            self.write_byte(b' ');
            self.emit(identifier.node_id());
        }
        if let Some(parameters) = class.type_parameters(self.tree) {
            self.emit(parameters.node_id());
        }
        if let Some(super_class) = class.super_class(self.tree) {
            self.write_str(b" extends ");
            self.emit_expr(
                super_class.node_id(),
                ExprContext {
                    precedence: Precedence::CALL,
                    ..ExprContext::default()
                },
            );
            if let Some(arguments) = class.super_type_arguments(self.tree) {
                self.emit(arguments.node_id());
            }
        }
        if !self.config.strip_ts && !class.implements(self.tree).is_empty() {
            self.write_str(b" implements ");
            self.emit_list(class.implements(self.tree));
        }
        self.space();
        self.emit(class.body(self.tree).node_id());
    }

    fn emit_class_body(&mut self, body: swc_experimental_ecma_ast::ClassBody) {
        self.write_byte(b'{');
        self.indent_depth += 1;
        let mut any = false;
        for member in body.body(self.tree).iter() {
            let member = self.tree.get_node_in_sub_range(member);
            let cursor = self.cursor();
            let saved_semi = self.pending_semi;
            self.flush_semi();
            self.newline();
            if self.try_emit(member.node_id()) {
                any = true;
            } else {
                self.restore(cursor);
                self.pending_semi = saved_semi;
            }
        }
        self.indent_depth -= 1;
        if any {
            self.pending_semi = false;
            self.newline();
        } else if self.options.comments != Comments::None {
            self.emit_inside_comments(self.current_idx.expect("class body is current"));
        }
        self.write_byte(b'}');
    }

    fn emit_method_definition(&mut self, method: swc_experimental_ecma_ast::MethodDefinition) {
        use swc_experimental_ecma_ast::{Accessibility, MethodDefinitionKind};
        let NodeKindData::Function(function) = self.node_data(method.value(self.tree).node_id())
        else {
            unreachable!("method value must be a function");
        };
        if self.config.strip_ts
            && (method.r#abstract(self.tree) || function.body(self.tree).is_none())
        {
            return;
        }
        self.print_decorators(method.decorators(self.tree));
        let key = method.key(self.tree).node_id();
        self.hoist_key_comments(Some(key));
        if !self.config.strip_ts && method.accessibility(self.tree) != Accessibility::None {
            self.write_str(match method.accessibility(self.tree) {
                Accessibility::Public => b"public",
                Accessibility::Private => b"private",
                Accessibility::Protected => b"protected",
                Accessibility::None => unreachable!(),
            });
            self.write_byte(b' ');
        }
        if method.r#static(self.tree) {
            self.write_str(b"static ");
        }
        if !self.config.strip_ts {
            if method.r#abstract(self.tree) {
                self.write_str(b"abstract ");
            }
            if method.r#override(self.tree) {
                self.write_str(b"override ");
            }
        }
        match method.kind(self.tree) {
            MethodDefinitionKind::Get => self.write_str(b"get "),
            MethodDefinitionKind::Set => self.write_str(b"set "),
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {
                if function.r#async(self.tree) {
                    self.write_str(b"async ");
                }
                if function.generator(self.tree) {
                    self.write_byte(b'*');
                }
            }
        }
        self.print_class_key(
            key,
            method.computed(self.tree),
            method.r#static(self.tree),
            false,
        );
        if !self.config.strip_ts && method.optional(self.tree) {
            self.write_byte(b'?');
        }
        self.print_function_as_method(function);
        self.skip_leading_of = None;
    }

    fn emit_property_definition(
        &mut self,
        property: swc_experimental_ecma_ast::PropertyDefinition,
    ) {
        use swc_experimental_ecma_ast::Accessibility;
        if self.config.strip_ts && (property.declare(self.tree) || property.r#abstract(self.tree)) {
            return;
        }
        self.print_decorators(property.decorators(self.tree));
        let key = property.key(self.tree).node_id();
        self.hoist_key_comments(Some(key));
        if !self.config.strip_ts {
            if property.declare(self.tree) {
                self.write_str(b"declare ");
            }
            if property.accessibility(self.tree) != Accessibility::None {
                self.write_str(match property.accessibility(self.tree) {
                    Accessibility::Public => b"public",
                    Accessibility::Private => b"private",
                    Accessibility::Protected => b"protected",
                    Accessibility::None => unreachable!(),
                });
                self.write_byte(b' ');
            }
        }
        if property.r#static(self.tree) {
            self.write_str(b"static ");
        }
        if !self.config.strip_ts {
            if property.r#abstract(self.tree) {
                self.write_str(b"abstract ");
            }
            if property.r#override(self.tree) {
                self.write_str(b"override ");
            }
            if property.readonly(self.tree) {
                self.write_str(b"readonly ");
            }
        }
        if property.accessor(self.tree) {
            self.write_str(b"accessor ");
        }
        self.print_class_key(
            key,
            property.computed(self.tree),
            property.r#static(self.tree),
            true,
        );
        if !self.config.strip_ts {
            if property.optional(self.tree) {
                self.write_byte(b'?');
            }
            if property.definite(self.tree) {
                self.write_byte(b'!');
            }
        }
        if let Some(annotation) = property.type_annotation(self.tree) {
            self.emit(annotation.node_id());
        }
        if let Some(value) = property.value(self.tree) {
            self.print_eq();
            self.emit_value(value.node_id());
        }
        self.soft_semi();
        self.skip_leading_of = None;
    }

    fn emit_decorator(&mut self, decorator: swc_experimental_ecma_ast::Decorator) {
        self.write_byte(b'@');
        let expression = decorator.expression(self.tree).node_id();
        self.emit_expr(
            expression,
            ExprContext {
                precedence: if self.decorator_is_simple(expression) {
                    Precedence::LOWEST
                } else {
                    Precedence::GROUPING
                },
                ..ExprContext::default()
            },
        );
    }

    fn decorator_is_simple(&self, idx: NodeId) -> bool {
        match self.node_data(idx) {
            NodeKindData::IdentifierReference(_) => true,
            NodeKindData::MemberExpression(member) if !member.computed(self.tree) => {
                self.decorator_is_simple(member.object(self.tree).node_id())
            }
            NodeKindData::CallExpression(call) => {
                self.decorator_is_simple(call.callee(self.tree).node_id())
            }
            _ => false,
        }
    }

    fn print_decorators(
        &mut self,
        decorators: TypedSubRange<swc_experimental_ecma_ast::Decorator>,
    ) {
        if decorators.is_empty() {
            return;
        }
        let carry = self.sm.as_ref().and_then(sourcemap::State::last_mapping);
        let len = decorators.len();
        for (index, decorator) in decorators.iter().enumerate() {
            let decorator = self.tree.get_node_in_sub_range(decorator);
            self.emit(decorator.node_id());
            if self.pretty() {
                self.newline();
            } else if index + 1 == len {
                self.write_byte(b' ');
            }
        }
        if let (Some(carry), Some(sm)) = (carry, &mut self.sm) {
            sm.record(self.allocator, carry.orig_line, carry.orig_col);
        }
    }

    fn emit_import_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::ImportDeclaration,
    ) {
        use swc_experimental_ecma_ast::{ImportOrExportKind, ImportPhase};
        let specifiers = declaration.specifiers(self.tree);
        if self.config.strip_ts {
            if declaration.import_kind(self.tree) == ImportOrExportKind::Type {
                return;
            }
            if !specifiers.is_empty() && !has_value_import_specifier(self.tree, specifiers) {
                return;
            }
        }
        self.write_str(b"import");
        if declaration.import_kind(self.tree) == ImportOrExportKind::Type {
            self.write_str(b" type");
        }
        if let Some(phase) = declaration.phase(self.tree) {
            self.write_byte(b' ');
            self.write_str(match phase {
                ImportPhase::Source => b"source",
                ImportPhase::Defer => b"defer",
            });
        }
        if !specifiers.is_empty() {
            self.write_byte(b' ');
            let mut index = 0;
            let first = specifiers
                .get_node(self.tree, 0)
                .expect("non-empty import specifiers");
            if self.tree.node_kind(first.node_id()) == NodeKind::ImportDefaultSpecifier {
                self.emit(first.node_id());
                index = 1;
            }
            if index < specifiers.len() {
                if index > 0 {
                    self.comma();
                }
                let next = specifiers
                    .get_node(self.tree, index)
                    .expect("specifier index in bounds");
                if self.tree.node_kind(next.node_id()) == NodeKind::ImportNamespaceSpecifier {
                    self.emit(next.node_id());
                } else {
                    self.write_byte(b'{');
                    self.space();
                    let mut first = true;
                    while index < specifiers.len() {
                        let specifier = specifiers
                            .get_node(self.tree, index)
                            .expect("specifier index in bounds");
                        first = self.emit_separated(specifier.node_id(), first);
                        index += 1;
                    }
                    self.space();
                    self.write_byte(b'}');
                }
            }
            self.write_str(b" from ");
        } else {
            self.write_byte(b' ');
        }
        self.emit(declaration.source(self.tree).node_id());
        self.print_attributes(declaration.attributes(self.tree));
        self.soft_semi();
    }

    fn emit_import_specifier(&mut self, specifier: swc_experimental_ecma_ast::ImportSpecifier) {
        use swc_experimental_ecma_ast::ImportOrExportKind;
        if self.config.strip_ts && specifier.import_kind(self.tree) == ImportOrExportKind::Type {
            return;
        }
        if specifier.import_kind(self.tree) == ImportOrExportKind::Type {
            self.write_str(b"type ");
        }
        let imported = specifier.imported(self.tree).node_id();
        let local = specifier.local(self.tree).node_id();
        self.emit(imported);
        if !same_identifier(self.tree, imported, local) {
            self.write_str(b" as ");
            self.emit(local);
        }
    }

    fn emit_import_default_specifier(
        &mut self,
        specifier: swc_experimental_ecma_ast::ImportDefaultSpecifier,
    ) {
        self.emit(specifier.local(self.tree).node_id());
    }

    fn emit_import_namespace_specifier(
        &mut self,
        specifier: swc_experimental_ecma_ast::ImportNamespaceSpecifier,
    ) {
        self.write_str(b"* as ");
        self.emit(specifier.local(self.tree).node_id());
    }

    fn emit_import_attribute(&mut self, attribute: swc_experimental_ecma_ast::ImportAttribute) {
        self.emit(attribute.key(self.tree).node_id());
        self.write_byte(b':');
        self.space();
        self.emit(attribute.value(self.tree).node_id());
    }

    fn emit_import_expression(&mut self, expression: swc_experimental_ecma_ast::ImportExpression) {
        use swc_experimental_ecma_ast::ImportPhase;
        self.write_str(b"import");
        if let Some(phase) = expression.phase(self.tree) {
            self.write_byte(b'.');
            self.write_str(match phase {
                ImportPhase::Source => b"source",
                ImportPhase::Defer => b"defer",
            });
        }
        self.write_byte(b'(');
        self.emit_value(expression.source(self.tree).node_id());
        if let Some(options) = expression.options(self.tree) {
            self.write_byte(b',');
            self.space();
            self.emit_value(options.node_id());
        }
        self.write_byte(b')');
    }

    fn emit_export_named_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::ExportNamedDeclaration,
    ) {
        use swc_experimental_ecma_ast::ImportOrExportKind;
        if self.config.strip_ts && declaration.export_kind(self.tree) == ImportOrExportKind::Type {
            return;
        }
        let specifiers = declaration.specifiers(self.tree);
        if self.config.strip_ts
            && declaration.declaration(self.tree).is_none()
            && !specifiers.is_empty()
            && !has_value_export_specifier(self.tree, specifiers)
        {
            return;
        }
        let cursor = self.cursor();
        self.write_str(b"export");
        if declaration.export_kind(self.tree) == ImportOrExportKind::Type
            && declaration.declaration(self.tree).is_none()
        {
            self.write_str(b" type");
        }
        if let Some(inner) = declaration.declaration(self.tree) {
            self.write_byte(b' ');
            if !self.try_emit(inner.node_id()) {
                self.restore(cursor);
            }
            return;
        }
        self.space();
        self.write_byte(b'{');
        if !specifiers.is_empty() {
            self.space();
            let mut first = true;
            for specifier in specifiers.iter() {
                let specifier = self.tree.get_node_in_sub_range(specifier);
                first = self.emit_separated(specifier.node_id(), first);
            }
            self.space();
        }
        self.write_byte(b'}');
        if let Some(source) = declaration.source(self.tree) {
            self.write_str(b" from ");
            self.emit(source.node_id());
        }
        self.print_attributes(declaration.attributes(self.tree));
        self.soft_semi();
    }

    fn emit_export_default_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::ExportDefaultDeclaration,
    ) {
        let cursor = self.cursor();
        self.write_str(b"export default ");
        let inner = declaration.declaration(self.tree).node_id();
        let kind = self.tree.node_kind(inner);
        let emitted = if matches!(
            kind,
            NodeKind::Function | NodeKind::Class | NodeKind::TsInterfaceDeclaration
        ) {
            self.try_emit(inner)
        } else {
            self.emit_expr(
                inner,
                ExprContext {
                    precedence: Precedence::ASSIGNMENT,
                    ..ExprContext::default()
                },
            );
            true
        };
        if !emitted {
            self.restore(cursor);
            return;
        }
        if !matches!(
            kind,
            NodeKind::Function | NodeKind::Class | NodeKind::TsInterfaceDeclaration
        ) {
            self.soft_semi();
        }
    }

    fn emit_export_all_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::ExportAllDeclaration,
    ) {
        use swc_experimental_ecma_ast::ImportOrExportKind;
        if self.config.strip_ts && declaration.export_kind(self.tree) == ImportOrExportKind::Type {
            return;
        }
        self.write_str(b"export");
        if declaration.export_kind(self.tree) == ImportOrExportKind::Type {
            self.write_str(b" type");
        }
        self.write_str(b" *");
        if let Some(exported) = declaration.exported(self.tree) {
            self.write_str(b" as ");
            self.emit(exported.node_id());
        }
        self.write_str(b" from ");
        self.emit(declaration.source(self.tree).node_id());
        self.print_attributes(declaration.attributes(self.tree));
        self.soft_semi();
    }

    fn emit_export_specifier(&mut self, specifier: swc_experimental_ecma_ast::ExportSpecifier) {
        use swc_experimental_ecma_ast::ImportOrExportKind;
        if self.config.strip_ts && specifier.export_kind(self.tree) == ImportOrExportKind::Type {
            return;
        }
        if specifier.export_kind(self.tree) == ImportOrExportKind::Type {
            self.write_str(b"type ");
        }
        let local = specifier.local(self.tree).node_id();
        let exported = specifier.exported(self.tree).node_id();
        self.emit(local);
        if !same_identifier(self.tree, local, exported) {
            self.write_str(b" as ");
            self.emit(exported);
        }
    }

    fn print_attributes(
        &mut self,
        attributes: TypedSubRange<swc_experimental_ecma_ast::ImportAttribute>,
    ) {
        if attributes.is_empty() {
            return;
        }
        self.write_str(b" with ");
        self.write_byte(b'{');
        self.space();
        self.emit_list(attributes);
        self.space();
        self.write_byte(b'}');
    }

    fn emit_ts_type_annotation(&mut self, annotation: swc_experimental_ecma_ast::TsTypeAnnotation) {
        self.write_byte(b':');
        self.space();
        self.emit(annotation.type_annotation(self.tree).node_id());
    }

    fn emit_ts_type_reference(&mut self, reference: swc_experimental_ecma_ast::TsTypeReference) {
        self.emit_entity_name(reference.type_name(self.tree).node_id());
        if let Some(arguments) = reference.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
    }

    fn emit_ts_qualified_name(&mut self, qualified: swc_experimental_ecma_ast::TsQualifiedName) {
        self.emit_entity_name(qualified.left(self.tree).node_id());
        self.write_byte(b'.');
        self.emit_entity_name(qualified.right(self.tree).node_id());
    }

    fn emit_ts_type_query(&mut self, query: swc_experimental_ecma_ast::TsTypeQuery) {
        self.write_str(b"typeof ");
        self.emit_entity_name(query.expr_name(self.tree).node_id());
        if let Some(arguments) = query.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
    }

    fn emit_ts_import_type(&mut self, import: swc_experimental_ecma_ast::TsImportType) {
        self.write_str(b"import(");
        self.emit(import.source(self.tree).node_id());
        if let Some(options) = import.options(self.tree) {
            self.write_byte(b',');
            self.space();
            self.emit(options.node_id());
        }
        self.write_byte(b')');
        if let Some(qualifier) = import.qualifier(self.tree) {
            self.write_byte(b'.');
            self.emit_entity_name(qualifier.node_id());
        }
        if let Some(arguments) = import.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
    }

    fn emit_entity_name(&mut self, idx: NodeId) {
        if let NodeKindData::IdentifierReference(identifier) = self.node_data(idx) {
            if self.sm.is_some() {
                self.record_mapping(idx);
            }
            self.write_string(identifier.name(self.tree));
        } else {
            self.emit(idx);
        }
    }

    fn emit_ts_type_parameter(&mut self, parameter: swc_experimental_ecma_ast::TsTypeParameter) {
        if parameter.r#const(self.tree) {
            self.write_str(b"const ");
        }
        if parameter.r#in(self.tree) {
            self.write_str(b"in ");
        }
        if parameter.out(self.tree) {
            self.write_str(b"out ");
        }
        self.emit(parameter.name(self.tree).node_id());
        if let Some(constraint) = parameter.constraint(self.tree) {
            self.write_str(b" extends ");
            self.emit(constraint.node_id());
        }
        if let Some(default) = parameter.default(self.tree) {
            self.print_eq();
            self.emit(default.node_id());
        }
    }

    fn print_angle_list<T>(&mut self, parameters: TypedSubRange<T>)
    where
        T: AstNode + ExtraDataCompact<'a>,
    {
        self.write_byte(b'<');
        self.emit_list(parameters);
        self.write_byte(b'>');
    }

    fn emit_ts_type_parameter_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsTypeParameterDeclaration,
    ) {
        self.print_angle_list(declaration.params(self.tree));
    }

    fn emit_ts_type_parameter_instantiation(
        &mut self,
        instantiation: swc_experimental_ecma_ast::TsTypeParameterInstantiation,
    ) {
        self.print_angle_list(instantiation.params(self.tree));
    }

    fn emit_ts_literal_type(&mut self, literal: swc_experimental_ecma_ast::TsLiteralType) {
        let literal = literal.literal(self.tree).node_id();
        if let NodeKindData::BooleanLiteral(boolean) = self.node_data(literal) {
            self.write_str(if boolean.value(self.tree) {
                b"true"
            } else {
                b"false"
            });
        } else {
            self.emit(literal);
        }
    }

    fn emit_ts_template_literal_type(
        &mut self,
        literal: swc_experimental_ecma_ast::TsTemplateLiteralType,
    ) {
        self.print_template(literal.quasis(self.tree), literal.types(self.tree));
    }

    fn emit_type(&mut self, idx: NodeId, minimum: u8) {
        self.wrap_if(self.type_precedence(idx) < minimum, idx);
    }

    fn type_precedence(&self, idx: NodeId) -> u8 {
        match self.tree.node_kind(idx) {
            NodeKind::TsFunctionType
            | NodeKind::TsConstructorType
            | NodeKind::TsConditionalType
            | NodeKind::TsInferType => TypePrecedence::TRAILING,
            NodeKind::TsUnionType => TypePrecedence::UNION,
            NodeKind::TsIntersectionType => TypePrecedence::INTERSECTION,
            NodeKind::TsTypeOperator => TypePrecedence::OPERATOR,
            _ => TypePrecedence::PRIMARY,
        }
    }

    fn emit_ts_array_type(&mut self, array: swc_experimental_ecma_ast::TsArrayType) {
        self.emit_type(
            array.element_type(self.tree).node_id(),
            TypePrecedence::PRIMARY,
        );
        self.write_str(b"[]");
    }

    fn emit_ts_indexed_access_type(
        &mut self,
        indexed: swc_experimental_ecma_ast::TsIndexedAccessType,
    ) {
        self.emit_type(
            indexed.object_type(self.tree).node_id(),
            TypePrecedence::PRIMARY,
        );
        self.write_byte(b'[');
        self.emit(indexed.index_type(self.tree).node_id());
        self.write_byte(b']');
    }

    fn emit_ts_tuple_type(&mut self, tuple: swc_experimental_ecma_ast::TsTupleType) {
        self.write_byte(b'[');
        self.emit_list(tuple.element_types(self.tree));
        self.write_byte(b']');
    }

    fn emit_ts_named_tuple_member(
        &mut self,
        member: swc_experimental_ecma_ast::TsNamedTupleMember,
    ) {
        self.emit(member.label(self.tree).node_id());
        if member.optional(self.tree) {
            self.write_byte(b'?');
        }
        self.write_byte(b':');
        self.space();
        self.emit(member.element_type(self.tree).node_id());
    }

    fn emit_ts_optional_type(&mut self, optional: swc_experimental_ecma_ast::TsOptionalType) {
        self.emit_type(
            optional.type_annotation(self.tree).node_id(),
            TypePrecedence::PRIMARY,
        );
        self.write_byte(b'?');
    }

    fn emit_ts_rest_type(&mut self, rest: swc_experimental_ecma_ast::TsRestType) {
        self.write_str(b"...");
        self.emit(rest.type_annotation(self.tree).node_id());
    }

    fn emit_ts_jsdoc_nullable_type(
        &mut self,
        nullable: swc_experimental_ecma_ast::TsJsDocNullableType,
    ) {
        if nullable.postfix(self.tree) {
            self.emit(nullable.type_annotation(self.tree).node_id());
            self.write_byte(b'?');
        } else {
            self.write_byte(b'?');
            self.emit(nullable.type_annotation(self.tree).node_id());
        }
    }

    fn emit_ts_jsdoc_non_nullable_type(
        &mut self,
        non_nullable: swc_experimental_ecma_ast::TsJsDocNonNullableType,
    ) {
        if non_nullable.postfix(self.tree) {
            self.emit(non_nullable.type_annotation(self.tree).node_id());
            self.write_byte(b'!');
        } else {
            self.write_byte(b'!');
            self.emit(non_nullable.type_annotation(self.tree).node_id());
        }
    }

    fn emit_ts_union_type(&mut self, union: swc_experimental_ecma_ast::TsUnionType) {
        self.emit_type_list(union.types(self.tree), b'|');
    }

    fn emit_ts_intersection_type(
        &mut self,
        intersection: swc_experimental_ecma_ast::TsIntersectionType,
    ) {
        self.emit_type_list(intersection.types(self.tree), b'&');
    }

    fn emit_type_list<T>(&mut self, types: TypedSubRange<T>, operator: u8)
    where
        T: AstNode + ExtraDataCompact<'a>,
    {
        if types.len() == 1 {
            self.write_byte(operator);
            self.space();
        }
        let minimum = if operator == b'|' {
            TypePrecedence::INTERSECTION
        } else {
            TypePrecedence::OPERATOR
        };
        for (index, item) in types.iter().enumerate() {
            if index > 0 {
                self.space();
                self.write_byte(operator);
                self.space();
            }
            let item = self.tree.get_node_in_sub_range(item);
            self.emit_type(item.node_id(), minimum);
        }
    }

    fn emit_ts_conditional_type(
        &mut self,
        conditional: swc_experimental_ecma_ast::TsConditionalType,
    ) {
        self.emit_type(
            conditional.check_type(self.tree).node_id(),
            TypePrecedence::UNION,
        );
        self.write_str(b" extends ");
        self.emit_type(
            conditional.extends_type(self.tree).node_id(),
            TypePrecedence::UNION,
        );
        self.space();
        self.write_byte(b'?');
        self.space();
        self.emit(conditional.true_type(self.tree).node_id());
        self.space();
        self.write_byte(b':');
        self.space();
        self.emit(conditional.false_type(self.tree).node_id());
    }

    fn emit_ts_infer_type(&mut self, infer: swc_experimental_ecma_ast::TsInferType) {
        self.write_str(b"infer ");
        self.emit(infer.type_parameter(self.tree).node_id());
    }

    fn emit_ts_type_operator(&mut self, operator: swc_experimental_ecma_ast::TsTypeOperator) {
        use swc_experimental_ecma_ast::TsTypeOperatorKind;
        self.write_str(match operator.operator(self.tree) {
            TsTypeOperatorKind::Keyof => b"keyof",
            TsTypeOperatorKind::Unique => b"unique",
            TsTypeOperatorKind::Readonly => b"readonly",
        });
        self.write_byte(b' ');
        self.emit_type(
            operator.type_annotation(self.tree).node_id(),
            TypePrecedence::OPERATOR,
        );
    }

    fn emit_ts_parenthesized_type(
        &mut self,
        parenthesized: swc_experimental_ecma_ast::TsParenthesizedType,
    ) {
        self.write_byte(b'(');
        self.emit(parenthesized.type_annotation(self.tree).node_id());
        self.write_byte(b')');
    }

    fn emit_ts_function_type(&mut self, function: swc_experimental_ecma_ast::TsFunctionType) {
        self.print_arrow_type(
            function.type_parameters(self.tree).map(AstNode::node_id),
            function.params(self.tree).node_id(),
            function.return_type(self.tree).node_id(),
        );
    }

    fn emit_ts_constructor_type(
        &mut self,
        constructor: swc_experimental_ecma_ast::TsConstructorType,
    ) {
        if constructor.r#abstract(self.tree) {
            self.write_str(b"abstract ");
        }
        self.write_str(b"new ");
        self.print_arrow_type(
            constructor.type_parameters(self.tree).map(AstNode::node_id),
            constructor.params(self.tree).node_id(),
            constructor.return_type(self.tree).node_id(),
        );
    }

    fn print_arrow_type(
        &mut self,
        type_parameters: Option<NodeId>,
        parameters: NodeId,
        return_type: NodeId,
    ) {
        if let Some(type_parameters) = type_parameters {
            self.emit(type_parameters);
        }
        self.emit(parameters);
        self.space();
        self.write_str(b"=>");
        self.space();
        self.emit_unwrapped_type(return_type);
    }

    fn emit_unwrapped_type(&mut self, idx: NodeId) {
        if let NodeKindData::TsTypeAnnotation(annotation) = self.node_data(idx) {
            self.emit(annotation.type_annotation(self.tree).node_id());
        } else {
            self.emit(idx);
        }
    }

    fn emit_ts_type_predicate(&mut self, predicate: swc_experimental_ecma_ast::TsTypePredicate) {
        if predicate.asserts(self.tree) {
            self.write_str(b"asserts ");
        }
        self.emit(predicate.parameter_name(self.tree).node_id());
        if let Some(annotation) = predicate.type_annotation(self.tree) {
            self.write_str(b" is ");
            self.emit_unwrapped_type(annotation.node_id());
        }
    }

    fn emit_ts_type_literal(&mut self, literal: swc_experimental_ecma_ast::TsTypeLiteral) {
        self.print_signature_body(literal.members(self.tree));
    }

    fn emit_ts_mapped_type(&mut self, mapped: swc_experimental_ecma_ast::TsMappedType) {
        self.write_byte(b'{');
        self.space();
        self.print_mapped_modifier(mapped.readonly(self.tree), b"readonly");
        self.write_byte(b'[');
        self.emit(mapped.key(self.tree).node_id());
        self.write_str(b" in ");
        self.emit(mapped.constraint(self.tree).node_id());
        if let Some(name_type) = mapped.name_type(self.tree) {
            self.write_str(b" as ");
            self.emit(name_type.node_id());
        }
        self.write_byte(b']');
        self.print_mapped_optional(mapped.optional(self.tree));
        if let Some(annotation) = mapped.type_annotation(self.tree) {
            self.write_byte(b':');
            self.space();
            self.emit(annotation.node_id());
        }
        self.soft_semi();
        self.pending_semi = false;
        self.space();
        self.write_byte(b'}');
    }

    fn print_mapped_modifier(
        &mut self,
        modifier: swc_experimental_ecma_ast::TsMappedTypeModifier,
        keyword: &[u8],
    ) {
        use swc_experimental_ecma_ast::TsMappedTypeModifier;
        match modifier {
            TsMappedTypeModifier::None => {}
            TsMappedTypeModifier::True => {
                self.write_str(keyword);
                self.write_byte(b' ');
            }
            TsMappedTypeModifier::Plus => {
                self.write_byte(b'+');
                self.write_str(keyword);
                self.write_byte(b' ');
            }
            TsMappedTypeModifier::Minus => {
                self.write_byte(b'-');
                self.write_str(keyword);
                self.write_byte(b' ');
            }
        }
    }

    fn print_mapped_optional(&mut self, modifier: swc_experimental_ecma_ast::TsMappedTypeModifier) {
        use swc_experimental_ecma_ast::TsMappedTypeModifier;
        match modifier {
            TsMappedTypeModifier::None => {}
            TsMappedTypeModifier::True => self.write_byte(b'?'),
            TsMappedTypeModifier::Plus => self.write_str(b"+?"),
            TsMappedTypeModifier::Minus => self.write_str(b"-?"),
        }
    }

    fn emit_ts_property_signature(
        &mut self,
        signature: swc_experimental_ecma_ast::TsPropertySignature,
    ) {
        if signature.readonly(self.tree) {
            self.write_str(b"readonly ");
        }
        self.print_property_key(
            signature.key(self.tree).node_id(),
            signature.computed(self.tree),
        );
        if signature.optional(self.tree) {
            self.write_byte(b'?');
        }
        if let Some(annotation) = signature.type_annotation(self.tree) {
            self.emit(annotation.node_id());
        }
        self.soft_semi();
    }

    fn emit_ts_method_signature(
        &mut self,
        signature: swc_experimental_ecma_ast::TsMethodSignature,
    ) {
        use swc_experimental_ecma_ast::TsMethodSignatureKind;
        match signature.kind(self.tree) {
            TsMethodSignatureKind::Get => self.write_str(b"get "),
            TsMethodSignatureKind::Set => self.write_str(b"set "),
            TsMethodSignatureKind::Method => {}
        }
        self.print_property_key(
            signature.key(self.tree).node_id(),
            signature.computed(self.tree),
        );
        if signature.optional(self.tree) {
            self.write_byte(b'?');
        }
        self.print_signature_tail(
            signature.type_parameters(self.tree).map(AstNode::node_id),
            signature.params(self.tree).node_id(),
            signature.return_type(self.tree).map(AstNode::node_id),
        );
    }

    fn print_signature_tail(
        &mut self,
        type_parameters: Option<NodeId>,
        parameters: NodeId,
        return_type: Option<NodeId>,
    ) {
        if let Some(type_parameters) = type_parameters {
            self.emit(type_parameters);
        }
        self.emit(parameters);
        if let Some(return_type) = return_type {
            self.emit(return_type);
        }
        self.soft_semi();
    }

    fn emit_ts_call_signature_declaration(
        &mut self,
        signature: swc_experimental_ecma_ast::TsCallSignatureDeclaration,
    ) {
        self.print_signature_tail(
            signature.type_parameters(self.tree).map(AstNode::node_id),
            signature.params(self.tree).node_id(),
            signature.return_type(self.tree).map(AstNode::node_id),
        );
    }

    fn emit_ts_construct_signature_declaration(
        &mut self,
        signature: swc_experimental_ecma_ast::TsConstructSignatureDeclaration,
    ) {
        self.write_str(b"new ");
        self.print_signature_tail(
            signature.type_parameters(self.tree).map(AstNode::node_id),
            signature.params(self.tree).node_id(),
            signature.return_type(self.tree).map(AstNode::node_id),
        );
    }

    fn emit_ts_index_signature(&mut self, signature: swc_experimental_ecma_ast::TsIndexSignature) {
        if signature.r#static(self.tree) {
            self.write_str(b"static ");
        }
        if signature.readonly(self.tree) {
            self.write_str(b"readonly ");
        }
        self.write_byte(b'[');
        self.emit_list(signature.parameters(self.tree));
        self.write_byte(b']');
        self.emit(signature.type_annotation(self.tree).node_id());
        self.soft_semi();
    }

    fn print_signature_body<T>(&mut self, items: TypedSubRange<T>)
    where
        T: AstNode + ExtraDataCompact<'a>,
    {
        self.write_byte(b'{');
        if !items.is_empty() {
            self.indent_depth += 1;
            for item in items.iter() {
                self.flush_semi();
                self.newline();
                let item = self.tree.get_node_in_sub_range(item);
                self.emit(item.node_id());
            }
            self.indent_depth -= 1;
            self.pending_semi = false;
            self.newline();
        }
        self.write_byte(b'}');
    }

    fn emit_ts_type_alias_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsTypeAliasDeclaration,
    ) {
        if declaration.declare(self.tree) {
            self.write_str(b"declare ");
        }
        self.write_str(b"type ");
        self.emit(declaration.id(self.tree).node_id());
        if let Some(parameters) = declaration.type_parameters(self.tree) {
            self.emit(parameters.node_id());
        }
        self.print_eq();
        let annotation = declaration.type_annotation(self.tree).node_id();
        self.wrap_if(self.is_leftmost_intrinsic_reference(annotation), annotation);
        self.soft_semi();
    }

    fn wrap_if(&mut self, condition: bool, idx: NodeId) {
        if condition {
            self.write_byte(b'(');
        }
        self.emit(idx);
        if condition {
            self.write_byte(b')');
        }
    }

    fn is_leftmost_intrinsic_reference(&self, idx: NodeId) -> bool {
        match self.node_data(idx) {
            NodeKindData::TsTypeReference(reference)
                if reference.type_arguments(self.tree).is_none() =>
            {
                matches!(
                    self.node_data(reference.type_name(self.tree).node_id()),
                    NodeKindData::IdentifierReference(identifier)
                        if self.tree.string(identifier.name(self.tree)) == b"intrinsic"
                )
            }
            NodeKindData::TsArrayType(array) => {
                self.is_leftmost_intrinsic_reference(array.element_type(self.tree).node_id())
            }
            NodeKindData::TsIndexedAccessType(indexed) => {
                self.is_leftmost_intrinsic_reference(indexed.object_type(self.tree).node_id())
            }
            NodeKindData::TsUnionType(union) => union
                .types(self.tree)
                .first()
                .map(|first| self.tree.get_node_in_sub_range(first).node_id())
                .is_some_and(|first| self.is_leftmost_intrinsic_reference(first)),
            NodeKindData::TsIntersectionType(intersection) => intersection
                .types(self.tree)
                .first()
                .map(|first| self.tree.get_node_in_sub_range(first).node_id())
                .is_some_and(|first| self.is_leftmost_intrinsic_reference(first)),
            NodeKindData::TsConditionalType(conditional) => {
                self.is_leftmost_intrinsic_reference(conditional.check_type(self.tree).node_id())
            }
            _ => false,
        }
    }

    fn emit_ts_interface_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsInterfaceDeclaration,
    ) {
        if declaration.declare(self.tree) {
            self.write_str(b"declare ");
        }
        self.write_str(b"interface ");
        self.emit(declaration.id(self.tree).node_id());
        if let Some(parameters) = declaration.type_parameters(self.tree) {
            self.emit(parameters.node_id());
        }
        if !declaration.extends(self.tree).is_empty() {
            self.write_str(b" extends ");
            self.emit_list(declaration.extends(self.tree));
        }
        self.space();
        self.emit(declaration.body(self.tree).node_id());
    }

    fn emit_ts_interface_body(&mut self, body: swc_experimental_ecma_ast::TsInterfaceBody) {
        self.print_signature_body(body.body(self.tree));
    }

    fn emit_ts_interface_heritage(
        &mut self,
        heritage: swc_experimental_ecma_ast::TsInterfaceHeritage,
    ) {
        self.emit(heritage.expression(self.tree).node_id());
        if let Some(arguments) = heritage.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
    }

    fn emit_ts_class_implements(
        &mut self,
        implementation: swc_experimental_ecma_ast::TsClassImplements,
    ) {
        self.emit(implementation.expression(self.tree).node_id());
        if let Some(arguments) = implementation.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
    }

    fn emit_ts_enum_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsEnumDeclaration,
    ) {
        if declaration.declare(self.tree) {
            self.write_str(b"declare ");
        }
        if declaration.is_const(self.tree) {
            self.write_str(b"const ");
        }
        self.write_str(b"enum ");
        self.emit(declaration.id(self.tree).node_id());
        self.space();
        self.emit(declaration.body(self.tree).node_id());
    }

    fn emit_ts_enum_body(&mut self, body: swc_experimental_ecma_ast::TsEnumBody) {
        self.write_byte(b'{');
        let members = body.members(self.tree);
        if !members.is_empty() {
            self.indent_depth += 1;
            let len = members.len();
            for (index, member) in members.iter().enumerate() {
                self.newline();
                let member = self.tree.get_node_in_sub_range(member);
                self.emit(member.node_id());
                if index + 1 < len {
                    self.write_byte(b',');
                }
            }
            self.indent_depth -= 1;
            self.newline();
        }
        self.write_byte(b'}');
    }

    fn emit_ts_enum_member(&mut self, member: swc_experimental_ecma_ast::TsEnumMember) {
        if member.computed(self.tree) {
            self.write_byte(b'[');
            self.emit(member.id(self.tree).node_id());
            self.write_byte(b']');
        } else {
            self.emit(member.id(self.tree).node_id());
        }
        if let Some(initializer) = member.initializer(self.tree) {
            self.print_eq();
            self.emit(initializer.node_id());
        }
    }

    fn emit_ts_module_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsModuleDeclaration,
    ) {
        use swc_experimental_ecma_ast::TsModuleDeclarationKind;
        if declaration.declare(self.tree) {
            self.write_str(b"declare ");
        }
        self.write_str(match declaration.kind(self.tree) {
            TsModuleDeclarationKind::Namespace => b"namespace",
            TsModuleDeclarationKind::Module => b"module",
        });
        self.write_byte(b' ');
        self.emit_entity_name(declaration.id(self.tree).node_id());
        if let Some(body) = declaration.body(self.tree) {
            self.space();
            self.emit(body.node_id());
        } else {
            self.soft_semi();
        }
    }

    fn emit_ts_module_block(&mut self, block: swc_experimental_ecma_ast::TsModuleBlock) {
        self.print_block(block.body(self.tree), false);
    }

    fn emit_ts_global_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsGlobalDeclaration,
    ) {
        if declaration.declare(self.tree) {
            self.write_str(b"declare ");
        }
        self.emit_entity_name(declaration.id(self.tree).node_id());
        self.space();
        self.emit(declaration.body(self.tree).node_id());
    }

    fn emit_ts_as_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::TsAsExpression,
        context: ExprContext,
    ) {
        self.emit_expr(
            expression.expression(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::RELATIONAL,
                no_in: context.no_in,
                ..ExprContext::default()
            },
        );
        self.write_str(b" as ");
        self.emit(expression.type_annotation(self.tree).node_id());
    }

    fn emit_ts_satisfies_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::TsSatisfiesExpression,
        context: ExprContext,
    ) {
        self.emit_expr(
            expression.expression(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::RELATIONAL,
                no_in: context.no_in,
                ..ExprContext::default()
            },
        );
        self.write_str(b" satisfies ");
        self.emit(expression.type_annotation(self.tree).node_id());
    }

    fn emit_ts_type_assertion(&mut self, assertion: swc_experimental_ecma_ast::TsTypeAssertion) {
        self.write_byte(b'<');
        let annotation = assertion.type_annotation(self.tree).node_id();
        if type_starts_with_left_angle(self.tree, annotation) {
            self.write_byte(b' ');
        }
        self.emit(annotation);
        self.write_byte(b'>');
        self.emit_expr(
            assertion.expression(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::UNARY,
                ..ExprContext::default()
            },
        );
    }

    fn emit_ts_non_null_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::TsNonNullExpression,
        _context: ExprContext,
    ) {
        self.emit_expr(
            expression.expression(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::POSTFIX,
                ..ExprContext::default()
            },
        );
        self.write_byte(b'!');
    }

    fn emit_ts_instantiation_expression(
        &mut self,
        expression: swc_experimental_ecma_ast::TsInstantiationExpression,
        _context: ExprContext,
    ) {
        self.emit_expr(
            expression.expression(self.tree).node_id(),
            ExprContext {
                precedence: Precedence::POSTFIX,
                ..ExprContext::default()
            },
        );
        self.emit(expression.type_arguments(self.tree).node_id());
    }

    fn emit_ts_export_assignment(
        &mut self,
        assignment: swc_experimental_ecma_ast::TsExportAssignment,
    ) {
        self.write_str(b"export");
        self.print_eq();
        self.emit(assignment.expression(self.tree).node_id());
        self.soft_semi();
    }

    fn emit_ts_namespace_export_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsNamespaceExportDeclaration,
    ) {
        self.write_str(b"export as namespace ");
        self.emit(declaration.id(self.tree).node_id());
        self.soft_semi();
    }

    fn emit_ts_import_equals_declaration(
        &mut self,
        declaration: swc_experimental_ecma_ast::TsImportEqualsDeclaration,
    ) {
        use swc_experimental_ecma_ast::ImportOrExportKind;
        self.write_str(b"import ");
        if declaration.import_kind(self.tree) == ImportOrExportKind::Type {
            self.write_str(b"type ");
        }
        self.emit(declaration.id(self.tree).node_id());
        self.print_eq();
        self.emit(declaration.module_reference(self.tree).node_id());
        self.soft_semi();
    }

    fn emit_ts_external_module_reference(
        &mut self,
        reference: swc_experimental_ecma_ast::TsExternalModuleReference,
    ) {
        self.write_str(b"require(");
        self.emit(reference.expression(self.tree).node_id());
        self.write_byte(b')');
    }

    fn emit_ts_parameter_property(
        &mut self,
        property: swc_experimental_ecma_ast::TsParameterProperty,
    ) {
        use swc_experimental_ecma_ast::Accessibility;
        self.print_decorators(property.decorators(self.tree));
        if property.accessibility(self.tree) != Accessibility::None {
            self.write_str(match property.accessibility(self.tree) {
                Accessibility::Public => b"public",
                Accessibility::Private => b"private",
                Accessibility::Protected => b"protected",
                Accessibility::None => unreachable!(),
            });
            self.write_byte(b' ');
        }
        if property.r#override(self.tree) {
            self.write_str(b"override ");
        }
        if property.readonly(self.tree) {
            self.write_str(b"readonly ");
        }
        self.emit(property.parameter(self.tree).node_id());
    }

    fn emit_ts_this_parameter(&mut self, parameter: swc_experimental_ecma_ast::TsThisParameter) {
        self.write_str(b"this");
        if let Some(annotation) = parameter.type_annotation(self.tree) {
            self.emit(annotation.node_id());
        }
    }

    fn emit_jsx_element(&mut self, element: swc_experimental_ecma_ast::JsxElement) {
        self.emit(element.opening_element(self.tree).node_id());
        for child in element.children(self.tree).iter() {
            let child = self.tree.get_node_in_sub_range(child);
            self.emit(child.node_id());
        }
        if let Some(closing) = element.closing_element(self.tree) {
            self.emit(closing.node_id());
        }
    }

    fn emit_jsx_opening_element(&mut self, opening: swc_experimental_ecma_ast::JsxOpeningElement) {
        self.write_byte(b'<');
        self.emit(opening.name(self.tree).node_id());
        if let Some(arguments) = opening.type_arguments(self.tree) {
            self.emit(arguments.node_id());
        }
        for attribute in opening.attributes(self.tree).iter() {
            self.write_byte(b' ');
            let attribute = self.tree.get_node_in_sub_range(attribute);
            self.emit(attribute.node_id());
        }
        if opening.self_closing(self.tree) {
            self.space();
            self.write_str(b"/>");
        } else {
            self.write_byte(b'>');
        }
    }

    fn emit_jsx_closing_element(&mut self, closing: swc_experimental_ecma_ast::JsxClosingElement) {
        self.write_str(b"</");
        self.emit(closing.name(self.tree).node_id());
        self.write_byte(b'>');
    }

    fn emit_jsx_fragment(&mut self, fragment: swc_experimental_ecma_ast::JsxFragment) {
        self.emit(fragment.opening_fragment(self.tree).node_id());
        for child in fragment.children(self.tree).iter() {
            let child = self.tree.get_node_in_sub_range(child);
            self.emit(child.node_id());
        }
        self.emit(fragment.closing_fragment(self.tree).node_id());
    }

    fn emit_jsx_identifier(&mut self, identifier: swc_experimental_ecma_ast::JsxIdentifier) {
        self.write_string(identifier.name(self.tree));
    }

    fn emit_jsx_namespaced_name(&mut self, name: swc_experimental_ecma_ast::JsxNamespacedName) {
        self.emit(name.namespace(self.tree).node_id());
        self.write_byte(b':');
        self.emit(name.name(self.tree).node_id());
    }

    fn emit_jsx_member_expression(
        &mut self,
        member: swc_experimental_ecma_ast::JsxMemberExpression,
    ) {
        self.emit(member.object(self.tree).node_id());
        self.write_byte(b'.');
        self.emit(member.property(self.tree).node_id());
    }

    fn emit_jsx_attribute(&mut self, attribute: swc_experimental_ecma_ast::JsxAttribute) {
        self.emit(attribute.name(self.tree).node_id());
        if let Some(value) = attribute.value(self.tree) {
            self.write_byte(b'=');
            let value = value.node_id();
            if let NodeKindData::StringLiteral(literal) = self.node_data(value) {
                self.write_string(literal.raw(self.tree));
            } else {
                self.emit(value);
            }
        }
    }

    fn emit_jsx_spread_attribute(
        &mut self,
        attribute: swc_experimental_ecma_ast::JsxSpreadAttribute,
    ) {
        self.write_byte(b'{');
        self.write_str(b"...");
        self.emit_value(attribute.argument(self.tree).node_id());
        self.write_byte(b'}');
    }

    fn emit_jsx_expression_container(
        &mut self,
        container: swc_experimental_ecma_ast::JsxExpressionContainer,
    ) {
        self.write_byte(b'{');
        self.emit_value(container.expression(self.tree).node_id());
        self.write_byte(b'}');
    }

    fn emit_jsx_empty_expression(&mut self, _: swc_experimental_ecma_ast::JsxEmptyExpression) {}

    fn emit_jsx_text(&mut self, text: swc_experimental_ecma_ast::JsxText) {
        self.write_string(text.value(self.tree));
    }

    fn emit_jsx_spread_child(&mut self, child: swc_experimental_ecma_ast::JsxSpreadChild) {
        self.write_byte(b'{');
        self.write_str(b"...");
        self.emit_value(child.expression(self.tree).node_id());
        self.write_byte(b'}');
    }
}

fn is_bare_async_identifier(tree: &Ast<'_>, idx: NodeId) -> bool {
    identifier_string_or_null(tree, idx).is_some_and(|name| tree.string(name) == b"async")
}

fn leftmost_byte_is(tree: &Ast<'_>, idx: NodeId, byte: u8) -> bool {
    use swc_experimental_ecma_ast::{UnaryOperator, UpdateOperator};
    match tree.kind_data(idx) {
        NodeKindData::UnaryExpression(unary) => match unary.operator(tree) {
            UnaryOperator::Positive => byte == b'+',
            UnaryOperator::Negate => byte == b'-',
            UnaryOperator::LogicalNot => byte == b'!',
            UnaryOperator::BitwiseNot => byte == b'~',
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete => false,
        },
        NodeKindData::UpdateExpression(update) if update.prefix(tree) => {
            match update.operator(tree) {
                UpdateOperator::Increment => byte == b'+',
                UpdateOperator::Decrement => byte == b'-',
            }
        }
        NodeKindData::UpdateExpression(update) => {
            leftmost_byte_is(tree, update.argument(tree).node_id(), byte)
        }
        NodeKindData::RegExpLiteral(_) => byte == b'/',
        NodeKindData::MemberExpression(member) => {
            leftmost_byte_is(tree, member.object(tree).node_id(), byte)
        }
        NodeKindData::CallExpression(call) => {
            leftmost_byte_is(tree, call.callee(tree).node_id(), byte)
        }
        NodeKindData::ChainExpression(chain) => {
            leftmost_byte_is(tree, chain.expression(tree).node_id(), byte)
        }
        NodeKindData::TaggedTemplateExpression(tagged) => {
            leftmost_byte_is(tree, tagged.tag(tree).node_id(), byte)
        }
        NodeKindData::BinaryExpression(binary) => {
            leftmost_byte_is(tree, binary.left(tree).node_id(), byte)
        }
        NodeKindData::LogicalExpression(logical) => {
            leftmost_byte_is(tree, logical.left(tree).node_id(), byte)
        }
        NodeKindData::ConditionalExpression(conditional) => {
            leftmost_byte_is(tree, conditional.test(tree).node_id(), byte)
        }
        NodeKindData::AssignmentExpression(assignment) => {
            leftmost_byte_is(tree, assignment.left(tree).node_id(), byte)
        }
        NodeKindData::SequenceExpression(sequence) => sequence
            .expressions(tree)
            .get_node(tree, 0)
            .is_some_and(|first| leftmost_byte_is(tree, first.node_id(), byte)),
        NodeKindData::TsAsExpression(expression) => {
            leftmost_byte_is(tree, expression.expression(tree).node_id(), byte)
        }
        NodeKindData::TsSatisfiesExpression(expression) => {
            leftmost_byte_is(tree, expression.expression(tree).node_id(), byte)
        }
        NodeKindData::TsNonNullExpression(expression) => {
            leftmost_byte_is(tree, expression.expression(tree).node_id(), byte)
        }
        NodeKindData::TsInstantiationExpression(expression) => {
            leftmost_byte_is(tree, expression.expression(tree).node_id(), byte)
        }
        _ => false,
    }
}

fn same_identifier(tree: &Ast<'_>, a: NodeId, b: NodeId) -> bool {
    let Some(a) = identifier_string_or_null(tree, a) else {
        return false;
    };
    let Some(b) = identifier_string_or_null(tree, b) else {
        return false;
    };
    tree.string(a) == tree.string(b)
}

fn logical_mismatch(
    tree: &Ast<'_>,
    parent: swc_experimental_ecma_ast::LogicalOperator,
    child: NodeId,
) -> bool {
    let NodeKindData::LogicalExpression(expression) = tree.kind_data(child) else {
        return false;
    };
    let child = expression.operator(tree);
    (parent == swc_experimental_ecma_ast::LogicalOperator::NullishCoalescing)
        != (child == swc_experimental_ecma_ast::LogicalOperator::NullishCoalescing)
}

fn shorthand_still_valid(tree: &Ast<'_>, key: NodeId, value: NodeId) -> bool {
    let value = if let NodeKindData::AssignmentPattern(pattern) = tree.kind_data(value) {
        pattern.left(tree).node_id()
    } else {
        value
    };
    same_identifier(tree, key, value)
}

fn identifier_string_or_null(
    tree: &Ast<'_>,
    idx: NodeId,
) -> Option<swc_experimental_ecma_ast::String> {
    match tree.kind_data(idx) {
        NodeKindData::IdentifierName(identifier) => Some(identifier.name(tree)),
        NodeKindData::IdentifierReference(identifier) => Some(identifier.name(tree)),
        NodeKindData::BindingIdentifier(identifier) => Some(identifier.name(tree)),
        _ => None,
    }
}

fn has_value_import_specifier(
    tree: &Ast<'_>,
    list: TypedSubRange<swc_experimental_ecma_ast::ImportDeclarationSpecifier>,
) -> bool {
    use swc_experimental_ecma_ast::ImportOrExportKind;
    for specifier in list.iter() {
        let specifier = tree.get_node_in_sub_range(specifier);
        match tree.kind_data(specifier.node_id()) {
            NodeKindData::ImportDefaultSpecifier(_) | NodeKindData::ImportNamespaceSpecifier(_) => {
                return true;
            }
            NodeKindData::ImportSpecifier(specifier)
                if specifier.import_kind(tree) != ImportOrExportKind::Type =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn has_value_export_specifier(
    tree: &Ast<'_>,
    list: TypedSubRange<swc_experimental_ecma_ast::ExportSpecifier>,
) -> bool {
    use swc_experimental_ecma_ast::ImportOrExportKind;
    for specifier in list.iter() {
        let specifier = tree.get_node_in_sub_range(specifier);
        if specifier.export_kind(tree) != ImportOrExportKind::Type {
            return true;
        }
    }
    false
}

fn needs_parens_as_assign_target(tree: &Ast<'_>, idx: NodeId) -> bool {
    matches!(
        tree.node_kind(idx),
        NodeKind::TsAsExpression | NodeKind::TsSatisfiesExpression | NodeKind::TsTypeAssertion
    )
}

fn type_starts_with_left_angle(tree: &Ast<'_>, idx: NodeId) -> bool {
    match tree.kind_data(idx) {
        NodeKindData::TsFunctionType(function) => function.type_parameters(tree).is_some(),
        NodeKindData::TsConstructorType(constructor) => {
            !constructor.r#abstract(tree) && constructor.type_parameters(tree).is_some()
        }
        _ => false,
    }
}

fn simple_string_key<'a>(tree: &'a Ast<'a>, idx: NodeId) -> Option<&'a [u8]> {
    let NodeKindData::StringLiteral(literal) = tree.kind_data(idx) else {
        return None;
    };
    let value = tree.string(literal.value(tree));
    utils::is_identifier_name(value).then_some(value)
}

fn is_bare_integer_head(head: &[u8]) -> bool {
    head.first().is_some_and(u8::is_ascii_digit)
        && head[1..]
            .iter()
            .all(|byte| byte.is_ascii_digit() || *byte == b'_')
}

fn ends_with_ts_cast(tree: &Ast<'_>, idx: NodeId) -> bool {
    match tree.kind_data(idx) {
        NodeKindData::TsAsExpression(_) | NodeKindData::TsSatisfiesExpression(_) => true,
        NodeKindData::BinaryExpression(binary) => {
            ends_with_ts_cast(tree, binary.right(tree).node_id())
        }
        NodeKindData::LogicalExpression(logical) => {
            ends_with_ts_cast(tree, logical.right(tree).node_id())
        }
        NodeKindData::AssignmentExpression(assignment) => {
            ends_with_ts_cast(tree, assignment.right(tree).node_id())
        }
        NodeKindData::ConditionalExpression(conditional) => {
            ends_with_ts_cast(tree, conditional.alternate(tree).node_id())
        }
        _ => false,
    }
}
