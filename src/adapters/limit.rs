use core::str::pattern::{Pattern, SearchStep, Searcher};

/// Limits the [`Pattern`] to match at most `n` times in total.
///
/// # Example
///
/// ```
/// #![feature(pattern)]
/// use core::str::pattern::{SearchStep, Searcher, Pattern};
/// use pattern_adapters::adapters::PatternExt;
///
/// let mut matches = "12345678".matches((|c: char| c.is_ascii_digit()).limit(2));
///
/// assert_eq!(matches.next(), Some("1"));
/// assert_eq!(matches.next(), Some("2"));
/// // there are more than two digits in the string,
/// // but because of the limit pattern only two are returned
/// assert_eq!(matches.next(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitPattern<P>(P, usize);

impl<P> LimitPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P, n: usize) -> Self {
        Self(pattern, n)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for LimitPattern<P> {
    type Searcher = LimitSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        LimitSearcher::new(self.0.into_searcher(haystack), self.1)
    }
}

/// A [`Searcher`] that returns at most `n` [`SearchStep::Match`]es.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitSearcher<S> {
    searcher: S,
    remaining: usize,
}

impl<'a, S: Searcher<'a>> LimitSearcher<S> {
    #[must_use]
    pub(super) fn new(searcher: S, remaining: usize) -> Self {
        Self {
            searcher,
            remaining,
        }
    }
}

impl<S> LimitSearcher<S> {
    /// Returns the maximum number of remaining matches.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.remaining
    }

    /// Returns true, if there are no more remaining matches.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.remaining() == 0
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for LimitSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        match self.searcher.next() {
            SearchStep::Match(start, end) => {
                if self.is_exhausted() {
                    SearchStep::Reject(start, end)
                } else {
                    self.remaining -= 1;
                    SearchStep::Match(start, end)
                }
            }
            SearchStep::Reject(start, end) => SearchStep::Reject(start, end),
            SearchStep::Done => SearchStep::Done,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple_search() {
        let haystack = "aaaaaaaa";
        let mut searcher = LimitPattern::new('a', 4).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));
        assert_eq!(searcher.next(), SearchStep::Match(3, 4));
        assert_eq!(searcher.next(), SearchStep::Reject(4, 5));
        assert_eq!(searcher.next(), SearchStep::Reject(5, 6));
        assert_eq!(searcher.next(), SearchStep::Reject(6, 7));
        assert_eq!(searcher.next(), SearchStep::Reject(7, 8));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_more_remaining_than_needed() {
        let haystack = "abab";
        let mut searcher = LimitPattern::new("ab", 4).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 4));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_remaining_zero() {
        let haystack = "this haystack will be completely rejected";
        let mut searcher = LimitPattern::new(char::is_alphabetic, 0).into_searcher(haystack);

        for i in 0..haystack.len() {
            assert_eq!(searcher.next(), SearchStep::Reject(i, i + 1));
        }
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    #[ignore = "upstream issue"]
    fn test_fuzzer_failure_01() {
        // TODO: this searcher does not behave correctly (underlying searcher)
        let haystack = "\u{e}";
        let needle = "";
        let limit = 11646590111356813473;

        let mut searcher = LimitPattern::new(needle, limit).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Reject(0, 1));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    #[ignore = "upstream issue: rust-lang/rust#85462"]
    fn test_fuzzer_failure_02() {
        // TODO: https://github.com/rust-lang/rust/issues/85462
        let haystack = "";
        let needle = "";
        let limit = 0;

        let mut searcher = LimitPattern::new(needle, limit).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Reject(0, 0));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
