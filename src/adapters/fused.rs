use core::ops::Deref;
use core::str::pattern::{Pattern, SearchStep, Searcher, ReverseSearcher, DoubleEndedSearcher};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FusedPattern<P>(P);

impl<P> FusedPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P) -> Self {
        Self(pattern)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for FusedPattern<P> {
    type Searcher = FusedSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        FusedSearcher::new(self.0.into_searcher(haystack))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FusedSearcher<S> {
    searcher: S,
    exhausted: bool,
    rexhausted: bool,
}

impl<S> FusedSearcher<S> {
    #[must_use]
    pub(super) const fn new(searcher: S) -> Self {
        Self {
            searcher,
            exhausted: false,
            rexhausted: false,
        }
    }
}

impl<'a, S: Searcher<'a>> FusedSearcher<S> {
    /// Exhausts the Searcher by calling `Searcher::next` repeatedly, until `SearchStep::Done` is returned.
    ///
    /// ### Note
    ///
    /// This could possibly cause an endless loop if the underlying searcher is not implemented correctly.
    /// It should not happen, because `Searcher::haystack` is a finite string and the `SearchStep`s returned by
    /// `Searcher::next` must be non-overlapping.
    pub fn exhaust(&mut self) {
        while self.next() != SearchStep::Done {}
    }
}

impl<'a, S> Deref for FusedSearcher<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.searcher
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for FusedSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        if self.exhausted {
            return SearchStep::Done;
        }

        let step = self.searcher.next();

        if step == SearchStep::Done {
            self.exhausted = true;
        }

        step
    }
}

unsafe impl<'a, S: ReverseSearcher<'a>> ReverseSearcher<'a> for FusedSearcher<S> {
    fn next_back(&mut self) -> SearchStep {
        if self.rexhausted {
            return SearchStep::Done;
        }

        let step = self.searcher.next_back();

        if step == SearchStep::Done {
            self.rexhausted = true;
        }

        step
    }
}

impl<'a, S: DoubleEndedSearcher<'a>> DoubleEndedSearcher<'a> for FusedSearcher<S> {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple() {
        let haystack = "a";
        let mut searcher = FusedPattern::new('a').into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Done);
        // after finishing the searcher should not yield anything
        for _ in 0..20 {
            assert_eq!(searcher.next(), SearchStep::Done);
        }
    }
}
