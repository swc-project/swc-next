//! Class modifiers belong to the member, before its real key and function.

use swc_next_ecma_ast::{
    Accessibility, Ast, GetSpan, MethodDefinition, MethodDefinitionKind, PropertyDefinition,
    PropertyKey, PropertyKeyData, Span, TsMethodDefinition,
};

use crate::formatter::comments::CommentRunLayout;
use crate::formatter::expressions::objects::{ConstructorKeyGrammar, PropertyKeyContext};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::{Formatter, suppression::SuppressionPosition};
use crate::{FormatError, document::Writer};

const DECLARE: u8 = 1;
const PUBLIC: u8 = 2;
const PRIVATE: u8 = 4;
const PROTECTED: u8 = 8;
const STATIC: u8 = 16;
const ABSTRACT: u8 = 32;
const OVERRIDE: u8 = 64;
const READONLY: u8 = 128;
const MODIFIERS: [(&str, u8); 8] = [
    ("declare", DECLARE),
    ("public", PUBLIC),
    ("private", PRIVATE),
    ("protected", PROTECTED),
    ("static", STATIC),
    ("abstract", ABSTRACT),
    ("override", OVERRIDE),
    ("readonly", READONLY),
];

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ClassMemberPrefixOwner {
    Field(PropertyDefinition),
    Method(MethodDefinition),
    Bodyless(TsMethodDefinition),
}

impl ClassMemberPrefixOwner {
    pub(in crate::formatter) fn key_context(self, ast: &Ast<'_>) -> PropertyKeyContext {
        let (kind, optional, generator, generics) = match self {
            Self::Field(_) => return PropertyKeyContext::ClassField,
            Self::Method(method) => {
                let function = method.value(ast);
                (
                    method.kind(ast),
                    method.optional(ast),
                    function.generator(ast),
                    function.type_parameters(ast).is_some(),
                )
            }
            Self::Bodyless(method) => {
                let function = method.value(ast);
                (
                    method.kind(ast),
                    method.optional(ast),
                    function.generator(ast),
                    function.type_parameters(ast).is_some(),
                )
            }
        };
        // TS consumes static/async modifiers before the key. Accessors and `*`
        // take different parser branches and retain identifier-only diagnostics.
        let constructor =
            if generator || matches!(kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set) {
                ConstructorKeyGrammar::DistinctDiagnostics
            } else if optional || generics {
                ConstructorKeyGrammar::DistinctClassification
            } else {
                ConstructorKeyGrammar::Equivalent
            };
        PropertyKeyContext::ClassMethod(constructor)
    }
}

#[derive(Clone, Copy)]
enum PrefixMode {
    Lexical,
    Formatted,
    Source,
}

pub(in crate::formatter) struct ClassMemberPrefix {
    span: Span,
    modifiers: u8,
    mode: PrefixMode,
}

pub(in crate::formatter) struct ClassMemberPrefixParts {
    pub(in crate::formatter) member: ClassMemberPrefix,
    pub(in crate::formatter) keyword: Option<(&'static str, Span)>,
    pub(in crate::formatter) star: Option<u32>,
    pub(in crate::formatter) opening: Option<u32>,
}

impl ClassMemberPrefixParts {
    fn canonical(
        source: &str,
        span: Span,
        modifiers: u8,
        keyword: Option<&'static str>,
        generator: bool,
        computed: bool,
    ) -> Option<Self> {
        let mut remaining = source;
        let mut cursor = span.start;
        let mut end = cursor;
        for (word, bit) in MODIFIERS {
            if modifiers & bit == 0 {
                continue;
            }
            remaining = remaining.strip_prefix(word)?;
            cursor += word.len() as u32;
            end = cursor;
            remaining = remaining.strip_prefix(' ')?;
            cursor += 1;
        }
        let keyword = if let Some(word) = keyword {
            remaining = remaining.strip_prefix(word)?;
            let token = Span::new(cursor, cursor + word.len() as u32);
            cursor = token.end;
            remaining = remaining.strip_prefix(' ')?;
            cursor += 1;
            Some((word, token))
        } else {
            None
        };
        let star = if generator {
            remaining = remaining.strip_prefix('*')?;
            let position = cursor;
            cursor += 1;
            Some(position)
        } else {
            None
        };
        let opening = if computed {
            remaining = remaining.strip_prefix('[')?;
            Some(cursor)
        } else {
            None
        };
        remaining.is_empty().then_some(Self {
            member: ClassMemberPrefix {
                span: Span::new(span.start, end),
                modifiers,
                mode: PrefixMode::Formatted,
            },
            keyword,
            star,
            opening,
        })
    }
}

impl ClassMemberPrefix {
    pub(in crate::formatter) fn ordinary(start: u32, is_static: bool) -> Self {
        Self {
            span: Span::new(start, start + if is_static { 6 } else { 0 }),
            modifiers: if is_static { STATIC } else { 0 },
            mode: PrefixMode::Lexical,
        }
    }

    pub(in crate::formatter) fn start(&self) -> u32 {
        self.span.start
    }
    pub(in crate::formatter) fn end(&self) -> u32 {
        self.span.end
    }
    pub(in crate::formatter) fn has_modifiers(&self) -> bool {
        self.modifiers != 0
    }
    pub(in crate::formatter) fn is_source(&self) -> bool {
        matches!(self.mode, PrefixMode::Source)
    }
    pub(in crate::formatter) fn is_formatted(&self) -> bool {
        matches!(self.mode, PrefixMode::Formatted)
    }
    pub(in crate::formatter) fn source_span(&self) -> Option<Span> {
        (self.is_source() && self.span.start < self.span.end).then_some(self.span)
    }
}

impl ClassMemberPrefixOwner {
    fn key(self, ast: &Ast<'_>) -> PropertyKey {
        match self {
            Self::Field(node) => node.key(ast),
            Self::Method(node) => node.key(ast),
            Self::Bodyless(node) => node.key(ast),
        }
    }
    fn modifiers(self, ast: &Ast<'_>) -> u8 {
        let (accessibility, is_static, declare, is_abstract, is_override, readonly) = match self {
            Self::Field(node) => (
                node.accessibility(ast),
                node.r#static(ast),
                node.declare(ast),
                node.r#abstract(ast),
                node.r#override(ast),
                node.readonly(ast),
            ),
            Self::Method(node) => (
                node.accessibility(ast),
                node.r#static(ast),
                false,
                node.r#abstract(ast),
                node.r#override(ast),
                false,
            ),
            Self::Bodyless(node) => (
                node.accessibility(ast),
                node.r#static(ast),
                false,
                node.r#abstract(ast),
                node.r#override(ast),
                false,
            ),
        };
        let accessibility = match accessibility {
            Accessibility::None => 0,
            Accessibility::Public => PUBLIC,
            Accessibility::Private => PRIVATE,
            Accessibility::Protected => PROTECTED,
        };
        accessibility
            | if is_static { STATIC } else { 0 }
            | if declare { DECLARE } else { 0 }
            | if is_abstract { ABSTRACT } else { 0 }
            | if is_override { OVERRIDE } else { 0 }
            | if readonly { READONLY } else { 0 }
    }
    fn prefix_tokens(self, ast: &Ast<'_>) -> (Option<&'static str>, bool, bool) {
        let (kind, is_async, generator, computed) = match self {
            Self::Field(node) => {
                return (
                    node.accessor(ast).then_some("accessor"),
                    false,
                    node.computed(ast),
                );
            }
            Self::Method(node) => {
                let function = node.value(ast);
                (
                    node.kind(ast),
                    function.r#async(ast),
                    function.generator(ast),
                    node.computed(ast),
                )
            }
            Self::Bodyless(node) => {
                let function = node.value(ast);
                (
                    node.kind(ast),
                    function.r#async(ast),
                    function.generator(ast),
                    node.computed(ast),
                )
            }
        };
        let keyword = match kind {
            MethodDefinitionKind::Get => Some("get"),
            MethodDefinitionKind::Set => Some("set"),
            _ if is_async => Some("async"),
            _ => None,
        };
        (keyword, generator, computed)
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn source_class_member_prefix_at(
        &self,
        owner: ClassMemberPrefixOwner,
        core_start: u32,
    ) -> Result<Option<ClassMemberPrefixParts>, FormatError> {
        let ast = self.ast;
        if !ast.is_ts()
            && !matches!(owner, ClassMemberPrefixOwner::Field(node) if node.accessor(ast))
        {
            return Ok(None);
        }
        let expected = owner.modifiers(ast);
        let key = owner.key(ast);
        let span = Span::new(core_start, key.span(ast).start);
        let comments = self.comments.in_range(span);
        let (expected_keyword, generator, computed) = owner.prefix_tokens(ast);
        // A canonical AST prefix avoids locating its tokens individually.
        if comments.is_empty()
            && !self.diagnostic_range_is_sensitive(span)
            && let Some(parts) = ClassMemberPrefixParts::canonical(
                self.source(span),
                span,
                expected,
                expected_keyword,
                generator,
                computed,
            )
        {
            return Ok(Some(parts));
        }
        let field_keyword =
            matches!(owner, ClassMemberPrefixOwner::Field(node) if node.accessor(ast));
        let mut lexical_comments = comments.iter().peekable();
        let mut cursor = span.start;
        let mut end = cursor;
        let mut keyword = None;
        let mut star = None;
        let mut opening = None;
        while cursor < span.end {
            if let Some(comment) = lexical_comments.peek()
                && comment.span.start == cursor
            {
                cursor = comment.span.end;
                lexical_comments.next();
                continue;
            }
            let source = self.source(Span::new(cursor, span.end));
            let Some(character) = source.chars().next() else {
                break;
            };
            if is_source_whitespace(character) {
                cursor += character.len_utf8() as u32;
                continue;
            }
            if character == '*' && generator && star.is_none() {
                star = Some(cursor);
                cursor += 1;
                continue;
            }
            if character == '[' && computed && opening.is_none() {
                opening = Some(cursor);
                cursor += 1;
                continue;
            }
            let length = source.bytes().take_while(u8::is_ascii_alphabetic).count() as u32;
            if length == 0 {
                cursor += character.len_utf8() as u32;
                continue;
            }
            let token = self.source(Span::new(cursor, cursor + length));
            if let Some(word) = expected_keyword
                && word == token
                && keyword.is_none()
                && star.is_none()
                && opening.is_none()
            {
                keyword = Some((word, Span::new(cursor, cursor + length)));
                cursor += length;
                continue;
            }
            // The AST selects flags; this span retains the existing source prefix layout.
            cursor += length;
            if star.is_none() && opening.is_none() && (keyword.is_none() || field_keyword) {
                end = cursor;
            }
        }
        if keyword.is_some() != expected_keyword.is_some()
            || star.is_some() != generator
            || opening.is_some() != computed
        {
            return Err(self.unsupported("class member modifier boundary", key));
        }
        // Only the observed inline Identifier-key relation permits moving a directive.
        let target = (!computed
            && matches!(
                ast.property_key_data(key),
                PropertyKeyData::IdentifierName(_)
            ))
        .then_some(key.span(ast));
        let binding = self.comment_binding(comments, span, target);
        let inline = !self.has_suppression
            || self
                .suppression_positions(comments, span.end)
                .all(|position| matches!(position, SuppressionPosition::Inline));
        let mode = if binding.allows_relocation(target) && inline {
            PrefixMode::Formatted
        } else {
            PrefixMode::Source
        };
        Ok(Some(ClassMemberPrefixParts {
            member: ClassMemberPrefix {
                span: Span::new(span.start, end),
                modifiers: expected,
                mode,
            },
            keyword,
            star,
            opening,
        }))
    }

    pub(in crate::formatter) fn format_class_member_prefix(
        &mut self,
        prefix: &ClassMemberPrefix,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match prefix.mode {
            PrefixMode::Lexical => {
                if prefix.has_modifiers() {
                    writer.text("static");
                }
            }
            PrefixMode::Formatted => {
                for (keyword, bit) in MODIFIERS {
                    if prefix.modifiers & bit != 0 {
                        writer.text(keyword);
                        writer.space();
                    }
                }
            }
            PrefixMode::Source => self.format_verbatim(prefix.span, writer)?,
        }
        Ok(())
    }

    pub(in crate::formatter) fn format_class_member_key_gap(
        &mut self,
        prefix: &ClassMemberPrefix,
        source_end: u32,
        key_start: u32,
        space: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if !prefix.is_formatted() {
            return self.format_source_token_gap(source_end, key_start, space, writer);
        }
        let comments = self
            .comments
            .between(Span::new(prefix.start(), key_start))?;
        self.comments.claim(comments)?;
        if space {
            writer.space();
        }
        self.format_comment_run(comments, CommentRunLayout::FlatLeading(key_start), writer)
    }

    pub(in crate::formatter) fn source_class_member_needs_field_separator(
        &self,
        span: Span,
        key: PropertyKey,
    ) -> bool {
        let source = self.source(span);
        source.starts_with(['[', '*'])
            || span.start == key.span(self.ast).start
                && matches!(
                    self.property_key_identifier(
                        key,
                        crate::formatter::expressions::objects::PropertyKeyContext::Ordinary,
                        crate::formatter::suppression::ContentMode::Verbatim,
                        crate::formatter::expressions::objects::PropertyKeyQuoting::Preserve
                    ),
                    Some("in" | "instanceof")
                )
    }
}
