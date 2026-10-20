pub mod ast;
mod diagnostics;
mod parser_impl;

pub use parser_impl::Parser;

#[cfg(test)]
mod test {
    use crate::parser::reader::{Options, template_literal_parser::parser_impl::Parser};

    #[test]
    fn templates() {
        let check_source = |source_text: &str| {
            assert!(Parser::new(source_text, Options::default()).parse().is_ok());
        };

        check_source("``");

        check_source("`Hello, world!`");

        check_source(r#"`He said, "Hello!"`"#);

        check_source(r#"`She said, "Hello!"`"#);

        check_source(r"`It's a sunny day`");

        check_source(r"`Line1\nLine2`");

        check_source(r"`Column1\tColumn2`");

        check_source(r"`Path to file: C:\\Program Files\\MyApp`");

        check_source(r"`Backspace\bTest`");

        check_source(r"`FormFeed\fTest`");

        check_source(r"`CarriageReturn\rTest`");

        check_source(r"`TestWithValidDollarSignAtTheEnd$`");

        check_source(r"`TestWithValid$DollarSignBetween`");

        check_source(r"`VerticalTab\vTest`");

        check_source("`   `");
        // whitespace only

        check_source(r"`Escaped \``");
        // escaped backtick

        check_source(r"`Unicode: \u0041`");
        // unicode escape

        check_source(r"`Code point: \u{1F600}`");
        // unicode code point escape

        check_source(r"`Hex: \x41`");
        // hex escape

        check_source("`Line1\\\nLine2`");
        // line continuation

        check_source("`Price: $100`");
        // dollar sign not followed by brace

        check_source(r"`Smile: \uD83D\uDE00`");
        // surrogate pair

        check_source(r"`Mix: \n\t\x41\u0042\u{43}`");
        // multiple escapes

        check_source(r"`Dollar: \$`");
        // escaped dollar

        check_source(r"`Null: \0`");
        // should be valid if not followed by a digit

        check_source(r"`Surrogate: \uD800`");
        // lone high surrogate, not a valid code point but no error is reported

        check_source(r"`Valid: \z`");

        check_source("`This is\n            a multi-line\n            template literal`");
    }

    #[test]
    fn invalid_templates() {
        let check_source = |source_text: &str| {
            let result = Parser::new(source_text, Options::default()).parse();
            assert!(result.is_err());
        };

        check_source("`Unclosed template literal");

        check_source(r"`Invalid hex escape: \xG1`");

        check_source(r"`Invalid unicode escape: \u{G1}`");

        check_source(r"`Template with ${expression}`");
        // expression not supported

        check_source(r"`Incomplete: \u`");

        check_source(r"`Incomplete: \u{}`");

        check_source(r"`Incomplete: \u{110000}`");
        // out of Unicode range

        check_source(r"`Incomplete: \x`");

        check_source(r"`Incomplete: \x4`");

        check_source("`Unescaped backtick: ``");

        check_source(r"`Line continuation: \\n\xG1`");

        check_source(r"`Dollar: ${{}`");
    }
}
