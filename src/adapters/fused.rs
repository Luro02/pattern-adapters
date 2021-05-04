use core::ops::Deref;
use core::str::pattern::{Pattern, SearchStep, Searcher};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FusedPattern<P>(P);

impl<P> FusedPattern<P> {
    #[must_use]
    pub const fn new(pattern: P) -> Self {
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
}

impl<S> FusedSearcher<S> {
    #[must_use]
    const fn new(searcher: S) -> Self {
        Self {
            searcher,
            exhausted: false,
        }
    }
}

impl<'a, S: Searcher<'a>> FusedSearcher<S> {
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
