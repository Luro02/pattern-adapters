#![feature(pattern)]
#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use pattern_adapters::utils::Range;
use core::str::pattern::{Pattern, Searcher};
use core::ops;


fuzz_target!(|data: (ops::Range<usize>, ops::Range<usize>)| {
    let (range_left, range_right) = (Range::from(data.0), Range::from(data.1));

    // ranges intersection should be commutative:
    assert_eq!(
        range_left.intersect(range_right),
        range_right.intersect(range_left)
    );

    // the intersection of the range with itself is the range
    assert_eq!(range_left.intersect(range_left), Some(range_left));
    assert_eq!(range_right.intersect(range_right), Some(range_right));
});
