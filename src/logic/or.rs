use core::str::pattern::{Pattern, SearchStep, Searcher};

use crate::utils::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrPattern<A, B>(A, B);

impl<A, B> OrPattern<A, B> {
    #[must_use]
    pub const fn new(a: A, b: B) -> Self {
        Self(a, b)
    }
}

impl<'a, A: Pattern<'a>, B: Pattern<'a>> Pattern<'a> for OrPattern<A, B> {
    type Searcher = OrSearcher<A::Searcher, B::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        OrSearcher {
            a: self.0.into_searcher(haystack),
            b: self.1.into_searcher(haystack),
            index: 0,
            next_match: None,
            cached_match: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CachedMatch {
    A(usize, usize),
    B(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrSearcher<A, B> {
    a: A,
    b: B,
    index: usize,
    next_match: Option<(usize, usize)>,
    cached_match: Option<CachedMatch>,
}

impl<'a, A: Searcher<'a>, B: Searcher<'a>> OrSearcher<A, B> {
    pub fn index(&self) -> usize {
        self.index
    }

    fn match_step(&mut self, range: Range) -> SearchStep {
        let index = self.index;

        if index < range.start() {
            self.next_match = Some(range.into());
            self.index = range.start();
            return SearchStep::Reject(index, range.start());
        }

        self.index = range.end();

        // TODO: what if this is not the case?
        assert_eq!(index, range.start());

        SearchStep::Match(range.start(), range.end())
    }

    #[allow(clippy::type_complexity)]
    fn next_matches(&mut self) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
        match self.cached_match.take() {
            Some(CachedMatch::A(start, end)) => (Some((start, end)), self.b.next_match()),
            Some(CachedMatch::B(start, end)) => (self.a.next_match(), Some((start, end))),
            None => (self.a.next_match(), self.b.next_match()),
        }
    }
}

unsafe impl<'a, A: Searcher<'a>, B: Searcher<'a>> Searcher<'a> for OrSearcher<A, B> {
    fn haystack(&self) -> &'a str {
        // SAFETY: if this is not the case, we would have undefined behavior
        debug_assert_eq!(self.a.haystack(), self.b.haystack());
        self.a.haystack()
    }

    fn next(&mut self) -> SearchStep {
        // One might have to reject a range first, before one can match.
        // This if will be called if the last step was reject
        if let Some((start, end)) = self.next_match.take() {
            self.index = end;
            return SearchStep::Match(start, end);
        }

        match self.next_matches() {
            (Some(a), Some(b)) => {
                let match_range = {
                    let (a, b) = (Range::from(a), Range::from(b));

                    // NOTE: a == b is implied by a.intersect(b).is_some()
                    if a.intersect(b).is_some() || a == b {
                        a
                    } else if a.start() < b.start() {
                        self.cached_match = Some(CachedMatch::B(b.start(), b.end()));
                        a
                    } else if a.start() > b.start() {
                        // the ranges are disjoint, so one match has to be cached!
                        self.cached_match = Some(CachedMatch::A(a.start(), a.end()));
                        b
                    } else {
                        unreachable!()
                    }
                };

                self.match_step(match_range)
            }
            (Some(range), None) | (None, Some(range)) => self.match_step(range.into()),
            (None, None) => {
                let haystack_length = self.haystack().len();
                if self.index == haystack_length {
                    SearchStep::Done
                } else {
                    let start = self.index;
                    self.index = haystack_length;
                    SearchStep::Reject(start, self.index)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_searcher_same_size() {
        let haystack = "a b c a b b a a b";
        let mut searcher = OrPattern::new('a', 'b').into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));
        assert_eq!(searcher.next(), SearchStep::Reject(3, 6));
        assert_eq!(searcher.next(), SearchStep::Match(6, 7));
        assert_eq!(searcher.next(), SearchStep::Reject(7, 8));
        assert_eq!(searcher.next(), SearchStep::Match(8, 9));
        assert_eq!(searcher.next(), SearchStep::Reject(9, 10));
        assert_eq!(searcher.next(), SearchStep::Match(10, 11));
        assert_eq!(searcher.next(), SearchStep::Reject(11, 12));
        assert_eq!(searcher.next(), SearchStep::Match(12, 13));
        assert_eq!(searcher.next(), SearchStep::Reject(13, 14));
        assert_eq!(searcher.next(), SearchStep::Match(14, 15));
        assert_eq!(searcher.next(), SearchStep::Reject(15, 16));
        assert_eq!(searcher.next(), SearchStep::Match(16, 17));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_searcher_left_smaller() {
        let haystack = "abcaabbaab";
        let mut searcher = OrPattern::new("a", "ab").into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 3));
        assert_eq!(searcher.next(), SearchStep::Match(3, 4));
        assert_eq!(searcher.next(), SearchStep::Match(4, 5));
        assert_eq!(searcher.next(), SearchStep::Reject(5, 7));
        assert_eq!(searcher.next(), SearchStep::Match(7, 8));
        assert_eq!(searcher.next(), SearchStep::Match(8, 9));
        assert_eq!(searcher.next(), SearchStep::Reject(9, 10));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_searcher_right_smaller() {
        let haystack = "abcaabbaab";
        let mut searcher = OrPattern::new("ab", "a").into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 2));
        assert_eq!(searcher.next(), SearchStep::Reject(2, 3));
        assert_eq!(searcher.next(), SearchStep::Match(3, 4));
        assert_eq!(searcher.next(), SearchStep::Match(4, 6));
        assert_eq!(searcher.next(), SearchStep::Reject(6, 7));
        assert_eq!(searcher.next(), SearchStep::Match(7, 8));
        assert_eq!(searcher.next(), SearchStep::Match(8, 10));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_searcher_unicode_right_larger() {
        let haystack = "\nMäry häd ä little lämb\n\r\nLittle lämb\n";
        let mut searcher = OrPattern::new("\r\n", "\n").into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 27));
        assert_eq!(searcher.next(), SearchStep::Match(27, 28));
        assert_eq!(searcher.next(), SearchStep::Match(28, 30));
        assert_eq!(searcher.next(), SearchStep::Reject(30, 42));
        assert_eq!(searcher.next(), SearchStep::Match(42, 43));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_searcher_empty_string() {
        let haystack = "";
        let mut searcher = OrPattern::new("abc", "ab").into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
