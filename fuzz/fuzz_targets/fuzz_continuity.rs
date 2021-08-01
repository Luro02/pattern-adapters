#![feature(pattern)]
#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use pattern_adapters::adapters::PatternExt;

fuzz_target!(|data: (&str, &str, char, usize)| {
    let (haystack, needle_a, needle_b, limit) = data;

    utils::assert_integrity(haystack, needle_a.fuse());
    utils::assert_integrity(haystack, needle_b.fuse());

    utils::assert_integrity(haystack, needle_a.indexed());
    utils::assert_integrity(haystack, needle_b.indexed());

    utils::assert_integrity(haystack, needle_a.limit(limit));
    utils::assert_integrity(haystack, needle_b.limit(limit));

    utils::assert_integrity(haystack, needle_a.peekable());
    utils::assert_integrity(haystack, needle_b.peekable());

    // repeat is skipped, because it has its own fuzz target

    utils::assert_integrity(haystack, needle_a.simplify());
    utils::assert_integrity(haystack, needle_b.simplify());

    utils::assert_integrity(haystack, needle_a.skip(limit));
    utils::assert_integrity(haystack, needle_b.skip(limit));

    // TODO: stateful?
});
