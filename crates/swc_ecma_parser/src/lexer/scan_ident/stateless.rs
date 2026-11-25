use super::TokenTag;
#[cfg(target_arch = "x86_64")]
use super::imp::scan_ascii_identifier_or_keyword as scan;
#[cfg(not(target_arch = "x86_64"))]
use super::scan_ascii_identifier_or_keyword_slow as scan;

#[derive(Clone, Debug)]
pub(in crate::lexer) struct IdentifierScanner;

impl IdentifierScanner {
    #[inline(always)]
    pub(in crate::lexer) fn new(_source: &[u8]) -> Self {
        Self
    }

    #[inline(always)]
    pub(in crate::lexer) fn scan(&mut self, source: &[u8], start: u32) -> Option<(u32, TokenTag)> {
        scan(source, start)
    }
}
