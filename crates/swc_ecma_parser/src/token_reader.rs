use swc_next_allocator::Allocator;
use swc_next_ecma_ast::SourceType;

use crate::{NoTokenParserConfig, Token, TokenTag, lexer::Lexer};

/// Reads single tokens on demand without parsing or collecting a token stream.
///
/// Tokens are scanned without parser context: `/` is always a division
/// punctuator, `}` never continues a template, and `>>` is never split. Comments
/// are skipped and not collected.
pub struct TokenReader<'a> {
    lexer: Lexer<'a, NoTokenParserConfig>,
}

impl<'a> TokenReader<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str, source_type: SourceType) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        Self {
            lexer: Lexer::init(
                allocator,
                source,
                source_type != SourceType::Module,
                false,
                NoTokenParserConfig,
            ),
        }
    }

    /// Reads the first token at or after `position`, skipping trivia.
    ///
    /// `position` must not be inside a token or comment. The token reports a
    /// line terminator before it if the skipped trivia contains one. Returns
    /// `None` at the end of the source or on a lexical error.
    pub fn read(&mut self, position: u32) -> Option<Token> {
        self.lexer.rewind_to(position);
        let mut token = Token::eof(position);
        self.lexer.next_token(&mut token).ok()?;
        (token.tag() != TokenTag::Eof).then_some(token)
    }
}
