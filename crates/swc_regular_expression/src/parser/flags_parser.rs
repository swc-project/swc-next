use crate::diagnostic::Result;
use bitflags::bitflags;

use crate::{
    diagnostics,
    parser::{reader::Reader, span_factory::SpanFactory},
};

bitflags! {
    /// Regular expression flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u8 {
        /// Indices flag
        const D = 1 << 0;
        /// Global flag
        const G = 1 << 1;
        /// Ignore case flag
        const I = 1 << 2;
        /// Multiline flag
        const M = 1 << 3;
        /// DotAll flag
        const S = 1 << 4;
        /// Unicode flag
        const U = 1 << 5;
        /// Unicode sets flag
        const V = 1 << 6;
        /// Sticky flag
        const Y = 1 << 7;
    }
}

impl TryFrom<char> for Flags {
    type Error = ();

    fn try_from(c: char) -> std::result::Result<Self, Self::Error> {
        match c {
            'd' => Ok(Self::D),
            'g' => Ok(Self::G),
            'i' => Ok(Self::I),
            'm' => Ok(Self::M),
            's' => Ok(Self::S),
            'u' => Ok(Self::U),
            'v' => Ok(Self::V),
            'y' => Ok(Self::Y),
            _ => Err(()),
        }
    }
}

pub struct FlagsParser<'a> {
    reader: Reader<'a>,
    span_factory: SpanFactory,
}

impl<'a> FlagsParser<'a> {
    pub fn new(reader: Reader<'a>, span_offset: u32) -> Self {
        Self {
            reader,
            span_factory: SpanFactory::new(span_offset),
        }
    }

    /// Returns: (is_unicode_mode, is_unicode_sets_mode)
    pub fn parse(mut self) -> Result<(bool, bool)> {
        let mut is_unicode_mode = false;
        let mut is_unicode_sets_mode = false;
        let mut seen = Flags::empty();

        while let Some(cp) = self.reader.peek() {
            let span_start = self.reader.offset();
            self.reader.advance();
            let span_end = self.reader.offset();

            let flag = char::try_from(cp)
                .ok()
                .and_then(|c| Flags::try_from(c).ok())
                .ok_or_else(|| {
                    diagnostics::unknown_flag(
                        self.span_factory.create(span_start, span_end),
                        self.reader.str(span_start, span_end),
                        &["d", "g", "i", "m", "s", "u", "v", "y"],
                    )
                })?;

            if seen.contains(flag) {
                return Err(diagnostics::duplicated_flags(
                    self.span_factory.create(span_start, span_end),
                    self.reader.str(span_start, span_end),
                ));
            }

            if flag == Flags::U {
                if seen.contains(Flags::V) {
                    return Err(diagnostics::invalid_unicode_flags(
                        self.span_factory.create(span_start, span_end),
                    ));
                }
                is_unicode_mode = true;
            } else if flag == Flags::V {
                if seen.contains(Flags::U) {
                    return Err(diagnostics::invalid_unicode_flags(
                        self.span_factory.create(span_start, span_end),
                    ));
                }
                is_unicode_mode = true;
                is_unicode_sets_mode = true;
            }

            seen.insert(flag);
        }

        Ok((is_unicode_mode, is_unicode_sets_mode))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flags() {
        let check_case = |flags_text: &str, expected: (bool, bool)| {
            let reader = Reader::initialize(flags_text, true, false).unwrap();
            let result = FlagsParser::new(reader, 0).parse().unwrap();
            assert_eq!(result, expected);
        };

        check_case("", (false, false));

        check_case("i", (false, false));

        check_case("u", (true, false));

        check_case("v", (true, true));

        check_case("vg", (true, true));
    }

    #[test]
    fn invalid_flags() {
        let check_case = |flags_text| {
            let reader = Reader::initialize(flags_text, true, false).unwrap();
            let err = FlagsParser::new(reader, 0).parse();
            assert!(err.is_err());
        };

        check_case("uv");

        check_case("vu");

        check_case("uu");

        check_case("vv");

        check_case("gg");

        check_case("$");
        let check_case = |flags_text| {
            let reader = Reader::initialize(flags_text, true, true).unwrap();
            let err = FlagsParser::new(reader, 0).parse();
            assert!(err.is_err());
        };

        check_case(r#""uv""#);

        check_case("'V'");

        check_case("\"-\"");

        check_case(r#""\162""#);
    }

    #[test]
    fn string_flags() {
        let check_reader = |reader: Reader<'_>| {
            let result = FlagsParser::new(reader, 0).parse().unwrap();
            assert_eq!(result, (true, false));
        };

        check_reader(Reader::initialize("u", true, false).unwrap());

        check_reader(Reader::initialize("'u'", true, true).unwrap());

        check_reader(Reader::initialize(r#""\165""#, true, true).unwrap());

        check_reader(Reader::initialize(r#""\x75""#, true, true).unwrap());

        check_reader(Reader::initialize(r#""\u0075""#, true, true).unwrap());

        check_reader(Reader::initialize(r#""\u{0075}""#, true, true).unwrap());
    }
}
