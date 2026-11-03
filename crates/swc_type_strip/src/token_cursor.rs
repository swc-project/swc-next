use swc_next_allocator::Allocator;
#[cfg(debug_assertions)]
use swc_next_allocator::vec::ArenaVec;
use swc_next_ecma_ast::Span;
use swc_next_ecma_parser::{Options as ParserOptions, Token, TokenReader, TokenTag};
#[cfg(debug_assertions)]
use swc_next_ecma_parser::{Parser, TokenParserConfig};

/// Reads tokens on demand at token boundaries, typically the ends and starts of
/// AST nodes.
///
/// Tokens are read without parser context, so a read must not depend on it:
/// each read starts at a token boundary, and scans cover only gaps between AST
/// nodes that hold punctuators, keywords, and trivia. Such gaps never contain
/// regular expressions, template continuations, or `>` tokens that the parser
/// splits. A `}` that continues a template reads as `}`, so no read may look
/// for `}` there. Debug builds check every read against the parser's tokens.
pub(crate) struct TokenCursor<'a> {
    source: &'a str,
    reader: TokenReader<'a>,
    /// The parser's tokens, if the source parses without diagnostics.
    #[cfg(debug_assertions)]
    parser_tokens: Option<ArenaVec<'a, Token>>,
}

impl<'a> TokenCursor<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        Self {
            source,
            reader: TokenReader::new(allocator, source, options.source_type),
            #[cfg(debug_assertions)]
            parser_tokens: {
                let parse = Parser::init(allocator, source, options, TokenParserConfig).parse();
                parse.diagnostics.is_empty().then_some(parse.tokens)
            },
        }
    }

    fn read(&mut self, position: u32) -> Option<Token> {
        let token = self.reader.read(position);
        #[cfg(debug_assertions)]
        self.check_read(position, token);
        token
    }

    /// Asserts that `token` is the parser's first token at or after `position`.
    #[cfg(debug_assertions)]
    fn check_read(&self, position: u32, token: Option<Token>) {
        let Some(tokens) = &self.parser_tokens else {
            return;
        };
        let index = tokens.partition_point(|token| token.start() < position);
        let expected = tokens
            .get(index)
            .copied()
            .filter(|token| token.tag() != TokenTag::Eof);
        let (Some(token), Some(expected)) = (token, expected) else {
            assert_eq!(
                token, expected,
                "context-free token read at {position} differs from the parser's token"
            );
            return;
        };

        let continues_template = token.tag() == TokenTag::RightBrace
            && matches!(
                expected.tag(),
                TokenTag::TemplateMiddle | TokenTag::TemplateTail
            );
        if !continues_template {
            assert_eq!(
                (token.span(), token.tag()),
                (expected.span(), expected.tag()),
                "context-free token read at {position} differs from the parser's token"
            );
        }
        assert_eq!(token.start(), expected.start());

        // The line terminator flag covers only the trivia after `position`.
        let previous_end = index.checked_sub(1).map_or(0, |index| tokens[index].end());
        if previous_end == position {
            assert_eq!(
                token.has_line_terminator_before(),
                expected.has_line_terminator_before(),
                "context-free token read at {position} differs in its line terminator flag"
            );
        }
    }

    /// Returns the first token after `start` if it ends at or before `end`.
    pub(crate) fn first_in_range(&mut self, start: u32, end: u32) -> Option<Token> {
        self.read(start).filter(|token| token.end() <= end)
    }

    pub(crate) fn at_or_after(&mut self, position: u32) -> Option<Token> {
        self.read(position)
    }

    /// Returns the first `tag` token in the token sequence from `start` to `end`.
    pub(crate) fn find_forward(&mut self, start: u32, end: u32, tag: TokenTag) -> Option<Token> {
        let mut position = start;
        loop {
            let token = self.first_in_range(position, end)?;
            if token.tag() == tag {
                return Some(token);
            }
            position = token.end();
        }
    }

    /// Returns whether the last token of `span` is the single-character `byte`
    /// punctuator.
    pub(crate) fn ends_with_punctuator(&self, span: Span, byte: u8) -> bool {
        span.start < span.end && self.source.as_bytes()[span.end as usize - 1] == byte
    }

    pub(crate) fn has_line_terminator(&self, start: u32, end: u32) -> bool {
        self.source[start as usize..end as usize].contains(['\n', '\r', '\u{2028}', '\u{2029}'])
    }

    pub(crate) fn trailing_character_span(&self, end: u32) -> Option<Span> {
        let prefix = self.source.get(..end as usize)?;
        let (start, _) = prefix.char_indices().next_back()?;
        Some(Span::new(start as u32, end))
    }

    pub(crate) fn leading_character_span(&self, start: u32) -> Option<Span> {
        let character = self.source.get(start as usize..)?.chars().next()?;
        Some(Span::new(start, start + character.len_utf8() as u32))
    }
}

#[cfg(test)]
mod tests {
    use swc_next_ecma_ast::SourceType;

    use super::*;
    use crate::{Lang, Options};

    fn module_options() -> ParserOptions {
        Options {
            source_type: SourceType::Module,
            lang: Lang::Ts,
        }
        .into()
    }

    fn find(source: &str, start: u32, tag: TokenTag) -> Option<Span> {
        let allocator = Allocator::default();
        let mut cursor = TokenCursor::new(&allocator, source, module_options());
        cursor
            .find_forward(start, source.len() as u32, tag)
            .map(Token::span)
    }

    fn next(source: &str, start: u32) -> Option<(Span, bool)> {
        let allocator = Allocator::default();
        let mut cursor = TokenCursor::new(&allocator, source, module_options());
        cursor
            .at_or_after(start)
            .map(|token| (token.span(), token.has_line_terminator_before()))
    }

    #[test]
    fn find_forward_skips_trivia() {
        assert_eq!(
            find("a /* ? */ ?: T", 1, TokenTag::Question),
            Some(Span::new(10, 11))
        );

        assert_eq!(
            find("a // , }\n, b", 1, TokenTag::Comma),
            Some(Span::new(9, 10))
        );

        assert_eq!(
            find("a /* } */ }", 1, TokenTag::RightBrace),
            Some(Span::new(10, 11))
        );

        assert_eq!(find("a ?.b", 1, TokenTag::Question), None);

        assert_eq!(find("x != y", 1, TokenTag::LogicalNot), None);
    }

    #[test]
    fn find_forward_reads_whole_tokens() {
        assert_eq!(
            find("{ 'a,}' as b, c }", 7, TokenTag::Comma),
            Some(Span::new(12, 13))
        );

        assert_eq!(
            find("{ a, 'b}' }", 4, TokenTag::RightBrace),
            Some(Span::new(10, 11))
        );

        assert_eq!(
            find("readonlyx readonly", 0, TokenTag::Readonly),
            Some(Span::new(10, 18))
        );

        assert_eq!(
            find("re\\u0061donly x", 0, TokenTag::Readonly),
            Some(Span::new(0, 13))
        );
    }

    #[test]
    fn find_forward_respects_end() {
        let allocator = Allocator::default();
        let source = "a , b";
        let mut cursor = TokenCursor::new(&allocator, source, module_options());

        assert!(cursor.find_forward(1, 2, TokenTag::Comma).is_none());
    }

    #[test]
    fn at_or_after_reports_line_terminators() {
        assert_eq!(next("a /* \n */ (", 1), Some((Span::new(10, 11), true)));

        assert_eq!(next("a /* */ (", 1), Some((Span::new(8, 9), false)));

        assert_eq!(next("a // x\n[", 1), Some((Span::new(7, 8), true)));

        assert_eq!(next("a \u{2028}`b`", 1), Some((Span::new(5, 8), true)));

        assert_eq!(next("a   ", 1), None);
    }
}
