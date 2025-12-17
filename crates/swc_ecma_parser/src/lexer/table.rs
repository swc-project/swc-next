//! Lookup table for byte handlers.
//!
//! Idea is taken from ratel.
//!
//! https://github.com/ratel-rust/ratel-core/blob/e55a1310ba69a3f5ce2a9a6eef643feced02ac08/ratel/src/lexer/mod.rs#L665

use either::Either;

use super::{LexResult, Lexer, pos_span};
use crate::{
    error::SyntaxError,
    lexer::{
        char_ext::CharExt,
        token::{Token, TokenValue},
    },
};

pub(super) type ByteHandler = fn(&mut Lexer<'_>) -> LexResult<Token>;

/// Lookup table mapping any incoming byte to a handler function defined below.
#[rustfmt::skip]
pub(super) static BYTE_HANDLERS: [ByteHandler; 256] = [
//   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F   //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    ERR, EXL, QOT, HSH, IDN, PRC, AMP, QOT, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, MOR, QST, // 3
    AT_, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, // 4
    IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, IDN, BTO, IDN, BTC, CRT, IDN, // 5
    TPL, L_A, L_B, L_C, L_D, L_E, L_F, L_G, L_H, L_I, L_J, L_K, L_L, L_M, L_N, L_O, // 6
    L_P, L_Q, L_R, L_S, L_T, L_U, L_V, L_W, L_X, L_Y, L_Z, BEO, PIP, BEC, TLD, ERR, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

const ERR: ByteHandler = |lexer| {
    let c = unsafe {
        // Safety: Byte handler is only called for non-last chracters
        lexer.input.peek().unwrap_unchecked()
    };

    let start = lexer.cur_pos();
    unsafe {
        // Safety: Byte handler is only called for non-last chracters
        lexer.input.bump_bytes(1);
    }
    lexer.error_span(
        pos_span(start),
        SyntaxError::UnexpectedChar { c: c as char },
    )?
};

/// Identifier and we know that this cannot be a keyword or known ident.
const IDN: ByteHandler = |lexer| lexer.read_ident_unknown();

const L_A: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "bstract" => Some(Token::Abstract),
        "s" => Some(Token::As),
        "wait" => Some(Token::Await),
        "sync" => Some(Token::Async),
        "ssert" => Some(Token::Assert),
        "sserts" => Some(Token::Asserts),
        "ny" => Some(Token::Any),
        "ccessor" => Some(Token::Accessor),
        _ => None,
    })
};

const L_B: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "reak" => Some(Token::Break),
        "oolean" => Some(Token::Boolean),
        "igint" => Some(Token::Bigint),
        _ => None,
    })
};

const L_C: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "ase" => Some(Token::Case),
        "atch" => Some(Token::Catch),
        "lass" => Some(Token::Class),
        "onst" => Some(Token::Const),
        "ontinue" => Some(Token::Continue),
        _ => None,
    })
};

const L_D: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "ebugger" => Some(Token::Debugger),
        "efault" => Some(Token::Default),
        "elete" => Some(Token::Delete),
        "o" => Some(Token::Do),
        "eclare" => Some(Token::Declare),
        _ => None,
    })
};

const L_E: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "lse" => Some(Token::Else),
        "num" => Some(Token::Enum),
        "xport" => Some(Token::Export),
        "xtends" => Some(Token::Extends),
        _ => None,
    })
};

const L_F: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "alse" => Some(Token::False),
        "inally" => Some(Token::Finally),
        "or" => Some(Token::For),
        "unction" => Some(Token::Function),
        "rom" => Some(Token::From),
        _ => None,
    })
};

const L_G: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "lobal" => Some(Token::Global),
        "et" => Some(Token::Get),
        _ => None,
    })
};

const L_H: ByteHandler = IDN;

const L_I: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "f" => Some(Token::If),
        "mport" => Some(Token::Import),
        "n" => Some(Token::In),
        "nstanceof" => Some(Token::InstanceOf),
        "s" => Some(Token::Is),
        "nfer" => Some(Token::Infer),
        "nterface" => Some(Token::Interface),
        "mplements" => Some(Token::Implements),
        "ntrinsic" => Some(Token::Intrinsic),
        _ => None,
    })
};

const L_J: ByteHandler = IDN;

const L_K: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "eyof" => Some(Token::Keyof),
        _ => None,
    })
};

const L_L: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "et" => Some(Token::Let),
        _ => None,
    })
};

const L_M: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "eta" => Some(Token::Meta),
        _ => None,
    })
};

const L_N: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "ew" => Some(Token::New),
        "ull" => Some(Token::Null),
        "umber" => Some(Token::Number),
        "ever" => Some(Token::Never),
        "amespace" => Some(Token::Namespace),
        _ => None,
    })
};

const L_O: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "f" => Some(Token::Of),
        "bject" => Some(Token::Object),
        "ut" => Some(Token::Out),
        "verride" => Some(Token::Override),
        _ => None,
    })
};

const L_P: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "ublic" => Some(Token::Public),
        "ackage" => Some(Token::Package),
        "rotected" => Some(Token::Protected),
        "rivate" => Some(Token::Private),
        _ => None,
    })
};

const L_Q: ByteHandler = IDN;

const L_R: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "eturn" => Some(Token::Return),
        "eadonly" => Some(Token::Readonly),
        "equire" => Some(Token::Require),
        _ => None,
    })
};

const L_S: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "uper" => Some(Token::Super),
        "tatic" => Some(Token::Static),
        "witch" => Some(Token::Switch),
        "ymbol" => Some(Token::Symbol),
        "et" => Some(Token::Set),
        "tring" => Some(Token::String),
        "atisfies" => Some(Token::Satisfies),
        _ => None,
    })
};

const L_T: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "his" => Some(Token::This),
        "hrow" => Some(Token::Throw),
        "rue" => Some(Token::True),
        "ypeof" => Some(Token::TypeOf),
        "ry" => Some(Token::Try),
        "ype" => Some(Token::Type),
        "arget" => Some(Token::Target),
        _ => None,
    })
};

const L_U: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "sing" => Some(Token::Using),
        "nique" => Some(Token::Unique),
        "ndefined" => Some(Token::Undefined),
        "nknown" => Some(Token::Unknown),
        _ => None,
    })
};

const L_V: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "ar" => Some(Token::Var),
        "oid" => Some(Token::Void),
        _ => None,
    })
};

const L_W: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "hile" => Some(Token::While),
        "ith" => Some(Token::With),
        _ => None,
    })
};

const L_X: ByteHandler = IDN;

const L_Y: ByteHandler = |lexer| {
    lexer.read_keyword_with(&|s| match s {
        "ield" => Some(Token::Yield),
        _ => None,
    })
};

const L_Z: ByteHandler = IDN;

/// `0`
const ZER: ByteHandler = |lexer| lexer.read_token_zero();

/// Numbers
const DIG: ByteHandler = |lexer| {
    debug_assert!(lexer.peek().is_some_and(|cur| cur != b'0'));
    lexer.read_number::<false, false>().map(|v| match v {
        Either::Left((value, raw)) => {
            lexer.state.set_token_value(TokenValue::Num { value, raw });
            Token::Num
        }
        Either::Right((value, raw)) => {
            lexer
                .state
                .set_token_value(TokenValue::BigInt { value, raw });
            Token::BigInt
        }
    })
};

/// String literals with `'` or `"`
const QOT: ByteHandler = |lexer| lexer.read_str_lit();

/// Unicode
const UNI: ByteHandler = |lexer| {
    let c = unsafe {
        // Safety: Byte handler is only called for non-last chracters
        lexer.input.peek_char().unwrap_unchecked()
    };

    // Identifier or keyword. '\uXXXX' sequences are allowed in
    // identifiers, so '\' also dispatches to that.
    if c == '\\' || c.is_ident_start() {
        return lexer.read_ident_unknown();
    }

    let start = lexer.cur_pos();
    unsafe {
        // Safety: Byte handler is only called for non-last chracters
        lexer.input.bump_bytes(c.len_utf8());
    }
    lexer.error_span(pos_span(start), SyntaxError::UnexpectedChar { c })?
};

/// `:`
const COL: ByteHandler = |lexer| lexer.read_token_colon();

/// `%`
const PRC: ByteHandler = |lexer| lexer.read_token_mul_mod::<false>();

/// `*`
const ATR: ByteHandler = |lexer| lexer.read_token_mul_mod::<true>();

/// `?`
const QST: ByteHandler = |lexer| lexer.read_token_question_mark();

/// `&`
const AMP: ByteHandler = |lexer| lexer.read_token_logical::<b'&'>();

/// `|`
const PIP: ByteHandler = |lexer| lexer.read_token_logical::<b'|'>();

macro_rules! single_char {
    ($name:ident, $c:literal, $token:ident) => {
        const $name: ByteHandler = |lexer| {
            lexer.bump(1);
            Ok(Token::$token)
        };
    };
}

single_char!(SEM, b';', Semi);
single_char!(COM, b',', Comma);

/// `\``
const TPL: ByteHandler = |lexer| lexer.read_token_back_quote();

single_char!(TLD, b'~', Tilde);
single_char!(AT_, b'@', At);

single_char!(PNO, b'(', LParen);
single_char!(PNC, b')', RParen);

single_char!(BTO, b'[', LBracket);
single_char!(BTC, b']', RBracket);

single_char!(BEO, b'{', LBrace);
single_char!(BEC, b'}', RBrace);

/// `^`
const CRT: ByteHandler = |lexer| {
    // Bitwise xor
    lexer.bump(1);
    Ok(if lexer.input.peek() == Some(b'=') {
        lexer.bump(1);
        Token::BitXorEq
    } else {
        Token::Caret
    })
};

/// `+`
const PLS: ByteHandler = |lexer| lexer.read_token_plus_minus::<b'+'>();

/// `-`
const MIN: ByteHandler = |lexer| lexer.read_token_plus_minus::<b'-'>();

/// `!`
const EXL: ByteHandler = |lexer| lexer.read_token_bang_or_eq::<b'!'>();

/// `=`
const EQL: ByteHandler = |lexer| lexer.read_token_bang_or_eq::<b'='>();

/// `.`
const PRD: ByteHandler = |lexer| lexer.read_token_dot();

/// `<`
const LSS: ByteHandler = |lexer| lexer.read_token_lt_gt::<b'<'>();

/// `>`
const MOR: ByteHandler = |lexer| lexer.read_token_lt_gt::<b'>'>();

/// `/`
const SLH: ByteHandler = |lexer| lexer.read_slash();

/// `#`
const HSH: ByteHandler = |lexer| lexer.read_token_number_sign();
