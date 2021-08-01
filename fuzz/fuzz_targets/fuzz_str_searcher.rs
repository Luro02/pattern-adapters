#![feature(pattern)]
#![no_main]

use libfuzzer_sys::fuzz_target;
use pattern_adapters::adapters::PatternExt;

mod utils;

fuzz_target!(|data: (&str, &str)| {
    let (haystack, needle) = data;

    if !haystack.is_empty() && !needle.is_empty() {
        utils::assert_integrity(haystack, needle);
    }
});
