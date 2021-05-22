#![feature(pattern)]
#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use pattern_adapters::logic::LogicPatternExt;
use core::str::pattern::{Pattern, Searcher, SearchStep};


fuzz_target!(|data: (&str, &str)| {
    // fuzzed code goes here
    let (haystack, pattern) = data;
    utils::assert_integrity(haystack, pattern.lor(pattern));
    utils::assert_integrity(haystack, pattern.ror(pattern));

    // the following property should hold for the or patterns:
    let mut lor_searcher = pattern.lor(pattern).into_searcher(haystack);
    let mut ror_searcher = pattern.ror(pattern).into_searcher(haystack);
    // TODO: they should be equal to this searcher as well
    // let mut searcher = pattern.into_searcher(haystack);

    assert_searcher_eq!(lor_searcher, ror_searcher);
});
