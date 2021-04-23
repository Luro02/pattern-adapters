use core::str::pattern::{Pattern, SearchStep, Searcher};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitedPattern<P>(P, usize);

impl<P> LimitedPattern<P> {
    #[must_use]
    pub const fn new(pattern: P, remaining: usize) -> Self { Self(pattern, remaining) }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for LimitedPattern<P> {
    type Searcher = LimitedSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        LimitedSearcher::new(self.0.into_searcher(haystack), self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitedSearcher<S> {
    searcher: S,
    remaining: usize,
    index: usize,
}

impl<S> LimitedSearcher<S> {
    #[must_use]
    const fn new(searcher: S, remaining: usize) -> Self {
        Self {
            searcher,
            remaining,
            index: 0,
        }
    }

    #[must_use]
    pub const fn index(&self) -> usize { self.index }

    #[must_use]
    pub const fn remaining(&self) -> usize { self.remaining }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for LimitedSearcher<S> {
    fn haystack(&self) -> &'a str { self.searcher.haystack() }

    fn next(&mut self) -> SearchStep {
        match self.searcher.next() {
            SearchStep::Match(start, end) => {
                // if there are any remaining matches
                if let Some(remaining) = self.remaining.checked_sub(1) {
                    self.remaining = remaining;
                    self.index = end;
                    return SearchStep::Match(start, end);
                }

                if self.index < self.haystack().len() {
                    self.index = self.haystack().len();

                    SearchStep::Reject(start, self.index)
                } else {
                    SearchStep::Done
                }
            }
            SearchStep::Reject(start, end) => {
                self.index = end;
                SearchStep::Reject(start, end)
            }
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
        let mut searcher = LimitedPattern::new('a', 4).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));
        assert_eq!(searcher.next(), SearchStep::Match(3, 4));
        assert_eq!(searcher.next(), SearchStep::Reject(4, 8));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_more_remaining_than_needed() {
        let haystack = "abab";
        let mut searcher = LimitedPattern::new("ab", 4).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 4));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_remaining_none() {
        let haystack = "this haystack will be completely rejected";
        let mut searcher = LimitedPattern::new(char::is_alphabetic, 0).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Reject(0, haystack.len()));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
