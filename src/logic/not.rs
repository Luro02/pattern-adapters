use core::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, SearchStep, Searcher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotPattern<A>(A);

impl<A> NotPattern<A> {
    /// Constructs a new `NotPattern` with the provided [`Pattern`].
    #[must_use]
    pub(super) const fn new(a: A) -> Self {
        Self(a)
    }
}

impl<'a, A: Pattern<'a>> Pattern<'a> for NotPattern<A> {
    type Searcher = NotSearcher<A::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        NotSearcher(self.0.into_searcher(haystack))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotSearcher<S>(pub(super) S);

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for NotSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.0.haystack()
    }

    fn next(&mut self) -> SearchStep {
        match self.0.next() {
            SearchStep::Match(start, end) => SearchStep::Reject(start, end),
            SearchStep::Reject(start, end) => SearchStep::Match(start, end),
            SearchStep::Done => SearchStep::Done,
        }
    }
}

unsafe impl<'a, S: ReverseSearcher<'a>> ReverseSearcher<'a> for NotSearcher<S> {
    fn next_back(&mut self) -> SearchStep {
        match self.0.next() {
            SearchStep::Match(start, end) => SearchStep::Reject(start, end),
            SearchStep::Reject(start, end) => SearchStep::Match(start, end),
            SearchStep::Done => SearchStep::Done,
        }
    }

    fn next_match_back(&mut self) -> Option<(usize, usize)> {
        self.0.next_reject_back()
    }

    fn next_reject_back(&mut self) -> Option<(usize, usize)> {
        self.0.next_match_back()
    }
}

impl<'a, S: DoubleEndedSearcher<'a>> DoubleEndedSearcher<'a> for NotSearcher<S> {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_searcher() {
        let haystack = "aababbaa a";
        let mut searcher = NotPattern::new("a").into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Reject(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));
        assert_eq!(searcher.next(), SearchStep::Reject(3, 4));
        assert_eq!(searcher.next(), SearchStep::Match(4, 5));
        assert_eq!(searcher.next(), SearchStep::Match(5, 6));
        assert_eq!(searcher.next(), SearchStep::Reject(6, 7));
        assert_eq!(searcher.next(), SearchStep::Reject(7, 8));
        assert_eq!(searcher.next(), SearchStep::Match(8, 9));
        assert_eq!(searcher.next(), SearchStep::Reject(9, 10));
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
