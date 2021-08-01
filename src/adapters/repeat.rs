use core::str::pattern::{Pattern, SearchStep, Searcher};

use super::PeekableSearcher;

// TODO: make it possible to specify, whether this is greedy or not
// TODO: if it is greedy (current implementation) it will try to match as much as possible
// TODO: if it is not it will return after the minimum number of matches have been found
// TODO: (maybe one could split this pattern up into two patterns, one for min and another for max?)
// TODO: max would be something like limit, but limit limits the total number of matches, while max would limit the number
// TODO: of consecutive matches
//
// TODO: it would be awesome if the matches would not be merged (this could be implemented by keeping track of how many have been matched already?)

/// Repeatedly matches a [`Pattern`].
///
/// # Examples
///
/// Matching a number one or two times:
///
/// ```
/// #![feature(pattern)]
/// use core::str::pattern::{SearchStep, Searcher, Pattern};
/// use pattern_adapters::adapters::RepeatPattern;
///
/// // the string one wants to search through:
/// let haystack = "123SD98";
/// // the pattern with which one can search:
/// let pattern = RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 2);
/// let mut searcher = pattern.into_searcher(haystack);
///
/// assert_eq!(searcher.next_match(), Some((0, 2))); // matches "12"
/// assert_eq!(searcher.next_match(), Some((2, 3))); // matches "3"
/// assert_eq!(searcher.next_match(), Some((5, 7))); // matches "98"
/// ```
///
/// [`Pattern`]: core::str::pattern::Pattern
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepeatPattern<P> {
    pattern: P,
    min: usize,
    max: usize,
}

impl<P> RepeatPattern<P> {
    #[must_use]
    pub const fn new(pattern: P, min: usize, max: usize) -> Self {
        Self { pattern, min, max }
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for RepeatPattern<P> {
    type Searcher = RepeatSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        RepeatSearcher::new(self.pattern.into_searcher(haystack), self.min, self.max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatSearcher<S> {
    searcher: PeekableSearcher<S>,
    min: usize,
    max: usize,
}

impl<S> RepeatSearcher<S> {
    #[must_use]
    pub(super) fn new(searcher: S, min: usize, max: usize) -> Self {
        Self {
            searcher: PeekableSearcher::new(searcher),
            min,
            max,
        }
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for RepeatSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        let step = self.searcher.next();

        if let SearchStep::Match(start, end) = step {
            let mut end = end;
            let mut matches = 1;

            for _ in 1..self.max {
                if let SearchStep::Match(next_start, next_end) = self.searcher.peek() {
                    // check that the next match starts at the end of the previous match:
                    if next_start == end {
                        // advance the searcher:
                        self.searcher.next();
                        matches += 1;
                        end = next_end;
                    } else {
                        // discontinuity between the matches

                        // check that enough has been matched to return something:
                        if matches <= self.max && matches >= self.min {
                            return SearchStep::Match(start, end);
                        }

                        return SearchStep::Reject(start, next_start);
                    }
                } else {
                    break;
                }
            }

            if matches < self.min {
                return SearchStep::Reject(start, end);
            }

            SearchStep::Match(start, end)
        } else {
            step
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // TODO: this should be used in all tests!
    fn assert_continuity<'a, P: Pattern<'a>>(haystack: &'a str, pattern: P) {
        let mut searcher = pattern.into_searcher(haystack);

        let mut last_end = 0;
        loop {
            if let SearchStep::Match(start, end) | SearchStep::Reject(start, end) = searcher.next()
            {
                assert_eq!(last_end, start);
                last_end = end;
            } else {
                break;
            }
        }

        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_continuity() {
        // TODO: add more tests/strings
        assert_continuity(
            "1 2 3 4aäalpqkdpawdjap 1320pjf.-as ,m",
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 1),
        );
    }

    #[test]
    fn test_simple() {
        let haystack = "0123456789";
        let mut searcher =
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 5).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 5));
        assert_eq!(searcher.next(), SearchStep::Match(5, 10));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_exactly_one() {
        let haystack = "012";
        let mut searcher =
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 1).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_unicode() {
        let haystack = "0 äö 13 hello 2";
        let mut searcher =
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 3).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 2));
        assert_eq!(searcher.next(), SearchStep::Reject(2, 4));
        assert_eq!(searcher.next(), SearchStep::Reject(4, 6));
        assert_eq!(searcher.next(), SearchStep::Reject(6, 7));
        assert_eq!(searcher.next(), SearchStep::Match(7, 9)); // TODO: this may be split up?
        assert_eq!(searcher.next(), SearchStep::Reject(9, 10));
        assert_eq!(searcher.next(), SearchStep::Reject(10, 11));
        assert_eq!(searcher.next(), SearchStep::Reject(11, 12));
        assert_eq!(searcher.next(), SearchStep::Reject(12, 13));
        assert_eq!(searcher.next(), SearchStep::Reject(13, 14));
        assert_eq!(searcher.next(), SearchStep::Reject(14, 15));
        assert_eq!(searcher.next(), SearchStep::Reject(15, 16));
        assert_eq!(searcher.next(), SearchStep::Match(16, 17));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_fuzzer_failure_01() {
        let haystack = concat!(
            "\u{1c},\u{0}\u{0}\u{0}\u{0}\u{0}=\u{0}\u{0}",
            "\u{0}\u{0}\u{1e}\u{0}\u{0},\u{0}\u{0}\u{0}\u{0}",
            "\u{0}\u{0}#\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}#,,\u{0},\u{0},",
            "\u{0},,\u{0}\u{0}\u{0}=\u{0}\u{0}\u{10}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0},,",
            ",\u{0}\u{0}\u{1a}\u{1c};\u{0}\u{1a}\u{1}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{4}",
            "\u{0}\u{1c}\u{0}\u{1a},\u{0}\u{4}\u{0}\u{1c}\u{0}\u{1a}\u{1e}\u{0}/\u{0}#;\u{0}\u{0}J\u{0}#,",
            ",\u{0},\u{0}\u{1a},\u{0}\u{4}\u{0}\u{1c}\u{0}\u{1a}\u{1e}\u{0}/\u{0}\u{1c}\u{0}\u{1a}\u{1c};",
            "\u{0}\u{0}J\u{0}#,,\u{0},\u{0},\u{1c}\u{0}\u{1a},\u{0}\u{4}\u{0}\u{1c}\u{0}\u{1a}\u{1e}\u{0}/",
            "\u{0}\u{1c}\u{0}\u{1a}\u{1c};\u{0}\u{7},\u{0}$\u{0}\u{0}\u{0}\u{0}\u{0}=)\u{0}\u{0}\u{0}\u{0}\u{0}"
        );

        let pattern = RepeatPattern::new('\u{0}', 169618582, 3170534138692239568);
        assert_continuity(haystack, pattern);
    }
}
