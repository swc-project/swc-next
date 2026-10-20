mod flags_parser;
mod parser_impl;
mod pattern_parser;
mod reader;
mod span_factory;

#[cfg(test)]
pub use parser_impl::ConstructorParser;
pub use parser_impl::LiteralParser;

#[cfg(test)]
mod test {
    use swc_next_allocator::Allocator;

    use crate::{ConstructorParser, LiteralParser, Options};

    fn parse_literal<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        flags: Option<&str>,
    ) -> crate::diagnostic::Result<crate::ast::Pattern<'a>> {
        LiteralParser::new(allocator, source, flags, Options::default()).parse()
    }

    #[test]
    fn decimal_backreferences() {
        use crate::ast::{CharacterKind, Term};

        let allocator = Allocator::default();

        let pattern = parse_literal(&allocator, r"()\1", Some("u")).unwrap();
        let terms = &pattern.body.body[0].body;
        assert!(matches!(&terms[1], Term::IndexedReference(reference) if reference.index == 1));

        let pattern = parse_literal(&allocator, r"()\4294967297", None).unwrap();
        let terms = &pattern.body.body[0].body;
        assert!(
            matches!(&terms[1], Term::Character(c) if c.value == 34 && c.kind == CharacterKind::Octal2)
        );
        assert_eq!(terms.len(), 10);

        let pattern = parse_literal(&allocator, r"()\18446744073709551616", None).unwrap();
        let terms = &pattern.body.body[0].body;
        assert!(
            matches!(&terms[1], Term::Character(c) if c.value == 1 && c.kind == CharacterKind::Octal1)
        );
        assert_eq!(terms.len(), 21);

        let pattern = parse_literal(&allocator, r"()\01", None).unwrap();
        let terms = &pattern.body.body[0].body;
        assert!(
            matches!(&terms[1], Term::Character(c) if c.value == 1 && c.kind == CharacterKind::Octal2)
        );

        let pattern = parse_literal(&allocator, r"\99999999999999999999", None).unwrap();
        let terms = &pattern.body.body[0].body;
        assert!(matches!(&terms[0], Term::Character(c) if c.value == u32::from(b'9')));
        assert_eq!(terms.len(), 20);

        let pattern = parse_literal(&allocator, r"()\18446744073709551616", Some("u"));
        assert!(pattern.is_err());

        let pattern = parse_literal(&allocator, r"()\18446744073709551616", Some("v"));
        assert!(pattern.is_err());

        let pattern = parse_literal(&allocator, r"()\4294967297", Some("u"));
        assert!(pattern.is_err());
    }

    #[test]
    fn patterns() {
        let allocator = Allocator::default();

        let check_case = |pattern_text: &str, flags_text: &str| {
            let res = LiteralParser::new(
                &allocator,
                pattern_text,
                Some(flags_text),
                Options::default(),
            )
            .parse();
            assert!(res.is_ok());
        };

        check_case("", "");

        check_case("a", "");

        check_case("a+", "");

        check_case("a*", "");

        check_case("a?", "");

        check_case("^$^$^$", "");

        check_case("(?=a){1}", "");

        check_case("(?!a){1}", "");

        check_case("a{1}", "");

        check_case("a{1", "");

        check_case("a|{", "");

        check_case("a{", "");

        check_case("a{,", "");

        check_case("a{1,", "");

        check_case("a{1,}", "");

        check_case("a{1,2}", "");

        check_case("x{9007199254740991}", "");

        check_case("x{9007199254740991,9007199254740991}", "");

        check_case("a|b", "");

        check_case("a|b|c", "");

        check_case("a|b+?|c", "");

        check_case("a+b*?c{1}d{2,}e{3,4}?", "");

        check_case(r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$", "");

        check_case("a.b..", "");

        check_case(r"\d\D\s\S\w\W", "");

        check_case(r"\x", "");

        check_case(
            r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{Basic_Emoji}",
            "",
        );

        check_case(
            r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}",
            "u",
        );

        check_case(r"^\p{General_Category=cntrl}+$", "u");

        check_case(r"\p{Basic_Emoji}", "v");

        check_case(r"\n\cM\0\x41\u1f60\.\/", "");

        check_case(r"\c0", "");

        check_case(r"\0", "");

        check_case(r"\0", "u");

        check_case(r"\u", "");

        check_case(r"\u{", "");

        check_case(r"\u{}", "");

        check_case(r"\u{0}", "");

        check_case(r"\u{1f600}", "");

        check_case(r"\u{1f600}", "u");

        check_case("(?:abc)", "");

        check_case(r"(?<\u{1d49c}>.)\x1f", "");

        check_case("a]", "");

        check_case("a}", "");

        check_case("]", "");

        check_case("[]", "");

        check_case("[a]", "");

        check_case("[ab]", "");

        check_case("[a-b]", "");

        check_case("[-]", "");

        check_case("[a-]", "");

        check_case("[-a]", "");

        check_case("[-a-]", "");

        check_case(r"[a\-b]", "");

        check_case(r"[-a-b]", "");

        check_case(r"[a-b-]", "");

        check_case(r"[a\-b-]", "");

        check_case(r"[\[\]\-]", "");

        check_case("[a-z0-9]", "");

        check_case("[a-a]", "");

        check_case(r"[\d-\D]", "");

        check_case(r"^([\ud801[\udc28-\udc4f])$", "");

        check_case(r"[a-c]]", "");

        check_case(
            r"[ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ]",
            "",
        );

        check_case(r"[a-z0-9[.\\]]", "v");

        check_case(r"[a&&b&&c]", "v");

        check_case(r"[a--b--c]", "v");

        check_case(r"[[a-z]--b--c]", "v");

        check_case(r"[[[[[[[[[[[[[[[[[[[[[[[[a]]]]]]]]]]]]]]]]]]]]]]]]", "v");

        check_case(r"[\q{}\q{a}\q{bc}\q{d|e|f}\q{|||}]", "v");

        check_case(r"(?<foo>A)\k<foo>", "");

        check_case(r"(?<!a>)\k<a>", "");

        check_case(r"\k", "");

        check_case(r"\k<4>", "");

        check_case(r"\k<a>", "");

        check_case(r"(?<a>)\k<a>", "");

        check_case(r"(?<a>)\k<a>", "u");

        check_case(r"\1", "");

        check_case(r"\1()", "");

        check_case(r"\1()", "u");

        check_case(r"(?<n1>..)(?<n2>..)", "");
        // ES2025 ---
        // Duplicate named capturing groups

        check_case(r"(?<n1>..)|(?<n1>..)", "");

        check_case(r"(?<year>[0-9]{4})-[0-9]{2}|[0-9]{2}-(?<year>[0-9]{4})", "");

        check_case(r"(?:(?<a>x)|(?<a>y))\k<a>", "");

        check_case(r"(?<x>a)|(?<x>b)", "");

        check_case(r"(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))", "");

        check_case(r"(?:(?<x>a)|(?<x>b))\\k<x>", "");

        check_case(r"(?:(?:(?<x>a)|(?<x>b)|c)\\k<x>){2}", "");

        check_case(r"(?:(?:(?<x>a)|(?<x>b))\\k<x>){2}", "");

        check_case(r"(?:(?:(?<x>a)\\k<x>|(?<x>b)\\k<x>)|(?:))\\k<x>", "");

        check_case(r"(?:(?:(?<x>a\\k<x>)|(?<x>b\\k<x>))|(?:))\\k<x>", "");
        // Modifiers

        check_case(r"(?:.)", "");

        check_case(r"(?s:.)", "");

        check_case(r"(?ism:.)", "");

        check_case(r"(?-s:.)", "");

        check_case(r"(?-smi:.)", "");

        check_case(r"(?s-im:.)", "");

        check_case(r"(?si-m:.)", "");

        check_case(r"(?im-s:.)", "v");

        check_case(r"(?ims-:.)", "");
    }

    #[test]
    fn invalid_patterns() {
        let allocator = Allocator::default();

        let check_case = |pattern_text: &str, flags_text: &str| {
            assert!(
                LiteralParser::new(
                    &allocator,
                    pattern_text,
                    Some(flags_text),
                    Options::default()
                )
                .parse()
                .is_err()
            );
        };

        check_case("a)", "");

        check_case(r"a\", "");

        check_case("a]", "u");

        check_case("a}", "u");

        check_case("a|+", "");

        check_case("a|{", "u");

        check_case("a{", "u");

        check_case("a{1", "u");

        check_case("a{1,", "u");

        check_case("a{,", "u");

        check_case("x{9007199254740992}", "");

        check_case("x{9007199254740991,9007199254740992}", "");

        check_case("x{99999999999999999999999999999999999999999999999999}", "");

        check_case(r"\99999999999999999999999999999999999999999999999999", "u");

        check_case(r"\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}", "u");

        check_case("(?=a", "");

        check_case("(?<!a", "");

        check_case(r"\c0", "u");

        check_case(r"\xa", "u");

        check_case(r"a\u", "u");

        check_case(r"\p{Emoji_Presentation", "u");

        check_case(r"\p{Script=", "u");

        check_case(r"\ka", "u");

        check_case(r"\k", "u");

        check_case(r"\k<", "u");

        check_case(r"\k<>", "u");

        check_case(r"\k<4>", "u");

        check_case(r"\k<a", "u");

        check_case(r"\1", "u");

        check_case(r"\k<a>", "u");

        check_case("a(?:", "");

        check_case("(", "");

        check_case(")", "v");

        check_case("(a", "");

        check_case("(?<a>", "");

        check_case("(?<", "");

        check_case(r"(?<a\>.)", "");

        check_case(r"(?<a\>.)", "u");

        check_case(r"(?<\>.)", "");

        check_case(r"(?<\>.)", "u");

        check_case("(?)", "");

        check_case("(?=a){1}", "u");

        check_case("(?!a){1}", "u");

        check_case(r"[\d-\D]", "u");

        check_case("[", "");

        check_case("[", "v");

        check_case("[[", "v");

        check_case("[[]", "v");

        check_case("[z-a]", "");

        check_case(r"[a-c]]", "u");

        check_case(
            r"^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ-ώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐ-ΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$",
            "",
        );

        check_case(r"[[\d-\D]]", "v");

        check_case(r"[a&&b--c]", "v");

        check_case(r"[a--b&&c]", "v");

        check_case(r"[\q{]", "v");

        check_case(r"[\q{\a}]", "v");
        // ES2025 ---
        // Duplicate named capturing groups

        check_case(r"(?<n>.)(?<n>.)", "");

        check_case(r"(?<n>.(?<n>..))", "u");

        check_case("(?<n>)|(?<n>)(?<n>)", "");

        check_case("(((((((?<n>.)))))))(?<n>)", "");

        check_case("(?:(?<x>a)|(?<x>b))(?<x>c)", "");

        check_case("(?<x>a)(?:(?<x>b)|(?<x>c))", "");

        check_case("(?:(?:(?<x>a)|(?<x>b)))(?<x>c)", "");

        check_case("(?:(?:(?<x>a)|(?<x>b))|(?:))(?<x>c)", "");
        // Modifiers

        check_case(r"(?a:.)", "");

        check_case(r"(?-S:.)", "");

        check_case(r"(?-:.)", "");

        check_case(r"(?iM:.)", "");

        check_case(r"(?imms:.)", "");

        check_case(r"(?-sI:.)", "");

        check_case(r"(?ii-s:.)", "");

        check_case(r"(?i-msm:.)", "");

        check_case(r"(?i", "");

        check_case(r"(?i-", "");

        check_case(r"(?i-s", "");
    }

    #[test]
    fn early_errors() {
        let allocator = Allocator::default();

        let check_case = |pattern_text: &str, flags_text: &str, is_err: bool| {
            assert_eq!(
                LiteralParser::new(
                    &allocator,
                    pattern_text,
                    Some(flags_text),
                    Options::default()
                )
                .parse()
                .is_err(),
                is_err
            );
        };
        // No tests for 4,294,967,295 left parens

        check_case(r"(?<n>..)(?<n>..)", "", true);

        check_case(r"a{2,1}", "", true);

        check_case(r"(?<a>)\k<n>", "", true);

        check_case(r"()\2", "u", true);

        check_case(r"[a-\d]", "u", true);

        check_case(r"[\d-z]", "u", true);

        check_case(r"[\d-\d]", "u", true);

        check_case(r"[z-a]", "", true);

        check_case(r"\u{110000}", "u", true);

        check_case(r"(?<\uD800\uDBFF>)", "", true);

        check_case(r"\u{0}\u{110000}", "u", true);

        check_case(r"(?<a\uD800\uDBFF>)", "", true);

        check_case(r"\p{Foo=Bar}", "u", true);

        check_case(r"\p{Foo}", "u", true);

        check_case(r"\p{Basic_Emoji}", "u", true);

        check_case(r"\P{Basic_Emoji}", "v", true);

        check_case(r"[^\p{Basic_Emoji}]", "v", true);

        check_case(r"[[^\p{Basic_Emoji}]]", "v", true);

        check_case(r"[^\q{}]", "v", true);

        check_case(r"[[^\q{}]]", "v", true);

        check_case(r"[[^\q{ng}]]", "v", true);

        check_case(r"[[^\q{a|}]]", "v", true);

        check_case(r"[[^\q{ng}\q{o|k}]]", "v", true);

        check_case(r"[[^\q{o|k}\q{ng}\q{o|k}]]", "v", true);

        check_case(r"[[^\q{o|k}\q{o|k}\q{ng}]]", "v", true);

        check_case(r"[[^\q{}&&\q{ng}]]", "v", true);

        check_case(r"[[^\q{ng}&&\q{o|k}]]", "v", false);

        check_case(r"[[^\q{ng}&&\q{o|k}&&\q{ng}]]", "v", false);

        check_case(r"[[^\q{ng}--\q{o|k}]]", "v", true);

        check_case(r"[[^\q{o|k}--\q{ng}]]", "v", false);

        check_case(r"[[z-a]]", "v", true);

        check_case(r"[[[[[^[[[[\q{ng}]]]]]]]]]", "v", true);

        check_case(r"[^[[[[[[[[[[[[[[[[\q{ng}]]]]]]]]]]]]]]]]]", "v", true);
        // ES2025 ---
        // Duplicated named capture groups

        check_case("(?:(?<x>a)|(?<x>b))(?<x>c)", "", true);

        check_case("(?:(?<x>a)|(?<x>b))(?<X>c)", "", false);

        check_case("(?<x>a)(?:(?<x>b)|(?<x>c))", "", true);

        check_case("(?<x>a)|(?:(?<x>b)|(?<x>c))", "", false);
        // Modifiers

        check_case(r"(?ii:.)", "", true);

        check_case(r"(?-ss:.)", "", true);

        check_case(r"(?im-im:.)", "", true);
    }

    #[test]
    fn empty_patterns() {
        let allocator = Allocator::default();
        let pattern1 = LiteralParser::new(&allocator, "", None, Options::default())
            .parse()
            .unwrap();
        let pattern2 = ConstructorParser::new(&allocator, "''", None, Options::default())
            .parse()
            .unwrap();

        assert_eq!(pattern1.body.body[0].body.len(), 1);
        assert_eq!(pattern2.body.body[0].body.len(), 1);
    }

    #[test]
    fn unicode() {
        let allocator = Allocator::default();
        let source_text = "このEmoji🥹の数が変わる";

        for (flags_text, expected) in [(None, 15), (Some("u"), 14), (Some("v"), 14)] {
            let pattern =
                LiteralParser::new(&allocator, source_text, flags_text, Options::default())
                    .parse()
                    .unwrap();
            assert_eq!(pattern.body.body[0].body.len(), expected);
        }
    }

    #[test]
    fn span_offset() {
        let allocator = Allocator::default();

        let pattern_text = "Adjust span but should have no side effect for parsing";
        let ret1 = LiteralParser::new(
            &allocator,
            pattern_text,
            None,
            Options {
                pattern_span_offset: 0,
                flags_span_offset: 0,
            },
        )
        .parse()
        .unwrap();
        let ret2 = LiteralParser::new(
            &allocator,
            pattern_text,
            None,
            Options {
                pattern_span_offset: 123,
                flags_span_offset: 456,
            },
        )
        .parse()
        .unwrap();

        assert_ne!(ret1.span, ret2.span);
        assert_eq!(ret1.to_string(), ret2.to_string());
    }

    #[test]
    fn string_literal() {
        let allocator = Allocator::default();

        let source_text = r"RegExp('Invalid! -> \u{1234568} <-')";
        let err = ConstructorParser::new(
            &allocator,
            &source_text[7..35],
            None,
            Options {
                pattern_span_offset: 7,
                ..Options::default()
            },
        )
        .parse();
        assert!(err.is_err());

        let ret1 = LiteralParser::new(
            &allocator,
            r"\d{4}-\d{2}-\d{2}",
            Some("vi"),
            Options::default(),
        )
        .parse()
        .unwrap();
        let ret2 = ConstructorParser::new(
            &allocator,
            r"'\\d{4}-\\d{2}-\\d{2}'",
            Some("'vi'"),
            Options::default(),
        )
        .parse()
        .unwrap();
        assert_eq!(ret1.to_string(), ret2.to_string());
    }

    /// Test for <https://github.com/oxc-project/oxc/issues/13660>
    /// Unicode escapes in string literals should be detected as CharacterKind::UnicodeEscape
    #[test]
    fn unicode_escape_in_string_literal() {
        use crate::ast::{CharacterClassContents, CharacterKind, Term};

        let allocator = Allocator::default();

        // RegExp literal: /[A\u0301]/
        let pattern1 = LiteralParser::new(&allocator, r"[A\u0301]", None, Options::default())
            .parse()
            .unwrap();

        // RegExp constructor: RegExp("[A\u0301]") - the string literal has the escape resolved
        let pattern2 =
            ConstructorParser::new(&allocator, r#""[A\u0301]""#, None, Options::default())
                .parse()
                .unwrap();

        // Extract the character class from both patterns
        let get_char_kinds = |pattern: &crate::ast::Pattern<'_>| -> Vec<CharacterKind> {
            let term = &pattern.body.body[0].body[0];
            if let Term::CharacterClass(cc) = term {
                cc.body
                    .iter()
                    .filter_map(|item| {
                        if let CharacterClassContents::Character(c) = item {
                            Some(c.kind)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        };

        let kinds1 = get_char_kinds(&pattern1);
        let kinds2 = get_char_kinds(&pattern2);

        // Both should have: Symbol (A), UnicodeEscape (\u0301)
        assert_eq!(
            kinds1,
            vec![CharacterKind::Symbol, CharacterKind::UnicodeEscape]
        );
        assert_eq!(
            kinds2,
            vec![CharacterKind::Symbol, CharacterKind::UnicodeEscape]
        );
    }
}
