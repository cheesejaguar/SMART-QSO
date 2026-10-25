//! Beacon content filter: text it passes meets the rules that can be
//! stated without the word lists
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::filter::{self, MAX_LEN, PUNCTUATION, Verdict};

fuzz_target!(|data: &[u8]| {
    if filter::check(data) == Verdict::Ok {
        assert!(!data.is_empty() && data.len() <= MAX_LEN);
        assert!(data.iter().all(|c| c.is_ascii_alphanumeric() || PUNCTUATION.as_bytes().contains(c)));
        let lower: Vec<u8> = data.to_ascii_lowercase();
        for mark in filter::URL_MARKS {
            assert!(!lower.windows(mark.len()).any(|w| w == mark.as_bytes()));
        }
        for word in data.split(|c| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()) {
            assert!(word.len() <= filter::MAX_WORD);
        }
    }
});
