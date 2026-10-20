pub mod ast;
mod diagnostics;
mod parser_impl;

pub use parser_impl::{Parser, parse_regexp_literal};

#[cfg(test)]
mod test {
    use crate::parser::reader::Options;

    use super::{Parser, ast, parse_regexp_literal};

    #[test]
    fn strings() {
        let check_source = |source_text: &str| {
            assert!(Parser::new(source_text, Options::default()).parse().is_ok());
        };

        check_source(r#""""#);

        check_source(r"''");

        check_source(r#""Hello, world!""#);

        check_source(r"'Hello, world!'");

        check_source(r#""He said, \"Hello!\"""#);

        check_source(r#"'She said, "Hello!"'"#);

        check_source(r"'It\'s a sunny day'");

        check_source("'Line1\\nLine2'");

        check_source("'Column1\tColumn2'");

        check_source(r#""Path to file: C:\\Program Files\\MyApp""#);

        check_source(r"'Backspace\bTest'");

        check_source(r"'FormFeed\fTest'");

        check_source("'CarriageReturn\\rTest'");

        check_source(r"'VerticalTab\vTest'");

        check_source(r#""NullChar\0Test""#);

        check_source(r#""Hex A: \x41""#);

        check_source(r#""Unicode A: \u0041""#);

        check_source(r#""Smiley: 😀""#);

        check_source(r#""Smiley: \u{1F600}""#);

        check_source(r#""Octal 7: \7""#);

        check_source(r#""Octal 77: \77""#);

        check_source(r#""Octal 123: \123""#);

        check_source(r#""Non-octal digits: \8\9""#);

        check_source("\"This is a long string\\\n    that spans\\\n multiple lines\"");

        check_source(r"'Line separator:\u2028Test'");

        check_source(r"'Paragraph separator:\u2029Test'");

        check_source(r#""NonEscapeCharacter: \c""#);

        check_source(r#""Zero followed by 8 and 9: \08\09""#);

        check_source(r#""NonZeroOctalDigit not followed by OctalDigit: \1x""#);

        check_source(r#""ZeroToThree OctalDigit not followed by OctalDigit: \33x""#);

        check_source(r#""FourToSeven OctalDigit: \47""#);

        check_source(r#""Unicode brace escape: \u{0041}""#);

        check_source(r#""Escaped backslash and quote: \\\"""#);

        check_source(r#""Invalid escape: \@""#);

        check_source(r#""He said, 'Hello!'""#);

        check_source(r#"'She replied, "Hi!"'"#);

        check_source(r#""Multiple escapes: \n\t\r""#);

        check_source(r#""Longest octal escape: \377""#);

        check_source(r#""Number at end: \1234""#);

        check_source(r#""Escape followed by letter: \1a""#);

        check_source(
            "\"This is a long string that spans\\\n multiple lines using\\\n multiple continuations\"",
        );

        check_source(r#""Not an escape: \\g""#);

        check_source(r#""Surrogate pair: \uD83D\uDE00""#);

        check_source("\"Line continuation with terminator: \\\n\\r\\n\"");

        check_source(r#""Special chars: !@#$%^&*()_+-=[]{}|;':,.<>/?\"''""#);

        check_source(r#""Combined escapes: \n\\\"\t\u0041\x42""#);
    }

    #[test]
    fn invalid_strings() {
        let check_source = |source_text: &str| {
            let result = Parser::new(source_text, Options::default()).parse();
            assert!(result.is_err());
        };

        check_source(r"Not quoted");

        check_source(r"'Unterminated");

        check_source("\"Line terminator\nwithout continuation\"");

        check_source(r#""Invalid hex escape: \xG1""#);

        check_source(r#""Invalid escapes: \x\y\z""#);

        check_source(r#""Invalid unicode escape: \u00G1""#);

        check_source(r#""Invalid unicode brace escape: \u{G1}""#);

        check_source(r#""Too many digits: \u{1234567}""#);

        check_source(r#""str"+'str'"#);

        check_source(r#"'str'+"str""#);
    }

    #[test]
    fn strict_escapes() {
        let check_source = |source_text: &str| {
            // These are allowed in non-strict mode.
            let result = Parser::new(
                source_text,
                Options {
                    strict_mode: false,
                    ..Options::default()
                },
            )
            .parse();
            assert!(result.is_ok());

            // But not in strict mode.
            let result = Parser::new(
                source_text,
                Options {
                    strict_mode: true,
                    ..Options::default()
                },
            )
            .parse();
            assert!(result.is_err());
        };

        check_source(r#""invalid octal \777""#);

        check_source(r#""invalid non-octal decimal \9""#);
    }

    #[test]
    fn parse_quotes() {
        let options = Options::default();

        let ast = Parser::new(r#""double""#, options).parse().unwrap();
        assert_eq!(ast.kind, ast::StringLiteralKind::Double);

        let ast = Parser::new(r"'single'", options).parse().unwrap();
        assert_eq!(ast.kind, ast::StringLiteralKind::Single);
    }

    #[test]
    fn surrogate_pairs() {
        let source_text = "'👈🏻(=2+2)'";

        let ast = Parser::new(
            source_text,
            Options {
                combine_surrogate_pair: false,
                ..Options::default()
            },
        )
        .parse()
        .unwrap();
        assert_eq!(ast.body.len(), 10);

        let ast = Parser::new(
            source_text,
            Options {
                combine_surrogate_pair: true,
                ..Options::default()
            },
        )
        .parse()
        .unwrap();
        assert_eq!(ast.body.len(), 8);
    }

    #[test]
    fn span_offset() {
        let source_text = "\"Adjust span but should have no side effect for parsing\"";
        let ret1 = Parser::new(
            source_text,
            Options {
                span_offset: 0,
                ..Options::default()
            },
        )
        .parse()
        .unwrap();
        let ret2 = Parser::new(
            source_text,
            Options {
                span_offset: 10,
                ..Options::default()
            },
        )
        .parse()
        .unwrap();

        assert_ne!(ret1.span, ret2.span);
        for (a, b) in ret1.body.iter().zip(ret2.body.iter()) {
            assert_ne!(a.span, b.span);
        }
    }

    #[test]
    fn restore_span() {
        let source_text = "'123'";
        let ast = Parser::new(
            source_text,
            Options {
                span_offset: 0,
                combine_surrogate_pair: false,
                strict_mode: false,
            },
        )
        .parse()
        .unwrap();

        assert_eq!(ast.span.source_text(source_text), source_text);

        let source_text = "\"Hi,\\n🦄\\w\"";
        let ast = Parser::new(
            source_text,
            Options {
                span_offset: 0,
                combine_surrogate_pair: true,
                strict_mode: false,
            },
        )
        .parse()
        .unwrap();

        assert_eq!(ast.span.source_text(source_text), source_text);
        let mut units = ast.body.iter();
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"H");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"i");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r",");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"\n");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"🦄");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"\w");
        assert!(units.next().is_none());

        let source_text = "...'<-HERE->'...";
        let ast = Parser::new(
            &source_text[3..13],
            Options {
                span_offset: 3,
                ..Options::default()
            },
        )
        .parse()
        .unwrap();

        assert_eq!(ast.span.source_text(source_text), "'<-HERE->'");
        let mut units = ast.body.iter();
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"<");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"-");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"H");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"E");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"R");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"E");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r"-");
        assert_eq!(units.next().unwrap().span.source_text(source_text), r">");
        assert!(units.next().is_none());
    }

    #[test]
    fn regexp_literal() {
        let source_text1 = r"re = new RegExp('^12🥳3\\d(?=4)\\\\$')";
        let offset1 = (16, 39);

        let source_text2 = r"re = /^12🥳3\d(?=4)\\$/";
        let offset2 = (6, 24);

        let combine_surrogate_pair = false;

        let ret1 = Parser::new(
            &source_text1[offset1.0..offset1.1],
            Options {
                span_offset: u32::try_from(offset1.0).unwrap(),
                combine_surrogate_pair,
                strict_mode: false,
            },
        )
        .parse()
        .unwrap()
        .body;
        let ret2 = parse_regexp_literal(
            &source_text2[offset2.0..offset2.1],
            u32::try_from(offset2.0).unwrap(),
            combine_surrogate_pair,
        );

        assert_eq!(ret1.len(), ret2.len());
        for (a, b) in ret1.iter().zip(ret2.iter()) {
            assert_eq!(a.value, b.value);
        }
    }
}
