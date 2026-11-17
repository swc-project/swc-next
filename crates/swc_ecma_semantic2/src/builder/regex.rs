//! Independent implementation of ECMA-262 §22.2.1 and Annex B.1.2.
//!
//! Grammar: <https://tc39.es/ecma262/multipage/text-processing.html#sec-patterns>
//! Legacy extensions: <https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-regular-expressions-patterns>
//! Unicode property values: Unicode 17.0.0 PropertyValueAliases.txt.
//!
//! Grammar functions consume Chumsky input or compose the named productions. Only
//! synthesized attributes needed by early errors are retained, never an AST.
//! `custom` handles prefix dispatch, inherited grammar parameters and early
//! errors; lexical sequences use Chumsky's ordinary combinators.

use std::cmp::Ordering;

use chumsky::{
    input::InputRef,
    prelude::*,
    recursive::{Direct, Recursive},
};
use rustc_hash::FxHashMap;

#[derive(Clone, Copy, Default)]
struct Mode {
    unicode: bool,
    sets: bool,
    named: bool,
}

struct CaptureSite {
    path: Vec<(usize, usize)>,
}

#[derive(Default)]
struct Validation {
    captures: usize,
    names: FxHashMap<String, Option<CaptureSite>>,
    max_numeric_reference: usize,
    path: Vec<(usize, usize)>,
    next_disjunction: usize,
    duplicate: bool,
}

impl Validation {
    fn declare_name(&mut self, name: String) {
        let previous = self.names.entry(name).or_default();
        if let Some(previous) = previous {
            // Traversal never revisits an earlier alternative, so the latest
            // site suffices while previous sites are mutually exclusive.
            let exclusive = self
                .path
                .iter()
                .zip(&previous.path)
                .take_while(|((id, _), (other_id, _))| id == other_id)
                .any(|((_, branch), (_, other_branch))| branch != other_branch);
            self.duplicate |= !exclusive;
            previous.path.clone_from(&self.path);
        } else {
            *previous = Some(CaptureSite {
                path: self.path.clone(),
            });
        }
    }
}

type Extra = extra::Full<EmptyErr, extra::SimpleState<Validation>, Mode>;
type Input<'src, 'parse> = InputRef<'src, 'parse, &'src str, Extra>;
type Parsed<T> = Result<T, EmptyErr>;
type Subpattern<'src> = Recursive<Direct<'src, 'src, &'src str, (), Extra>>;
type Class<'src> = Recursive<Direct<'src, 'src, &'src str, ClassValue, Extra>>;

// Structural productions share Chumsky's zero-size failure signal.
fn production<'src, O, F>(f: F) -> impl Parser<'src, &'src str, O, Extra> + Clone
where
    F: for<'parse> Fn(&mut Input<'src, 'parse>) -> Parsed<O> + Clone,
{
    custom(f)
}

fn eat(input: &mut Input<'_, '_>, ch: char) -> bool {
    if input.peek() == Some(ch) {
        input.next();
        true
    } else {
        false
    }
}

fn starts_with(input: &mut Input<'_, '_>, text: &str) -> bool {
    input.slice_from(&input.cursor()..).starts_with(text)
}

/// Validates literal pattern/flag text using the independently implemented
/// ECMA-262 grammar. The caller handles literal delimiters and line terminators.
/// No pattern AST or diagnostics are produced.
pub(super) fn validate_literal(pattern_text: &str, flag_text: &str) -> bool {
    let Ok(mode) = regular_expression_flags().parse(flag_text).into_result() else {
        return false;
    };
    let mut state = extra::SimpleState(Validation::default());
    if pattern(mode)
        .parse_with_state(pattern_text, &mut state)
        .has_errors()
    {
        return false;
    }
    // Annex B selects +NamedCaptureGroups if the pattern declares any names.
    if !mode.named && !state.names.is_empty() {
        state = extra::SimpleState(Validation::default());
        if pattern(Mode {
            named: true,
            ..mode
        })
        .parse_with_state(pattern_text, &mut state)
        .has_errors()
        {
            return false;
        }
    }
    !state.duplicate
        && state.max_numeric_reference <= state.captures
        && state.names.values().all(Option::is_some)
}

/// ```text
/// RegularExpressionFlags :: [empty] | RegularExpressionFlags IdentifierPart
/// ```
/// §22.2.3.2: only d g i m s u v y; no duplicates; u and v are exclusive.
fn regular_expression_flags<'src>() -> impl Parser<'src, &'src str, Mode, extra::Err<EmptyErr>> {
    custom(
        |input: &mut InputRef<'src, '_, &'src str, extra::Err<EmptyErr>>| {
            let mut seen = 0u8;
            while let Some(ch) = input.next() {
                let bit = match ch {
                    'd' => 1,
                    'g' => 2,
                    'i' => 4,
                    'm' => 8,
                    's' => 16,
                    'u' => 32,
                    'v' => 64,
                    'y' => 128,
                    _ => {
                        return Err(EmptyErr::default());
                    }
                };
                if seen & bit != 0 {
                    return Err(EmptyErr::default());
                }
                seen |= bit;
                if seen & 96 == 96 {
                    return Err(EmptyErr::default());
                }
            }
            Ok(Mode {
                unicode: seen & 96 != 0,
                sets: seen & 64 != 0,
                named: seen & 96 != 0,
            })
        },
    )
}

/// ```text
/// Pattern[U,V,N] :: Disjunction[?U,?V,?N]
/// ```
/// Capture/reference early errors are checked after the entire production.
fn pattern<'src>(
    mode: Mode,
) -> impl Parser<'src, &'src str, (), extra::Full<EmptyErr, extra::SimpleState<Validation>, ()>> {
    disjunction()
        .then_ignore(production(|input: &mut Input<'src, '_>| {
            match input.peek() {
                None => Ok(()),
                Some(')') => Err(EmptyErr::default()),
                _ => Err(EmptyErr::default()),
            }
        }))
        .with_ctx(mode)
}

/// ```text
/// Disjunction[U,V,N] :: Alternative[?U,?V,?N]
///                    | Alternative[?U,?V,?N] | Disjunction[?U,?V,?N]
/// ```
/// Unique disjunction IDs distinguish alternatives in sibling groups.
fn disjunction<'src>() -> Subpattern<'src> {
    recursive(|subpattern: Subpattern<'src>| {
        production(move |input: &mut Input<'src, '_>| {
            let id = input.state().next_disjunction;
            input.state().next_disjunction += 1;
            input.state().path.push((id, 0));
            alternative(input, &subpattern)?;
            while eat(input, '|') {
                input.state().path.last_mut().unwrap().1 += 1;
                alternative(input, &subpattern)?;
            }
            input.state().path.pop();
            Ok(())
        })
    })
}

/// ```text
/// Alternative[U,V,N] :: [empty] | Alternative[?U,?V,?N] Term[?U,?V,?N]
/// ```
fn alternative<'src>(input: &mut Input<'src, '_>, subpattern: &Subpattern<'src>) -> Parsed<()> {
    while !matches!(input.peek(), None | Some('|' | ')')) {
        term(input, subpattern)?;
    }
    Ok(())
}

/// ```text
/// Term[U,V,N] :: Assertion[?U,?V,?N]
///              | Atom[?U,?V,?N] Quantifier?
/// Annex B: [~U] QuantifiableAssertion[?N] Quantifier?
///              | ExtendedAtom[?N] Quantifier?
/// ```
fn term<'src>(input: &mut Input<'src, '_>, subpattern: &Subpattern<'src>) -> Parsed<()> {
    let assertion_prefix = matches!(input.peek(), Some('^' | '$'))
        || starts_with(input, "\\b")
        || starts_with(input, "\\B")
        || starts_with(input, "(?=")
        || starts_with(input, "(?!")
        || starts_with(input, "(?<=")
        || starts_with(input, "(?<!");
    let quantifiable = if assertion_prefix {
        assertion(input, subpattern)?
    } else {
        if input.ctx().unicode {
            atom(input, subpattern)?;
        } else {
            extended_atom(input, subpattern)?;
        }
        true
    };
    let checkpoint = input.save();

    match input.peek() {
        Some('*' | '+' | '?') => {
            if !quantifiable {
                return Err(EmptyErr::default());
            }
            quantifier(input)?;
        }
        Some('{') => {
            if quantifier(input)? {
                if !quantifiable {
                    return Err(EmptyErr::default());
                }
            } else if input.ctx().unicode {
                return Err(EmptyErr::default());
            } else {
                input.rewind(checkpoint);
            }
        }
        _ => (),
    }
    Ok(())
}

/// ```text
/// Assertion[U,V,N] :: ^ | $ | \b | \B
///   | (?= Disjunction[?U,?V,?N] ) | (?! Disjunction[?U,?V,?N] )
///   | (?<= Disjunction[?U,?V,?N] ) | (?<! Disjunction[?U,?V,?N] )
/// Annex B: [~U] QuantifiableAssertion[?N]
/// ```
/// The synthesized boolean permits only Annex B lookahead quantification.
fn assertion<'src>(input: &mut Input<'src, '_>, subpattern: &Subpattern<'src>) -> Parsed<bool> {
    if eat(input, '^') || eat(input, '$') {
        return Ok(false);
    }
    if starts_with(input, "\\b") || starts_with(input, "\\B") {
        input.parse(just('\\').then(one_of("bB")))?;
        return Ok(false);
    }
    if !input.ctx().unicode && (starts_with(input, "(?=") || starts_with(input, "(?!")) {
        quantifiable_assertion(input, subpattern)?;
        return Ok(true);
    }

    input.parse(just("(?"))?;
    eat(input, '<');
    input.parse(one_of("=!"))?;
    input.parse(subpattern)?;
    if !eat(input, ')') {
        return Err(EmptyErr::default());
    }
    Ok(false)
}

/// ```text
/// QuantifiableAssertion[N] :: (?= Disjunction[~U,~V,?N] )
///                          | (?! Disjunction[~U,~V,?N] )
/// ```
fn quantifiable_assertion<'src>(
    input: &mut Input<'src, '_>,
    subpattern: &Subpattern<'src>,
) -> Parsed<()> {
    input.parse(just("(?").then(one_of("=!")))?;
    input.parse(subpattern)?;
    if !eat(input, ')') {
        return Err(EmptyErr::default());
    }
    Ok(())
}

/// ```text
/// Quantifier :: QuantifierPrefix | QuantifierPrefix ?
/// ```
fn quantifier<'src>(input: &mut Input<'src, '_>) -> Parsed<bool> {
    let complete = quantifier_prefix(input)?;
    if complete {
        eat(input, '?');
    }
    Ok(complete)
}

/// ```text
/// QuantifierPrefix :: * | + | ? | { DecimalDigits[~Sep] }
///                    | { DecimalDigits[~Sep] , }
///                    | { DecimalDigits[~Sep] , DecimalDigits[~Sep] }
/// ```
/// Mathematical values are compared as decimal strings, with no machine bound.
/// False denotes an incomplete prefix, treated literally only in Annex B.
fn quantifier_prefix<'src>(input: &mut Input<'src, '_>) -> Parsed<bool> {
    if matches!(input.peek(), Some('*' | '+' | '?')) {
        input.next();
        return Ok(true);
    }

    if !eat(input, '{') {
        return Ok(false);
    }
    if !input.peek().is_some_and(|ch| ch.is_ascii_digit()) {
        return Ok(false);
    }
    let min = input.parse(decimal_digits())?;
    let max = if eat(input, ',') {
        if input.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            Some(input.parse(decimal_digits())?)
        } else {
            None
        }
    } else {
        Some(min)
    };
    if !eat(input, '}') {
        return Ok(false);
    }
    if max.is_some_and(|max| decimal_cmp(min, max) == Ordering::Greater) {
        return Err(EmptyErr::default());
    }
    Ok(true)
}

fn decimal_cmp(left: &str, right: &str) -> Ordering {
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

/// ```text
/// Atom[U,V,N] :: PatternCharacter | . | \ AtomEscape[?U,?N]
///   | CharacterClass[?U,?V]
///   | ( GroupSpecifier[?U]? Disjunction[?U,?V,?N] )
///   | (? RegularExpressionModifiers : Disjunction[?U,?V,?N] )
///   | (? RegularExpressionModifiers - RegularExpressionModifiers : Disjunction[?U,?V,?N] )
/// ```
fn atom<'src>(input: &mut Input<'src, '_>, subpattern: &Subpattern<'src>) -> Parsed<()> {
    match input.peek() {
        Some('.') => {
            input.next();
        }
        Some('\\') => {
            input.next();
            atom_escape(input)?;
        }
        Some('[') => {
            input.parse(character_class())?;
        }
        Some('(') => {
            input.next();
            if starts_with(input, "?<") {
                input.state().captures += 1;

                let name = input.parse(group_specifier())?;
                input.state().declare_name(name);
            } else if eat(input, '?') {
                if !eat(input, ':') {
                    let enabled = regular_expression_modifiers(input)?;
                    let disabled = if eat(input, '-') {
                        regular_expression_modifiers(input)?
                    } else {
                        0
                    };
                    if enabled & disabled != 0 {
                        return Err(EmptyErr::default());
                    }
                    if !eat(input, ':') {
                        return Err(EmptyErr::default());
                    }
                    if enabled | disabled == 0 {
                        return Err(EmptyErr::default());
                    }
                }
            } else {
                input.state().captures += 1;
            }
            if input.state().captures >= u32::MAX as usize {
                return Err(EmptyErr::default());
            }
            input.parse(subpattern)?;
            if !eat(input, ')') {
                return Err(EmptyErr::default());
            }
        }
        Some('*' | '+' | '?') => return Err(EmptyErr::default()),
        Some('{') => return Err(EmptyErr::default()),
        Some('}' | ']') => return Err(EmptyErr::default()),
        None => return Err(EmptyErr::default()),
        _ => {
            input.parse(pattern_character())?;
        }
    }
    Ok(())
}

/// ```text
/// ExtendedAtom[N] :: . | \ AtomEscape[~U,?N] | \ [lookahead = c]
///   | CharacterClass[~U,~V]
///   | ( GroupSpecifier[~U]? Disjunction[~U,~V,?N] )
///   | (? RegularExpressionModifiers (- RegularExpressionModifiers)? : Disjunction[~U,~V,?N] )
///   | InvalidBracedQuantifier | ExtendedPatternCharacter
/// ```
fn extended_atom<'src>(input: &mut Input<'src, '_>, subpattern: &Subpattern<'src>) -> Parsed<()> {
    if matches!(input.peek(), Some('{' | '}' | ']')) {
        if input.peek() == Some('{') {
            let checkpoint = input.save();

            if invalid_braced_quantifier(input)? {
                return Err(EmptyErr::default());
            }
            input.rewind(checkpoint);
        }
        input.parse(extended_pattern_character())?;
    } else if starts_with(input, "\\c") {
        let checkpoint = input.save();
        input.parse(just("\\c"))?;
        if input.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            input.next();
        } else {
            input.rewind(checkpoint);
            input.next(); // Annex B consumes only the backslash.
        }
    } else {
        atom(input, subpattern)?;
    }
    Ok(())
}

/// ```text
/// InvalidBracedQuantifier :: { DecimalDigits[~Sep] }
///   | { DecimalDigits[~Sep] , } | { DecimalDigits[~Sep] , DecimalDigits[~Sep] }
/// ```
fn invalid_braced_quantifier<'src>(input: &mut Input<'src, '_>) -> Parsed<bool> {
    quantifier_prefix(input)
}

/// ```text
/// ExtendedPatternCharacter :: SourceCharacter but not ^ $ \ . * + ? ( ) [ |
/// ```
fn extended_pattern_character<'src>() -> impl Parser<'src, &'src str, (), Extra> {
    any().filter(|ch| !"^$\\.*+?()[|".contains(*ch)).ignored()
}

/// ```text
/// SyntaxCharacter :: one of ^ $ \ . * + ? ( ) [ ] { } |
/// ```
fn syntax_character<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    one_of("^$\\.*+?()[]{}|")
}

/// ```text
/// PatternCharacter :: SourceCharacter but not SyntaxCharacter
/// ```
fn pattern_character<'src>() -> impl Parser<'src, &'src str, (), Extra> {
    syntax_character()
        .not()
        .ignore_then(any())
        .try_map(|ch, _| {
            if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                Err(EmptyErr::default())
            } else {
                Ok(())
            }
        })
}

/// ```text
/// RegularExpressionModifiers :: [empty] | RegularExpressionModifiers RegularExpressionModifier
/// ```
fn regular_expression_modifiers<'src>(input: &mut Input<'src, '_>) -> Parsed<u8> {
    let mut flags = 0;
    while input.peek().is_some_and(|ch| matches!(ch, 'i' | 'm' | 's')) {
        let flag = input.parse(regular_expression_modifier())?;
        if flags & flag != 0 {
            return Err(EmptyErr::default());
        }
        flags |= flag;
    }
    Ok(flags)
}

/// ```text
/// RegularExpressionModifier :: one of i m s
/// ```
fn regular_expression_modifier<'src>() -> impl Parser<'src, &'src str, u8, Extra> {
    one_of("ims").map(|ch| match ch {
        'i' => 1,
        'm' => 2,
        _ => 4,
    })
}

/// ```text
/// AtomEscape[U,N] :: DecimalEscape | CharacterClassEscape[?U]
///                  | CharacterEscape[?U,?N] | [+N] k GroupName[?U]
/// ```
fn atom_escape<'src>(input: &mut Input<'src, '_>) -> Parsed<()> {
    match input.peek() {
        Some('1'..='9') => {
            let index = input.parse(decimal_escape())?;
            if input.ctx().unicode {
                input.state().max_numeric_reference =
                    input.state().max_numeric_reference.max(index);
            }
        }
        Some('d' | 'D' | 's' | 'S' | 'w' | 'W') => {
            character_class_escape(input)?;
        }
        Some('p' | 'P') if input.ctx().unicode => {
            character_class_escape(input)?;
        }
        Some('k') if input.ctx().named => {
            input.next();
            if input.peek() != Some('<') {
                return Err(EmptyErr::default());
            }
            let name = input.parse(group_name())?;

            // None marks a forward reference until its group is parsed.
            input.state().names.entry(name).or_default();
        }
        None => return Err(EmptyErr::default()),
        _ => {
            character_escape(input)?;
        }
    }
    Ok(())
}

/// ```text
/// DecimalEscape :: NonZeroDigit DecimalDigits[~Sep]? [lookahead ∉ DecimalDigit]
/// ```
fn decimal_escape<'src>() -> impl Parser<'src, &'src str, usize, Extra> {
    one_of('1'..='9')
        .then(decimal_digits().or_not())
        .map(|(first, rest)| {
            rest.unwrap_or("")
                .bytes()
                .fold(first as usize - '0' as usize, |n, b| {
                    n.saturating_mul(10).saturating_add((b - b'0') as usize)
                })
        })
}

/// ```text
/// DecimalDigits[~Sep] :: DecimalDigit | DecimalDigits[~Sep] DecimalDigit
/// ```
fn decimal_digits<'src>() -> impl Parser<'src, &'src str, &'src str, Extra> {
    decimal_digit().repeated().at_least(1).to_slice()
}

/// ```text
/// DecimalDigit :: one of 0 1 2 3 4 5 6 7 8 9
/// ```
fn decimal_digit<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    one_of('0'..='9')
}

/// ```text
/// CharacterEscape[U,N] :: ControlEscape | c AsciiLetter
///   | 0 [lookahead ∉ DecimalDigit] | HexEscapeSequence
///   | RegExpUnicodeEscapeSequence[?U] | [~U] LegacyOctalEscapeSequence
///   | IdentityEscape[?U,?N]
/// ```
fn character_escape<'src>(input: &mut Input<'src, '_>) -> Parsed<u32> {
    match input.peek() {
        Some('f' | 'n' | 'r' | 't' | 'v') => input.parse(control_escape()),
        Some('c') => {
            input.next();
            if input.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                Ok(input.parse(ascii_letter())? as u32 % 32)
            } else {
                Err(EmptyErr::default())
            }
        }
        Some('0'..='7') if !input.ctx().unicode => legacy_octal_escape_sequence(input),
        Some('0') => {
            input.next();
            if input.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                Err(EmptyErr::default())
            } else {
                Ok(0)
            }
        }
        Some('x') => {
            let checkpoint = input.save();
            match input.parse(hex_escape_sequence()) {
                Ok(value) => Ok(value),
                Err(err) if input.ctx().unicode => Err(err),
                Err(_) => {
                    input.rewind(checkpoint);
                    identity_escape(input)
                }
            }
        }
        Some('u') => {
            let checkpoint = input.save();
            let unicode = input.ctx().unicode;
            match reg_exp_unicode_escape_sequence(input, unicode) {
                Ok(value) => Ok(value),
                Err(err) if unicode => Err(err),
                Err(_) => {
                    input.rewind(checkpoint);
                    identity_escape(input)
                }
            }
        }
        _ => identity_escape(input),
    }
}

/// ```text
/// ControlEscape :: one of f n r t v
/// ```
fn control_escape<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    one_of("fnrtv").map(|ch| match ch {
        'f' => 12,
        'n' => 10,
        'r' => 13,
        't' => 9,
        _ => 11,
    })
}

/// ```text
/// AsciiLetter :: one of a…z A…Z
/// ```
fn ascii_letter<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    any().filter(|ch: &char| ch.is_ascii_alphabetic())
}

/// ```text
/// HexEscapeSequence :: x HexDigit HexDigit
/// ```
fn hex_escape_sequence<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    just('x')
        .ignore_then(hex_digit().then(hex_digit()))
        .map(|(a, b)| a * 16 + b)
}

/// ```text
/// HexDigit :: one of 0…9 a…f A…F
/// ```
fn hex_digit<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    any().try_map(|ch: char, _| ch.to_digit(16).ok_or(EmptyErr::default()))
}

/// ```text
/// Hex4Digits :: HexDigit HexDigit HexDigit HexDigit
/// ```
fn hex4_digits<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    hex_digit()
        .then(hex_digit())
        .then(hex_digit())
        .then(hex_digit())
        .map(|(((a, b), c), d)| ((a * 16 + b) * 16 + c) * 16 + d)
}

/// ```text
/// RegExpUnicodeEscapeSequence[U] :: [+U] u HexLeadSurrogate \u HexTrailSurrogate
///   | [+U] u HexLeadSurrogate | [+U] u HexTrailSurrogate | [+U] u HexNonSurrogate
///   | [~U] u Hex4Digits | [+U] u{ CodePoint }
/// ```
fn reg_exp_unicode_escape_sequence<'src>(
    input: &mut Input<'src, '_>,
    unicode: bool,
) -> Parsed<u32> {
    input.parse(just('u'))?;
    if unicode && eat(input, '{') {
        let value = input.parse(code_point())?;
        if !eat(input, '}') {
            return Err(EmptyErr::default());
        }
        return Ok(value);
    }
    let value = input.parse(hex4_digits())?;
    if unicode && (0xd800..=0xdbff).contains(&value) && starts_with(input, "\\u") {
        let checkpoint = input.save();
        input.parse(just("\\u"))?;
        match input.parse(hex_trail_surrogate()) {
            Ok(trail) => return Ok(0x10000 + (value - 0xd800) * 1024 + trail - 0xdc00),
            Err(_) => input.rewind(checkpoint),
        }
    }
    Ok(value)
}

/// ```text
/// HexTrailSurrogate :: Hex4Digits but only if 0xDC00 ≤ MV ≤ 0xDFFF
/// ```
fn hex_trail_surrogate<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    hex4_digits().try_map(|value, _| {
        if (0xdc00..=0xdfff).contains(&value) {
            Ok(value)
        } else {
            Err(EmptyErr::default())
        }
    })
}

/// ```text
/// CodePoint :: HexDigits but only if MV ≤ 0x10FFFF
/// ```
fn code_point<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    any()
        .filter(|ch: &char| ch.is_ascii_hexdigit())
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(|digits: &str, _| {
            let value = digits.bytes().try_fold(0u32, |value, byte| {
                value
                    .checked_mul(16)?
                    .checked_add((byte as char).to_digit(16)?)
            });
            value
                .filter(|value| *value <= 0x10ffff)
                .ok_or(EmptyErr::default())
        })
}

/// ```text
/// LegacyOctalEscapeSequence :: 0 [lookahead ∉ OctalDigit]
///   | NonZeroOctalDigit [lookahead ∉ OctalDigit]
///   | ZeroToThree OctalDigit [lookahead ∉ OctalDigit]
///   | FourToSeven OctalDigit | ZeroToThree OctalDigit OctalDigit
/// ```
fn legacy_octal_escape_sequence<'src>(input: &mut Input<'src, '_>) -> Parsed<u32> {
    let first = input.parse(octal_digit())?;
    let mut value = first;
    for _ in 0..if first <= 3 { 2 } else { 1 } {
        if input.peek().is_some_and(|ch| matches!(ch, '0'..='7')) {
            value = value * 8 + input.parse(octal_digit())?;
        } else {
            break;
        }
    }
    Ok(value)
}

/// ```text
/// OctalDigit :: one of 0 1 2 3 4 5 6 7
/// ```
fn octal_digit<'src>() -> impl Parser<'src, &'src str, u32, Extra> {
    one_of('0'..='7').map(|ch| ch as u32 - '0' as u32)
}

/// ```text
/// IdentityEscape[U,N] :: [+U] SyntaxCharacter | [+U] /
///                     | [~U] SourceCharacterIdentityEscape[?N]
/// ```
fn identity_escape<'src>(input: &mut Input<'src, '_>) -> Parsed<u32> {
    if !input.ctx().unicode {
        return source_character_identity_escape(input);
    }
    if input.peek() == Some('/') {
        input.next();
        return Ok('/' as u32);
    }
    input.parse(syntax_character()).map(|ch| ch as u32)
}

/// ```text
/// SourceCharacterIdentityEscape[N] :: [~N] SourceCharacter but not c
///                                  | [+N] SourceCharacter but not c or k
/// ```
fn source_character_identity_escape<'src>(input: &mut Input<'src, '_>) -> Parsed<u32> {
    match input.next() {
        Some(ch)
            if ch != 'c'
                && !(input.ctx().named && ch == 'k')
                && !matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') =>
        {
            Ok(ch as u32)
        }
        None => Err(EmptyErr::default()),
        _ => Err(EmptyErr::default()),
    }
}

/// ```text
/// GroupSpecifier[U] :: ? GroupName[?U]
/// ```
fn group_specifier<'src>() -> impl Parser<'src, &'src str, String, Extra> {
    just('?').ignore_then(group_name())
}

/// ```text
/// GroupName[U] :: < RegExpIdentifierName[?U] >
/// ```
fn group_name<'src>() -> impl Parser<'src, &'src str, String, Extra> {
    just('<')
        .ignore_then(reg_exp_identifier_name())
        .then_ignore(just('>'))
}

/// ```text
/// RegExpIdentifierName[U] :: RegExpIdentifierStart[?U]
///                        | RegExpIdentifierName[?U] RegExpIdentifierPart[?U]
/// ```
fn reg_exp_identifier_name<'src>() -> impl Parser<'src, &'src str, String, Extra> {
    custom(|input: &mut Input<'src, '_>| {
        let mut name = String::new();
        name.push(reg_exp_identifier_start(input)?);
        loop {
            let checkpoint = input.save();
            match reg_exp_identifier_part(input) {
                Ok(ch) => name.push(ch),
                Err(_) => {
                    input.rewind(checkpoint);
                    return Ok(name);
                }
            }
        }
    })
}

/// ```text
/// RegExpIdentifierStart[U] :: IdentifierStartChar
///   | \ RegExpUnicodeEscapeSequence[+U] | [~U] UnicodeLeadSurrogate UnicodeTrailSurrogate
/// ```
fn reg_exp_identifier_start<'src>(input: &mut Input<'src, '_>) -> Parsed<char> {
    let cp = if eat(input, '\\') {
        reg_exp_unicode_escape_sequence(input, true)?
    } else {
        input.parse(any())? as u32
    };
    char::from_u32(cp)
        .filter(|ch| *ch == '$' || *ch == '_' || unicode_id_start::is_id_start(*ch))
        .ok_or(EmptyErr::default())
}

/// ```text
/// RegExpIdentifierPart[U] :: IdentifierPartChar
///   | \ RegExpUnicodeEscapeSequence[+U] | [~U] UnicodeLeadSurrogate UnicodeTrailSurrogate
/// ```
fn reg_exp_identifier_part<'src>(input: &mut Input<'src, '_>) -> Parsed<char> {
    let cp = if eat(input, '\\') {
        reg_exp_unicode_escape_sequence(input, true)?
    } else {
        input.parse(any())? as u32
    };
    char::from_u32(cp)
        .filter(|ch| {
            matches!(ch, '$' | '_' | '\u{200c}' | '\u{200d}')
                || unicode_id_start::is_id_continue(*ch)
        })
        .ok_or(EmptyErr::default())
}

#[derive(Clone, Copy)]
struct ClassValue {
    // Legacy raw astral characters contribute two UTF-16 code units.
    character: Option<(u32, u32)>,
    strings: bool,
}

impl ClassValue {
    fn character(value: u32, unicode: bool) -> Self {
        let units = if !unicode && value > 0xffff {
            (
                0xd800 + ((value - 0x10000) >> 10),
                0xdc00 + ((value - 0x10000) & 1023),
            )
        } else {
            (value, value)
        };
        Self {
            character: Some(units),
            strings: false,
        }
    }
    fn set(strings: bool) -> Self {
        Self {
            character: None,
            strings,
        }
    }
}

/// ```text
/// CharacterClass[U,V] :: [ [lookahead ≠ ^] ClassContents[?U,?V] ]
///                     | [^ ClassContents[?U,?V] ]
/// ```
/// A negated class must have MayContainStrings = false.
fn character_class<'src>() -> Class<'src> {
    recursive(|class: Class<'src>| {
        production(move |input: &mut Input<'src, '_>| {
            input.parse(just('['))?;
            let negative = eat(input, '^');
            let strings = class_contents(input, &class)?;
            if !eat(input, ']') {
                return Err(EmptyErr::default());
            }
            if negative && strings {
                return Err(EmptyErr::default());
            }
            Ok(ClassValue::set(strings))
        })
    })
}

/// ```text
/// ClassContents[U,V] :: [empty] | [~V] NonemptyClassRanges[?U]
///                    | [+V] ClassSetExpression
/// ```
fn class_contents<'src>(input: &mut Input<'src, '_>, class: &Class<'src>) -> Parsed<bool> {
    if input.peek() == Some(']') {
        return Ok(false);
    }
    if input.peek().is_none() {
        return Err(EmptyErr::default());
    }
    if input.ctx().sets {
        class_set_expression(input, class)
    } else {
        input.parse(nonempty_class_ranges())?;
        Ok(false)
    }
}

/// ```text
/// NonemptyClassRanges[U] :: ClassAtom[?U]
///   | ClassAtom[?U] NonemptyClassRangesNoDash[?U]
///   | ClassAtom[?U] - ClassAtom[?U] ClassContents[?U,~V]
/// ```
/// UTF-16 halves are retained locally when a raw astral range endpoint is read.
fn nonempty_class_ranges<'src>() -> impl Parser<'src, &'src str, (), Extra> {
    production(nonempty_class_ranges_no_dash)
}

/// ```text
/// NonemptyClassRangesNoDash[U] :: ClassAtom[?U]
///   | ClassAtomNoDash[?U] NonemptyClassRangesNoDash[?U]
///   | ClassAtomNoDash[?U] - ClassAtom[?U] ClassContents[?U,~V]
/// ```
/// Left recursion and tail calls are flattened into one loop.
fn nonempty_class_ranges_no_dash<'src>(input: &mut Input<'src, '_>) -> Parsed<()> {
    let mut pending = None;
    while pending.is_some() || !matches!(input.peek(), None | Some(']')) {
        let from = if let Some(value) = pending.take() {
            ClassValue::character(value, true)
        } else {
            input.parse(class_atom())?
        };
        if !eat(input, '-') {
            continue;
        }
        if input.peek() == Some(']') {
            continue;
        }
        if input.peek().is_none() {
            return Err(EmptyErr::default());
        }
        let to = input.parse(class_atom())?;
        if let Some((first, last)) = to.character
            && first != last
        {
            pending = Some(last);
        }
        match (from.character, to.character) {
            (Some((_, from)), Some((to, _))) if from > to => {
                return Err(EmptyErr::default());
            }
            (None, _) | (_, None) if input.ctx().unicode => {
                return Err(EmptyErr::default());
            }
            _ => (),
        }
    }
    Ok(())
}

/// ```text
/// ClassAtom[U] :: - | ClassAtomNoDash[?U]
/// ```
fn class_atom<'src>() -> impl Parser<'src, &'src str, ClassValue, Extra> {
    just('-')
        .map(|_| ClassValue::character('-' as u32, true))
        .or(production(class_atom_no_dash))
}

/// ```text
/// ClassAtomNoDash[U,N] :: SourceCharacter but not \ or ] or -
///   | \ ClassEscape[?U,?N] | [~U] \ [lookahead = c]
/// ```
fn class_atom_no_dash<'src>(input: &mut Input<'src, '_>) -> Parsed<ClassValue> {
    if eat(input, '\\') {
        return class_escape(input);
    }
    match input.next() {
        Some(ch) if !matches!(ch, ']' | '-' | '\n' | '\r' | '\u{2028}' | '\u{2029}') => {
            Ok(ClassValue::character(ch as u32, input.ctx().unicode))
        }
        _ => Err(EmptyErr::default()),
    }
}

/// ```text
/// ClassEscape[U,N] :: b | [+U] - | [~U] c ClassControlLetter
///                  | CharacterClassEscape[?U] | CharacterEscape[?U,?N]
/// ```
fn class_escape<'src>(input: &mut Input<'src, '_>) -> Parsed<ClassValue> {
    if eat(input, 'b') {
        return Ok(ClassValue::character(8, true));
    }
    if input.ctx().unicode && eat(input, '-') {
        return Ok(ClassValue::character('-' as u32, true));
    }
    if matches!(input.peek(), Some('d' | 'D' | 's' | 'S' | 'w' | 'W'))
        || input.ctx().unicode && matches!(input.peek(), Some('p' | 'P'))
    {
        return character_class_escape(input);
    }
    if input.peek() == Some('c') {
        let checkpoint = input.save();
        input.next();
        if !input.ctx().unicode
            && input
                .peek()
                .is_some_and(|ch| ch.is_ascii_digit() || ch == '_')
        {
            return Ok(ClassValue::character(
                input.parse(class_control_letter())? as u32 % 32,
                true,
            ));
        }
        let letter = input.peek().is_some_and(|ch| ch.is_ascii_alphabetic());
        input.rewind(checkpoint);
        if !letter {
            if input.ctx().unicode {
                return Err(EmptyErr::default());
            }
            return Ok(ClassValue::character('\\' as u32, true));
        }
    }
    let unicode = input.ctx().unicode;
    character_escape(input).map(|cp| ClassValue::character(cp, unicode))
}

/// ```text
/// ClassControlLetter :: DecimalDigit | _
/// ```
fn class_control_letter<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    decimal_digit().or(just('_'))
}

/// ```text
/// CharacterClassEscape[U] :: d | D | s | S | w | W
///   | [+U] p{ UnicodePropertyValueExpression }
///   | [+U] P{ UnicodePropertyValueExpression }
/// ```
fn character_class_escape<'src>(input: &mut Input<'src, '_>) -> Parsed<ClassValue> {
    if matches!(input.peek(), Some('d' | 'D' | 's' | 'S' | 'w' | 'W')) {
        input.parse(one_of("dDsSwW"))?;
        return Ok(ClassValue::set(false));
    }
    let negative = input.parse(one_of("pP"))? == 'P';
    if !eat(input, '{') {
        return Err(EmptyErr::default());
    }
    let strings = unicode_property_value_expression(input)?;
    if !eat(input, '}') || negative && strings {
        return Err(EmptyErr::default());
    }
    Ok(ClassValue::set(strings))
}

/// ```text
/// UnicodePropertyValueExpression :: UnicodePropertyName = UnicodePropertyValue
///                                | LoneUnicodePropertyNameOrValue
/// ```
/// Property spelling and aliases are exact, with no loose matching.
fn unicode_property_value_expression<'src>(input: &mut Input<'src, '_>) -> Parsed<bool> {
    let checkpoint = input.save();
    if let Ok(name) = input.parse(unicode_property_name())
        && eat(input, '=')
    {
        let value = input.parse(unicode_property_value())?;
        if valid_property_value(name, value) {
            return Ok(false);
        }
        return Err(EmptyErr::default());
    }
    input.rewind(checkpoint);
    let value = input.parse(lone_unicode_property_name_or_value())?;
    if contains_property(&GENERAL_CATEGORIES, value) || contains_property(&BINARY_PROPERTIES, value)
    {
        return Ok(false);
    }
    if input.ctx().sets && contains_property(&STRING_PROPERTIES, value) {
        return Ok(true);
    }
    Err(EmptyErr::default())
}

// Packed aliases avoid a pointer and dynamic relocation for every table entry.
// The const builder checks the order required by binary search and the offset width.
struct PropertyTable {
    bytes: &'static [u8],
    ends: &'static [u16],
}

const fn property_ends<const N: usize>(bytes: &[u8], lengths: &[usize; N]) -> [u16; N] {
    let mut ends = [0; N];
    let mut end = 0;
    let mut i = 0;
    while i < N {
        end += lengths[i];
        assert!(end <= u16::MAX as usize);
        if i > 0 {
            let previous_end = ends[i - 1] as usize;
            let mut left = if i == 1 { 0 } else { ends[i - 2] as usize };
            let mut right = previous_end;
            while left < previous_end && right < end && bytes[left] == bytes[right] {
                left += 1;
                right += 1;
            }
            assert!(right < end && (left == previous_end || bytes[left] < bytes[right]));
        }
        ends[i] = end as u16;
        i += 1;
    }
    ends
}

macro_rules! properties {
    ($($name:literal),* $(,)?) => {
        PropertyTable {
            bytes: concat!($($name),*).as_bytes(),
            ends: &property_ends(concat!($($name),*).as_bytes(), &[$($name.len()),*]),
        }
    };
}

#[inline(never)]
fn contains_property(properties: &PropertyTable, value: &str) -> bool {
    let mut low = 0;
    let mut high = properties.ends.len();
    while low < high {
        let mid = low + (high - low) / 2;
        let start = if mid == 0 {
            0
        } else {
            properties.ends[mid - 1] as usize
        };
        let end = properties.ends[mid] as usize;
        match properties.bytes[start..end].cmp(value.as_bytes()) {
            Ordering::Less => low = mid + 1,
            Ordering::Greater => high = mid,
            Ordering::Equal => return true,
        }
    }
    false
}

fn valid_property_value(name: &str, value: &str) -> bool {
    match name {
        "General_Category" | "gc" => contains_property(&GENERAL_CATEGORIES, value),
        "Script" | "sc" | "Script_Extensions" | "scx" => contains_property(&SCRIPTS, value),
        _ => false,
    }
}

/// ```text
/// UnicodePropertyName :: UnicodePropertyNameCharacters
/// ```
fn unicode_property_name<'src>() -> impl Parser<'src, &'src str, &'src str, Extra> {
    unicode_property_name_characters()
}

/// ```text
/// UnicodePropertyNameCharacters :: UnicodePropertyNameCharacter UnicodePropertyNameCharacters?
/// ```
fn unicode_property_name_characters<'src>() -> impl Parser<'src, &'src str, &'src str, Extra> {
    unicode_property_name_character()
        .repeated()
        .at_least(1)
        .to_slice()
}

/// ```text
/// UnicodePropertyNameCharacter :: AsciiLetter | _
/// ```
fn unicode_property_name_character<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    ascii_letter().or(just('_'))
}

/// ```text
/// UnicodePropertyValue :: UnicodePropertyValueCharacters
/// ```
fn unicode_property_value<'src>() -> impl Parser<'src, &'src str, &'src str, Extra> {
    unicode_property_value_characters()
}

/// ```text
/// LoneUnicodePropertyNameOrValue :: UnicodePropertyValueCharacters
/// ```
fn lone_unicode_property_name_or_value<'src>() -> impl Parser<'src, &'src str, &'src str, Extra> {
    unicode_property_value_characters()
}

/// ```text
/// UnicodePropertyValueCharacters :: UnicodePropertyValueCharacter UnicodePropertyValueCharacters?
/// ```
fn unicode_property_value_characters<'src>() -> impl Parser<'src, &'src str, &'src str, Extra> {
    unicode_property_value_character()
        .repeated()
        .at_least(1)
        .to_slice()
}

/// ```text
/// UnicodePropertyValueCharacter :: UnicodePropertyNameCharacter | DecimalDigit
/// ```
fn unicode_property_value_character<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    unicode_property_name_character().or(decimal_digit())
}

/// ```text
/// ClassSetExpression :: ClassUnion | ClassIntersection | ClassSubtraction
/// ```
/// The shared initial ClassSetOperand is factored before selecting a branch.
fn class_set_expression<'src>(input: &mut Input<'src, '_>, class: &Class<'src>) -> Parsed<bool> {
    let mut first = class_set_operand(input, class)?;
    let range = first.character.is_some() && input.peek() == Some('-') && !starts_with(input, "--");
    if range {
        first = class_set_range(input, first.character.unwrap().0)?;
    }
    if starts_with(input, "&&") {
        if range {
            return Err(EmptyErr::default());
        }
        class_intersection(input, first.strings, class)
    } else if starts_with(input, "--") {
        if range {
            return Err(EmptyErr::default());
        }
        class_subtraction(input, first.strings, class)
    } else {
        class_union(input, first.strings, class)
    }
}

/// ```text
/// ClassUnion :: ClassSetRange ClassUnion? | ClassSetOperand ClassUnion?
/// ```
/// `strings` is MayContainStrings of the already matched first item.
fn class_union<'src>(
    input: &mut Input<'src, '_>,
    strings: bool,
    class: &Class<'src>,
) -> Parsed<bool> {
    let mut strings = strings;
    while !matches!(input.peek(), None | Some(']')) {
        let mut operand = class_set_operand(input, class)?;
        if let Some((first, _)) = operand.character
            && input.peek() == Some('-')
            && !starts_with(input, "--")
        {
            operand = class_set_range(input, first)?;
        }
        strings |= operand.strings;
    }
    Ok(strings)
}

/// ```text
/// ClassIntersection :: ClassSetOperand && [lookahead ≠ &] ClassSetOperand
///                    | ClassIntersection && [lookahead ≠ &] ClassSetOperand
/// ```
fn class_intersection<'src>(
    input: &mut Input<'src, '_>,
    strings: bool,
    class: &Class<'src>,
) -> Parsed<bool> {
    let mut strings = strings;
    loop {
        if !starts_with(input, "&&") {
            return Err(EmptyErr::default());
        }
        input.parse(just("&&"))?;
        if input.peek() == Some('&') {
            return Err(EmptyErr::default());
        }
        strings &= class_set_operand(input, class)?.strings;
        if input.peek() == Some(']') {
            return Ok(strings);
        }
    }
}

/// ```text
/// ClassSubtraction :: ClassSetOperand -- ClassSetOperand
///                   | ClassSubtraction -- ClassSetOperand
/// ```
fn class_subtraction<'src>(
    input: &mut Input<'src, '_>,
    strings: bool,
    class: &Class<'src>,
) -> Parsed<bool> {
    loop {
        if !starts_with(input, "--") {
            return Err(EmptyErr::default());
        }
        input.parse(just("--"))?;
        class_set_operand(input, class)?;
        if input.peek() == Some(']') {
            return Ok(strings);
        }
    }
}

/// ```text
/// ClassSetRange :: ClassSetCharacter - ClassSetCharacter
/// ```
/// The first ClassSetCharacter has already been consumed by prefix factoring.
fn class_set_range<'src>(input: &mut Input<'src, '_>, first: u32) -> Parsed<ClassValue> {
    input.parse(just('-'))?;
    let last = class_set_character(input)?;
    if first > last {
        return Err(EmptyErr::default());
    }
    Ok(ClassValue::set(false))
}

/// ```text
/// ClassSetOperand :: NestedClass | ClassStringDisjunction | ClassSetCharacter
/// ```
fn class_set_operand<'src>(input: &mut Input<'src, '_>, class: &Class<'src>) -> Parsed<ClassValue> {
    if input.peek() == Some('[')
        || ["\\d", "\\D", "\\s", "\\S", "\\w", "\\W", "\\p", "\\P"]
            .iter()
            .any(|prefix| starts_with(input, prefix))
    {
        nested_class(input, class)
    } else if starts_with(input, "\\q{") {
        input.parse(class_string_disjunction()).map(ClassValue::set)
    } else {
        class_set_character(input).map(|cp| ClassValue::character(cp, true))
    }
}

/// ```text
/// NestedClass :: [ [lookahead ≠ ^] ClassContents[+U,+V] ]
///              | [^ ClassContents[+U,+V] ] | \ CharacterClassEscape[+U]
/// ```
fn nested_class<'src>(input: &mut Input<'src, '_>, class: &Class<'src>) -> Parsed<ClassValue> {
    if input.peek() == Some('[') {
        input.parse(class)
    } else {
        input.parse(just('\\').ignore_then(production(character_class_escape)))
    }
}

/// ```text
/// ClassStringDisjunction :: \q{ ClassStringDisjunctionContents }
/// ```
fn class_string_disjunction<'src>() -> impl Parser<'src, &'src str, bool, Extra> {
    just("\\q{")
        .ignore_then(production(class_string_disjunction_contents))
        .then_ignore(just('}'))
}

/// ```text
/// ClassStringDisjunctionContents :: ClassString
///                                | ClassString | ClassStringDisjunctionContents
/// ```
fn class_string_disjunction_contents<'src>(input: &mut Input<'src, '_>) -> Parsed<bool> {
    let mut strings = class_string(input)?;
    while eat(input, '|') {
        strings |= class_string(input)?;
    }
    Ok(strings)
}

/// ```text
/// ClassString :: [empty] | NonEmptyClassString
/// ```
fn class_string<'src>(input: &mut Input<'src, '_>) -> Parsed<bool> {
    if matches!(input.peek(), Some('}' | '|')) {
        Ok(true)
    } else {
        non_empty_class_string(input).map(|length| length != 1)
    }
}

/// ```text
/// NonEmptyClassString :: ClassSetCharacter NonEmptyClassString?
/// ```
fn non_empty_class_string<'src>(input: &mut Input<'src, '_>) -> Parsed<usize> {
    let mut length = 0usize;
    while !matches!(input.peek(), None | Some('}' | '|')) {
        class_set_character(input)?;
        length = length.saturating_add(1);
    }
    Ok(length)
}

/// ```text
/// ClassSetCharacter :: [lookahead ∉ ClassSetReservedDoublePunctuator]
///   SourceCharacter but not ClassSetSyntaxCharacter
///   | \ CharacterEscape[+U] | \ ClassSetReservedPunctuator | \b
/// ```
fn class_set_character<'src>(input: &mut Input<'src, '_>) -> Parsed<u32> {
    if eat(input, '\\') {
        if eat(input, 'b') {
            return Ok(8);
        }
        if input.peek().is_some_and(|ch| "&-!#%,:;<=>@`~".contains(ch)) {
            return input
                .parse(class_set_reserved_punctuator())
                .map(|ch| ch as u32);
        }
        return character_escape(input);
    }
    let checkpoint = input.save();
    if input.parse(class_set_reserved_double_punctuator()).is_ok() {
        return Err(EmptyErr::default());
    }
    input.rewind(checkpoint);
    let checkpoint = input.save();
    if input.parse(class_set_syntax_character()).is_ok() {
        return Err(EmptyErr::default());
    }
    input.rewind(checkpoint);
    match input.next() {
        Some(ch) if !matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') => Ok(ch as u32),
        _ => Err(EmptyErr::default()),
    }
}

/// ```text
/// ClassSetReservedDoublePunctuator :: one of
///   && !! ## $$ %% ** ++ ,, .. :: ;; << == >> ?? @@ ^^ `` ~~
/// ```
fn class_set_reserved_double_punctuator<'src>() -> impl Parser<'src, &'src str, (), Extra> {
    one_of("&!#$%*+,.:;<=>?@^`~")
        .then(any())
        .try_map(|(first, second), _| {
            if first == second {
                Ok(())
            } else {
                Err(EmptyErr::default())
            }
        })
}

/// ```text
/// ClassSetSyntaxCharacter :: one of ( ) [ ] { } / - \ |
/// ```
fn class_set_syntax_character<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    one_of("()[]{}/-\\|")
}

/// ```text
/// ClassSetReservedPunctuator :: one of & - ! # % , : ; < = > @ ` ~
/// ```
fn class_set_reserved_punctuator<'src>() -> impl Parser<'src, &'src str, char, Extra> {
    one_of("&-!#%,:;<=>@`~")
}

// ECMA-262 Tables 65–67 and Unicode 17.0.0 property aliases.
static BINARY_PROPERTIES: PropertyTable = properties![
    "AHex",
    "ASCII",
    "ASCII_Hex_Digit",
    "Alpha",
    "Alphabetic",
    "Any",
    "Assigned",
    "Bidi_C",
    "Bidi_Control",
    "Bidi_M",
    "Bidi_Mirrored",
    "CI",
    "CWCF",
    "CWCM",
    "CWKCF",
    "CWL",
    "CWT",
    "CWU",
    "Case_Ignorable",
    "Cased",
    "Changes_When_Casefolded",
    "Changes_When_Casemapped",
    "Changes_When_Lowercased",
    "Changes_When_NFKC_Casefolded",
    "Changes_When_Titlecased",
    "Changes_When_Uppercased",
    "DI",
    "Dash",
    "Default_Ignorable_Code_Point",
    "Dep",
    "Deprecated",
    "Dia",
    "Diacritic",
    "EBase",
    "EComp",
    "EMod",
    "EPres",
    "Emoji",
    "Emoji_Component",
    "Emoji_Modifier",
    "Emoji_Modifier_Base",
    "Emoji_Presentation",
    "Ext",
    "ExtPict",
    "Extended_Pictographic",
    "Extender",
    "Gr_Base",
    "Gr_Ext",
    "Grapheme_Base",
    "Grapheme_Extend",
    "Hex",
    "Hex_Digit",
    "IDC",
    "IDS",
    "IDSB",
    "IDST",
    "IDS_Binary_Operator",
    "IDS_Trinary_Operator",
    "ID_Continue",
    "ID_Start",
    "Ideo",
    "Ideographic",
    "Join_C",
    "Join_Control",
    "LOE",
    "Logical_Order_Exception",
    "Lower",
    "Lowercase",
    "Math",
    "NChar",
    "Noncharacter_Code_Point",
    "Pat_Syn",
    "Pat_WS",
    "Pattern_Syntax",
    "Pattern_White_Space",
    "QMark",
    "Quotation_Mark",
    "RI",
    "Radical",
    "Regional_Indicator",
    "SD",
    "STerm",
    "Sentence_Terminal",
    "Soft_Dotted",
    "Term",
    "Terminal_Punctuation",
    "UIdeo",
    "Unified_Ideograph",
    "Upper",
    "Uppercase",
    "VS",
    "Variation_Selector",
    "White_Space",
    "XIDC",
    "XIDS",
    "XID_Continue",
    "XID_Start",
    "space",
];

static STRING_PROPERTIES: PropertyTable = properties![
    "Basic_Emoji",
    "Emoji_Keycap_Sequence",
    "RGI_Emoji",
    "RGI_Emoji_Flag_Sequence",
    "RGI_Emoji_Modifier_Sequence",
    "RGI_Emoji_Tag_Sequence",
    "RGI_Emoji_ZWJ_Sequence",
];

static GENERAL_CATEGORIES: PropertyTable = properties![
    "C",
    "Cased_Letter",
    "Cc",
    "Cf",
    "Close_Punctuation",
    "Cn",
    "Co",
    "Combining_Mark",
    "Connector_Punctuation",
    "Control",
    "Cs",
    "Currency_Symbol",
    "Dash_Punctuation",
    "Decimal_Number",
    "Enclosing_Mark",
    "Final_Punctuation",
    "Format",
    "Initial_Punctuation",
    "L",
    "LC",
    "Letter",
    "Letter_Number",
    "Line_Separator",
    "Ll",
    "Lm",
    "Lo",
    "Lowercase_Letter",
    "Lt",
    "Lu",
    "M",
    "Mark",
    "Math_Symbol",
    "Mc",
    "Me",
    "Mn",
    "Modifier_Letter",
    "Modifier_Symbol",
    "N",
    "Nd",
    "Nl",
    "No",
    "Nonspacing_Mark",
    "Number",
    "Open_Punctuation",
    "Other",
    "Other_Letter",
    "Other_Number",
    "Other_Punctuation",
    "Other_Symbol",
    "P",
    "Paragraph_Separator",
    "Pc",
    "Pd",
    "Pe",
    "Pf",
    "Pi",
    "Po",
    "Private_Use",
    "Ps",
    "Punctuation",
    "S",
    "Sc",
    "Separator",
    "Sk",
    "Sm",
    "So",
    "Space_Separator",
    "Spacing_Mark",
    "Surrogate",
    "Symbol",
    "Titlecase_Letter",
    "Unassigned",
    "Uppercase_Letter",
    "Z",
    "Zl",
    "Zp",
    "Zs",
    "cntrl",
    "digit",
    "punct",
];

static SCRIPTS: PropertyTable = properties![
    "Adlam",
    "Adlm",
    "Aghb",
    "Ahom",
    "Anatolian_Hieroglyphs",
    "Arab",
    "Arabic",
    "Armenian",
    "Armi",
    "Armn",
    "Avestan",
    "Avst",
    "Bali",
    "Balinese",
    "Bamu",
    "Bamum",
    "Bass",
    "Bassa_Vah",
    "Batak",
    "Batk",
    "Beng",
    "Bengali",
    "Berf",
    "Beria_Erfe",
    "Bhaiksuki",
    "Bhks",
    "Bopo",
    "Bopomofo",
    "Brah",
    "Brahmi",
    "Brai",
    "Braille",
    "Bugi",
    "Buginese",
    "Buhd",
    "Buhid",
    "Cakm",
    "Canadian_Aboriginal",
    "Cans",
    "Cari",
    "Carian",
    "Caucasian_Albanian",
    "Chakma",
    "Cham",
    "Cher",
    "Cherokee",
    "Chorasmian",
    "Chrs",
    "Common",
    "Copt",
    "Coptic",
    "Cpmn",
    "Cprt",
    "Cuneiform",
    "Cypriot",
    "Cypro_Minoan",
    "Cyrillic",
    "Cyrl",
    "Deseret",
    "Deva",
    "Devanagari",
    "Diak",
    "Dives_Akuru",
    "Dogr",
    "Dogra",
    "Dsrt",
    "Dupl",
    "Duployan",
    "Egyp",
    "Egyptian_Hieroglyphs",
    "Elba",
    "Elbasan",
    "Elym",
    "Elymaic",
    "Ethi",
    "Ethiopic",
    "Gara",
    "Garay",
    "Geor",
    "Georgian",
    "Glag",
    "Glagolitic",
    "Gong",
    "Gonm",
    "Goth",
    "Gothic",
    "Gran",
    "Grantha",
    "Greek",
    "Grek",
    "Gujarati",
    "Gujr",
    "Gukh",
    "Gunjala_Gondi",
    "Gurmukhi",
    "Guru",
    "Gurung_Khema",
    "Han",
    "Hang",
    "Hangul",
    "Hani",
    "Hanifi_Rohingya",
    "Hano",
    "Hanunoo",
    "Hatr",
    "Hatran",
    "Hebr",
    "Hebrew",
    "Hira",
    "Hiragana",
    "Hluw",
    "Hmng",
    "Hmnp",
    "Hrkt",
    "Hung",
    "Imperial_Aramaic",
    "Inherited",
    "Inscriptional_Pahlavi",
    "Inscriptional_Parthian",
    "Ital",
    "Java",
    "Javanese",
    "Kaithi",
    "Kali",
    "Kana",
    "Kannada",
    "Katakana",
    "Katakana_Or_Hiragana",
    "Kawi",
    "Kayah_Li",
    "Khar",
    "Kharoshthi",
    "Khitan_Small_Script",
    "Khmer",
    "Khmr",
    "Khoj",
    "Khojki",
    "Khudawadi",
    "Kirat_Rai",
    "Kits",
    "Knda",
    "Krai",
    "Kthi",
    "Lana",
    "Lao",
    "Laoo",
    "Latin",
    "Latn",
    "Lepc",
    "Lepcha",
    "Limb",
    "Limbu",
    "Lina",
    "Linb",
    "Linear_A",
    "Linear_B",
    "Lisu",
    "Lyci",
    "Lycian",
    "Lydi",
    "Lydian",
    "Mahajani",
    "Mahj",
    "Maka",
    "Makasar",
    "Malayalam",
    "Mand",
    "Mandaic",
    "Mani",
    "Manichaean",
    "Marc",
    "Marchen",
    "Masaram_Gondi",
    "Medefaidrin",
    "Medf",
    "Meetei_Mayek",
    "Mend",
    "Mende_Kikakui",
    "Merc",
    "Mero",
    "Meroitic_Cursive",
    "Meroitic_Hieroglyphs",
    "Miao",
    "Mlym",
    "Modi",
    "Mong",
    "Mongolian",
    "Mro",
    "Mroo",
    "Mtei",
    "Mult",
    "Multani",
    "Myanmar",
    "Mymr",
    "Nabataean",
    "Nag_Mundari",
    "Nagm",
    "Nand",
    "Nandinagari",
    "Narb",
    "Nbat",
    "New_Tai_Lue",
    "Newa",
    "Nko",
    "Nkoo",
    "Nshu",
    "Nushu",
    "Nyiakeng_Puachue_Hmong",
    "Ogam",
    "Ogham",
    "Ol_Chiki",
    "Ol_Onal",
    "Olck",
    "Old_Hungarian",
    "Old_Italic",
    "Old_North_Arabian",
    "Old_Permic",
    "Old_Persian",
    "Old_Sogdian",
    "Old_South_Arabian",
    "Old_Turkic",
    "Old_Uyghur",
    "Onao",
    "Oriya",
    "Orkh",
    "Orya",
    "Osage",
    "Osge",
    "Osma",
    "Osmanya",
    "Ougr",
    "Pahawh_Hmong",
    "Palm",
    "Palmyrene",
    "Pau_Cin_Hau",
    "Pauc",
    "Perm",
    "Phag",
    "Phags_Pa",
    "Phli",
    "Phlp",
    "Phnx",
    "Phoenician",
    "Plrd",
    "Prti",
    "Psalter_Pahlavi",
    "Qaac",
    "Qaai",
    "Rejang",
    "Rjng",
    "Rohg",
    "Runic",
    "Runr",
    "Samaritan",
    "Samr",
    "Sarb",
    "Saur",
    "Saurashtra",
    "Sgnw",
    "Sharada",
    "Shavian",
    "Shaw",
    "Shrd",
    "Sidd",
    "Siddham",
    "Sidetic",
    "Sidt",
    "SignWriting",
    "Sind",
    "Sinh",
    "Sinhala",
    "Sogd",
    "Sogdian",
    "Sogo",
    "Sora",
    "Sora_Sompeng",
    "Soyo",
    "Soyombo",
    "Sund",
    "Sundanese",
    "Sunu",
    "Sunuwar",
    "Sylo",
    "Syloti_Nagri",
    "Syrc",
    "Syriac",
    "Tagalog",
    "Tagb",
    "Tagbanwa",
    "Tai_Le",
    "Tai_Tham",
    "Tai_Viet",
    "Tai_Yo",
    "Takr",
    "Takri",
    "Tale",
    "Talu",
    "Tamil",
    "Taml",
    "Tang",
    "Tangsa",
    "Tangut",
    "Tavt",
    "Tayo",
    "Telu",
    "Telugu",
    "Tfng",
    "Tglg",
    "Thaa",
    "Thaana",
    "Thai",
    "Tibetan",
    "Tibt",
    "Tifinagh",
    "Tirh",
    "Tirhuta",
    "Tnsa",
    "Todhri",
    "Todr",
    "Tolong_Siki",
    "Tols",
    "Toto",
    "Tulu_Tigalari",
    "Tutg",
    "Ugar",
    "Ugaritic",
    "Unknown",
    "Vai",
    "Vaii",
    "Vith",
    "Vithkuqi",
    "Wancho",
    "Wara",
    "Warang_Citi",
    "Wcho",
    "Xpeo",
    "Xsux",
    "Yezi",
    "Yezidi",
    "Yi",
    "Yiii",
    "Zanabazar_Square",
    "Zanb",
    "Zinh",
    "Zyyy",
    "Zzzz",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn check(pattern: &str, flags: &str) -> bool {
        validate_literal(pattern, flags)
    }

    #[test]
    fn named_captures() {
        for flags in ["", "u", "v"] {
            let result = check("(?<n>a)|(?:(?<n>b)|(?<n>c))", flags);
            assert!(result);

            let result = check("(?:(?<n>a)|b)(?:c|(?<n>d))", flags);
            assert!(!result);

            let result = check("(?:(?<n>a)|(?<n>b))(?<n>c)", flags);
            assert!(!result);

            let result = check("(?<n>a)(?<n>b)|(?<n>c)", flags);
            assert!(!result);

            let result = check(r"\k<n>(?<n>a)|(?<n>b)", flags);
            assert!(result);

            let result = check(r"\k<n>(?<n>a)|(?<n>b)\k<m>", flags);
            assert!(!result);
        }
    }

    #[test]
    fn acorn_errors() {
        assert!(!check("a", "z"));

        assert!(!check("a", "gg"));

        assert!(!check("a", "uv"));

        assert!(!check("(?<n>.)(?<n>.)", ""));

        assert!(!check(r"\k", "u"));

        assert!(!check("(?<>a)", ""));

        assert!(!check("+", "u"));

        assert!(!check("a{", "u"));

        assert!(!check("a|+", ""));

        assert!(!check("a{2,1}", ""));

        assert!(!check("(", ""));

        assert!(!check("[", ""));

        assert!(!check(r"\1", "u"));

        assert!(!check(r"\k<a>", "u"));

        assert!(!check(r"\P{Basic_Emoji}", "v"));

        assert!(!check(r"\p{Basic_Emoji}", "u"));

        assert!(!check(r"\p{Foo}", "u"));

        assert!(!check("[z-a]", ""));

        assert!(!check("[a--b]", ""));

        assert!(!check(r"[\d-z]", "u"));

        assert!(!check(r"[\c0]", "u"));

        assert!(!check("[&&]", "v"));

        assert!(!check("[a&&&b]", "v"));

        assert!(!check("[a&&]", "v"));

        assert!(!check(r"[[^\q{ng}]]", "v"));

        assert!(!check(r"[^\q{ab}]", "v"));

        assert!(!check(r"\u{100000000}", "u"));

        assert!(!check(r"\ua", "u"));

        assert!(!check("(?<😀>a)", ""));

        assert!(!check(r"\u{110000}", "u"));

        assert!(!check(r"\c0", "u"));

        assert!(!check("\\", ""));

        assert!(!check("(?ii:.)", ""));

        assert!(!check("(?i-i:.)", ""));

        assert!(!check("(?a:.)", ""));

        assert!(!check("(?-:.)", ""));

        assert!(!check(")", "v"));
    }

    #[test]
    fn acorn_acceptance() {
        let result = check("a{9007199254740992}", "");
        assert!(result);

        let result = check("(?i:.)", "");
        assert!(result);

        let result = check("(?<n>a)|(?<n>b)", "");
        assert!(result);

        let result = check(r"\uD800", "u");
        assert!(result);

        let result = check(r"\c0", "");
        assert!(result);

        let result = check("a{", "");
        assert!(result);
    }
}
