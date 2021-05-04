use core::str::pattern::{Pattern, SearchStep, Searcher};

/// An indexed pattern, that will keep track of the last matched index.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexedPattern<P>(P);

impl<P> IndexedPattern<P> {
    /// Constructs a new indexed pattern with the provided pattern.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use pattern_adapters::adapters::IndexedPattern;
    /// let pattern = IndexedPattern::new(|c: char| c.is_alphabetic());
    /// ```
    #[must_use]
    pub const fn new(pattern: P) -> Self {
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
    const fn new(searcher: S) -> Self {
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
