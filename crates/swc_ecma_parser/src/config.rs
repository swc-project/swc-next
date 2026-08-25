pub trait ParserConfig: Clone {
    fn tokens(&self) -> bool;
}

#[derive(Clone, Copy, Default)]
pub struct TokenParserConfig;

impl ParserConfig for TokenParserConfig {
    fn tokens(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Default)]
pub struct NoTokenParserConfig;

impl ParserConfig for NoTokenParserConfig {
    fn tokens(&self) -> bool {
        false
    }
}
