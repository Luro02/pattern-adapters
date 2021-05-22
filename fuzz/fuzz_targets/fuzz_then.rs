#![feature(pattern)]
#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use pattern_adapters::adapters::PatternExt;
use core::str::pattern::{Pattern, Searcher};


fuzz_target!(|data: (&str, &str, char)| {
    let (haystack, needle_a, needle_b) = data;

    utils::assert_integrity(haystack, needle_a.then(needle_b));
    utils::assert_integrity(haystack, needle_b.then(needle_a));
    utils::assert_integrity(haystack, needle_b.then(needle_b));

    // 'a'.then('b') should be equivalent to "ab"
    let equivalent_str = format!("{}{}", needle_b, needle_b);

    let mut searcher = equivalent_str.into_searcher(haystack);
    let mut then_searcher = needle_b.then(needle_b).into_searcher(haystack);

    // at least the matches should be equivalent
    assert_matches_eq!(searcher, then_searcher);
});
