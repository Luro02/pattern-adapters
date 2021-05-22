#![feature(pattern)]
#![no_main]

use core::cmp;
mod utils;

use libfuzzer_sys::fuzz_target;
use pattern_adapters::adapters::PatternExt;


fuzz_target!(|data: (&str, &str, usize, usize)| {
    // fuzzed code goes here
    let (haystack, needle, bound1, bound2) = data;
    let (min, max) = (cmp::min(bound1, bound2), cmp::max(bound1, bound2));
    utils::assert_integrity(haystack, PatternExt::repeat(needle, min, max));
    utils::assert_integrity(haystack, needle.chars().next().unwrap_or('0').repeat(min, max));
});

