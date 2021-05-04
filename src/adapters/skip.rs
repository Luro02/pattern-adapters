use core::str::pattern::{Pattern, SearchStep, Searcher};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkipPattern<P>(P, usize);

impl<P> SkipPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P, n: usize) -> Self {
        Self(pattern, n)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for SkipPattern<P> {
    type Searcher = SkipSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        SkipSearcher::new(self.0.into_searcher(haystack), self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipSearcher<S> {
    searcher: S,
    n: usize,
}

impl<S> SkipSearcher<S> {
    #[must_use]
    pub(super) const fn new(searcher: S, n: usize) -> Self {
        Self { searcher, n }
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for SkipSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        let step = self.searcher.next();

        if let SearchStep::Match(start, end) = step {
            if self.n > 0 {
                self.n -= 1;

                SearchStep::Reject(start, end)
            } else {
                SearchStep::Match(start, end)
            }
        } else {
            step
        }
    }
}
