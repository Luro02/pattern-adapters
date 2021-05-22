#![feature(pattern)]
#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use pattern_adapters::logic::LogicPatternExt;
use core::str::pattern::{Pattern, Searcher};


fuzz_target!(|data: (&str, &str)| {
    // fuzzed code goes here
    let (haystack, needle) = data;
    utils::assert_integrity(haystack, needle.and(needle));

    // the following property should hold for the or patterns:
    let mut and_searcher = needle.and(needle).into_searcher(haystack);
    let mut searcher = needle.into_searcher(haystack);

    assert_searcher_eq!(and_searcher, searcher);
});
