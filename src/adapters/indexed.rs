use core::str::pattern::{Pattern, SearchStep, Searcher};

/// An indexed pattern, that will keep track of the last matched index.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexedPattern<P>(P);

impl<P> IndexedPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P) -> Self {
        Self(pattern)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for IndexedPattern<P> {
    type Searcher = IndexedSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        IndexedSearcher::new(self.0.into_searcher(haystack))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexedSearcher<S> {
    searcher: S,
    index: usize,
}

impl<S> IndexedSearcher<S> {
    #[must_use]
    pub(super) const fn new(searcher: S) -> Self {
        Self { searcher, index: 0 }
    }

    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for IndexedSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        let step = self.searcher.next();

        if let SearchStep::Match(_, end) | SearchStep::Reject(_, end) = step {
            self.index = end;
        }

        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_indexed_searcher() {
        let haystack = "aabaaa";
        let mut searcher = IndexedPattern::new('a').into_searcher(haystack);

        assert_eq!(searcher.index(), 0);
        assert_eq!(searcher.next(), SearchStep::Match(0, 1));

        assert_eq!(searcher.index(), 1);
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));

        assert_eq!(searcher.index(), 2);
        assert_eq!(searcher.next(), SearchStep::Reject(2, 3));

        assert_eq!(searcher.index(), 3);
        assert_eq!(searcher.next(), SearchStep::Match(3, 4));

        assert_eq!(searcher.index(), 4);
        assert_eq!(searcher.next(), SearchStep::Match(4, 5));

        assert_eq!(searcher.index(), 5);
        assert_eq!(searcher.next(), SearchStep::Match(5, 6));

        assert_eq!(searcher.index(), 6);
        assert_eq!(searcher.next(), SearchStep::Done);

        assert_eq!(searcher.index(), 6);
        assert_eq!(searcher.next(), SearchStep::Done);

        assert_eq!(searcher.index(), 6);
    }
}
