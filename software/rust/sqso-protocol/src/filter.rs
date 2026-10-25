//! Beacon text content filter (SW-12; `docs/BEACON_SPEC.md` §6).
//!
//! Every beacon text that did not come from the OBC's own templates passes
//! this filter before it is transmitted. It enforces the 47 CFR 97.113
//! limits that can be checked mechanically, in this order:
//!
//! - LENGTH: 1 to [`MAX_LEN`] characters
//! - CHARSET: letters, digits, space and `. , ! ? ' " : ; ( ) / - &` only
//! - COMMERCIAL: no URL fragments (`http`, `www.`, `.com`, `.net`, `.org`)
//!
//! then word by word (a word is a maximal run of ASCII letters and digits):
//!
//! - ENCODED: longer than [`MAX_WORD`]
//! - CALLSIGN: shaped like a call sign but not this station's, or an SSID
//!   after this station's call that is not its own
//! - NUMERIC: digits other than "73", "1U" and this station's call and SSID
//! - ENCODED: five or more letters and no vowel, eight or more hex digits
//!   only, or more than two lower-to-upper case changes
//! - PROFANITY / COMMERCIAL: a word from [`PROFANITY`] / [`COMMERCIAL`]
//!
//! The OBC (through `sqso-ffi`) and the Jetson (through the Python module)
//! run this one implementation; both test suites read the shared vectors in
//! `software/flight/tests/vectors/beacon_filter_vectors.txt`.

use core::ffi::CStr;

use crate::station;

/// Longest beacon text: what fits beside the telemetry fields
pub const MAX_LEN: usize = 176;
/// Longest word
pub const MAX_WORD: usize = 20;

/// Filter verdict: OK or the first rule broken
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Verdict {
    /// Passes every rule
    Ok = 0,
    /// Empty or too long
    Length = 1,
    /// A character outside the allowed set
    Charset = 2,
    /// Looks encoded (97.113(a)(4))
    Encoded = 3,
    /// A number the OBC did not write
    Numeric = 4,
    /// Another station's call sign or SSID (97.113(a)(4))
    Callsign = 5,
    /// Obscene or indecent (97.113(a)(4))
    Profanity = 6,
    /// Business or advertising (97.113(a)(3))
    Commercial = 7,
}

impl Verdict {
    /// Name used in the shared vectors ("OK", "LENGTH", ...)
    pub const fn name(self) -> &'static str {
        match self {
            Verdict::Ok => "OK",
            Verdict::Length => "LENGTH",
            Verdict::Charset => "CHARSET",
            Verdict::Encoded => "ENCODED",
            Verdict::Numeric => "NUMERIC",
            Verdict::Callsign => "CALLSIGN",
            Verdict::Profanity => "PROFANITY",
            Verdict::Commercial => "COMMERCIAL",
        }
    }

    /// [`Verdict::name`] as a C string
    pub const fn c_name(self) -> &'static CStr {
        match self {
            Verdict::Ok => c"OK",
            Verdict::Length => c"LENGTH",
            Verdict::Charset => c"CHARSET",
            Verdict::Encoded => c"ENCODED",
            Verdict::Numeric => c"NUMERIC",
            Verdict::Callsign => c"CALLSIGN",
            Verdict::Profanity => c"PROFANITY",
            Verdict::Commercial => c"COMMERCIAL",
        }
    }

    /// From the C value
    pub const fn from_raw(raw: u32) -> Option<Verdict> {
        match raw {
            0 => Some(Verdict::Ok),
            1 => Some(Verdict::Length),
            2 => Some(Verdict::Charset),
            3 => Some(Verdict::Encoded),
            4 => Some(Verdict::Numeric),
            5 => Some(Verdict::Callsign),
            6 => Some(Verdict::Profanity),
            7 => Some(Verdict::Commercial),
            _ => None,
        }
    }
}

/// Obscene or indecent words (lower case, whole-word match)
pub const PROFANITY: &[&CStr] = &[
    c"ass",
    c"asshole",
    c"bastard",
    c"bitch",
    c"bollocks",
    c"bullshit",
    c"cock",
    c"cocksucker",
    c"crap",
    c"cunt",
    c"damn",
    c"dick",
    c"dickhead",
    c"dyke",
    c"fag",
    c"faggot",
    c"fuck",
    c"fucked",
    c"fucker",
    c"fucking",
    c"goddamn",
    c"motherfucker",
    c"nigga",
    c"nigger",
    c"piss",
    c"pissed",
    c"prick",
    c"pussy",
    c"retard",
    c"shit",
    c"shitty",
    c"slut",
    c"spic",
    c"twat",
    c"wank",
    c"wanker",
    c"whore",
];

/// Business, advertising or pecuniary words (lower case, whole-word match)
pub const COMMERCIAL: &[&CStr] = &[
    c"advert",
    c"advertise",
    c"advertisement",
    c"bitcoin",
    c"buy",
    c"buying",
    c"cheap",
    c"coupon",
    c"crypto",
    c"deal",
    c"deals",
    c"discount",
    c"dollar",
    c"dollars",
    c"invest",
    c"investment",
    c"price",
    c"prices",
    c"profit",
    c"promo",
    c"promotion",
    c"purchase",
    c"sale",
    c"sales",
    c"sell",
    c"selling",
    c"shop",
    c"sponsor",
    c"sponsored",
    c"subscribe",
    c"usd",
];

/// URL fragments, matched anywhere in the text
pub const URL_MARKS: &[&str] = &["http", "www.", ".com", ".net", ".org"];

/// Punctuation allowed besides letters and digits
pub const PUNCTUATION: &str = " .,!?'\":;()/-&";

fn allowed(c: u8) -> bool {
    c.is_ascii_alphanumeric() || PUNCTUATION.as_bytes().contains(&c)
}

fn is_vowel(c: u8) -> bool {
    matches!(c.to_ascii_lowercase(), b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
}

fn word_is(word: &[u8], lit: &[u8]) -> bool {
    word.eq_ignore_ascii_case(lit)
}

fn in_list(word: &[u8], list: &[&CStr]) -> bool {
    list.iter().any(|w| word_is(word, w.to_bytes()))
}

fn is_own_call(word: &[u8]) -> bool {
    word_is(word, station::CALLSIGN.as_bytes())
}

/// Prefix (letter, two letters, digit-letter or letter-digit), one digit,
/// then one to four letters: the ITU call-sign pattern
fn is_callsign_shape(word: &[u8]) -> bool {
    let letters = word.iter().rev().take_while(|c| c.is_ascii_alphabetic()).count();
    let Some(rest) = word.len().checked_sub(letters) else { return false };
    let Some((digit, prefix)) = word.get(..rest).and_then(<[u8]>::split_last) else { return false };
    if !(1..=4).contains(&letters) || !digit.is_ascii_digit() {
        return false;
    }
    match prefix {
        [a] => a.is_ascii_alphabetic(),
        [a, b] => a.is_ascii_alphabetic() || b.is_ascii_alphabetic(),
        _ => false,
    }
}

/// Digits only, at most two of them, equal to this station's SSID
fn check_ssid(word: &[u8]) -> Verdict {
    if !word.iter().all(u8::is_ascii_digit) || word.len() > 2 {
        return Verdict::Numeric;
    }
    let value = word.iter().fold(0u8, |v, d| v * 10 + (d - b'0'));
    if value == station::SSID { Verdict::Ok } else { Verdict::Callsign }
}

fn looks_encoded(word: &[u8]) -> bool {
    let all_alpha = word.iter().all(u8::is_ascii_alphabetic);
    let all_hex = word.iter().all(u8::is_ascii_hexdigit);
    let vowel = word.iter().any(|c| is_vowel(*c));
    let case_changes =
        word.windows(2).filter(|w| matches!(w, [a, b] if a.is_ascii_lowercase() && b.is_ascii_uppercase())).count();
    (all_alpha && word.len() >= 5 && !vowel) || (all_hex && word.len() >= 8) || case_changes > 2
}

fn check_word(word: &[u8], after_own_call: bool) -> Verdict {
    if word.len() > MAX_WORD {
        return Verdict::Encoded;
    }
    if after_own_call && word.first().is_some_and(u8::is_ascii_digit) {
        return check_ssid(word);
    }
    if is_callsign_shape(word) && !is_own_call(word) {
        return Verdict::Callsign;
    }
    if word.iter().any(u8::is_ascii_digit) && !word_is(word, b"73") && !word_is(word, b"1u") && !is_own_call(word) {
        return Verdict::Numeric;
    }
    if looks_encoded(word) {
        return Verdict::Encoded;
    }
    if in_list(word, PROFANITY) {
        return Verdict::Profanity;
    }
    if in_list(word, COMMERCIAL) {
        return Verdict::Commercial;
    }
    Verdict::Ok
}

fn contains_ignore_case(text: &[u8], lit: &[u8]) -> bool {
    text.windows(lit.len()).any(|w| w.eq_ignore_ascii_case(lit))
}

/// Check a beacon text; `Verdict::Ok` or the first rule it breaks
pub fn check(text: &[u8]) -> Verdict {
    if text.is_empty() || text.len() > MAX_LEN {
        return Verdict::Length;
    }
    if !text.iter().all(|c| allowed(*c)) {
        return Verdict::Charset;
    }
    if URL_MARKS.iter().any(|m| contains_ignore_case(text, m.as_bytes())) {
        return Verdict::Commercial;
    }

    let mut pos = 0;
    let mut prev: Option<(usize, bool)> = None; // end of the previous word, was it our call
    while let Some(offset) = text.get(pos..).and_then(|t| t.iter().position(u8::is_ascii_alphanumeric)) {
        let start = pos + offset;
        let rest = text.get(start..).unwrap_or(&[]);
        let len = rest.iter().take_while(|c| c.is_ascii_alphanumeric()).count();
        let word = rest.get(..len).unwrap_or(&[]);
        let after_own = matches!(prev, Some((end, true)) if start == end + 1 && text.get(end) == Some(&b'-'));
        let verdict = check_word(word, after_own);
        if verdict != Verdict::Ok {
            return verdict;
        }
        pos = start + len;
        prev = Some((pos, is_own_call(word)));
    }
    Verdict::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTORS: &str = include_str!("../../../flight/tests/vectors/beacon_filter_vectors.txt");

    #[test]
    fn shared_vectors() {
        let mut count = 0;
        for line in VECTORS.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let (expected, text) = line.split_once('|').unwrap();
            let verdict = check(text.as_bytes());
            assert_eq!(verdict.name(), expected, "{text:?}");
            assert_eq!(verdict.c_name().to_str(), Ok(expected));
            assert_eq!(Verdict::from_raw(verdict as u32), Some(verdict));
            count += 1;
        }
        assert!(count > 40, "vectors not read");
    }

    #[test]
    fn every_list_word_is_caught() {
        for w in PROFANITY {
            assert_eq!(check(w.to_bytes()), Verdict::Profanity, "{w:?}");
        }
        for w in COMMERCIAL {
            assert_eq!(check(w.to_bytes()), Verdict::Commercial, "{w:?}");
        }
    }

    /// LENGTH: 1 to MAX_LEN characters, both ends inclusive
    #[test]
    fn length_limit_is_inclusive() {
        let mut text = [0u8; MAX_LEN + 1];
        for (i, c) in text.iter_mut().enumerate() {
            *c = b"ab "[i % 3];
        }
        text[MAX_LEN - 1] = b'.';
        text[MAX_LEN] = b'.';
        assert_eq!(check(&text[..MAX_LEN]), Verdict::Ok);
        assert_eq!(check(&text), Verdict::Length);
        assert_eq!(check(&text[..1]), Verdict::Ok);
    }

    /// CHARSET: '|' separates the telemetry fields, so it is refused alone
    /// (the shared vector with '|' also holds '=')
    #[test]
    fn charset_excludes_the_field_separator() {
        for text in [&b"Hello | world"[..], b"a|b", b"Pay $5", b"me @ home"] {
            assert_eq!(check(text), Verdict::Charset, "{text:?}");
        }
    }

    /// COMMERCIAL: URL fragments match in any letter case
    #[test]
    fn url_marks_in_any_letter_case() {
        for text in [&b"See WWW.EXAMPLE.ORG today"[..], b"Visit HTTP now", b"Details at Example.Com", b"x.NeT"] {
            assert_eq!(check(text), Verdict::Commercial, "{}", core::str::from_utf8(text).unwrap());
        }
    }

    #[test]
    fn ssid_rules() {
        assert_eq!(check(b"de KK6GDB-11"), Verdict::Ok);
        assert_eq!(check(b"de KK6GDB-7"), Verdict::Callsign);
        assert_eq!(check(b"de KK6GDB-123"), Verdict::Numeric);
        assert_eq!(check(b"de W1AW"), Verdict::Callsign);
        assert_eq!(check(b"Hello\0"), Verdict::Charset);
    }

    #[test]
    fn from_raw_inverts_the_c_value() {
        let all = [
            Verdict::Ok,
            Verdict::Length,
            Verdict::Charset,
            Verdict::Encoded,
            Verdict::Numeric,
            Verdict::Callsign,
            Verdict::Profanity,
            Verdict::Commercial,
        ];
        for (raw, verdict) in (0u32..).zip(all) {
            assert_eq!(Verdict::from_raw(raw), Some(verdict));
        }
        assert_eq!(Verdict::from_raw(8), None);
        assert_eq!(Verdict::from_raw(u32::MAX), None);
    }

    #[test]
    fn text_of_max_len_passes() {
        // "hi hi hi ..."
        let mut text = [0u8; MAX_LEN + 1];
        for (i, c) in text.iter_mut().enumerate() {
            *c = b"hi "[i % 3];
        }
        assert_eq!(check(&text[..MAX_LEN]), Verdict::Ok);
        assert_eq!(check(&text), Verdict::Length);
    }

    #[test]
    fn word_of_max_word_passes() {
        let word = b"abcdefghijklmnopqrstu";
        assert_eq!(word.len(), MAX_WORD + 1);
        assert_eq!(check(&word[..MAX_WORD]), Verdict::Ok);
        assert_eq!(check(word), Verdict::Encoded);
    }

    #[test]
    fn two_case_changes_pass_three_are_encoded() {
        assert_eq!(check(b"iPoD"), Verdict::Ok);
        assert_eq!(check(b"iPoDsT"), Verdict::Encoded);
    }

    #[test]
    fn call_sign_shape_needs_one_to_four_final_letters() {
        assert_eq!(check(b"W1ABCD"), Verdict::Callsign);
        assert_eq!(check(b"W1ABCDE"), Verdict::Numeric); // five letters
        assert_eq!(check(b"A12"), Verdict::Numeric); // no final letter
    }
}
