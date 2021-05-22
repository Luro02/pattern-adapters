use core::str::pattern::{SearchStep, Searcher};

pub struct InternalSearcher<S> {
    searcher: S,
    index: usize,
    cached: Option<(usize, usize)>,
    next_match: Option<(usize, usize)>,
}

impl<S> InternalSearcher<S> {
    #[must_use]
    pub fn new(searcher: S) -> Self {
        Self {
            searcher,
            index: 0,
            cached: None,
            next_match: None,
        }
    }
}

impl<'a, S: Searcher<'a>> InternalSearcher<S> {
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    fn next_internal_match(&mut self) -> Option<(usize, usize)> {
        self.cached.take().or_else(|| self.searcher.next_match())
    }

    pub fn cache_match(&mut self, start: usize, end: usize) {
        self.cached = Some((start, end));
    }

    #[must_use]
    fn any_step(&mut self, step: SearchStep) -> SearchStep {
        if let SearchStep::Match(_, end) | SearchStep::Reject(_, end) = step {
            self.index = end;
        }

        step
    }

    #[must_use]
    pub fn match_step(&mut self, start: usize, end: usize) -> SearchStep {
        if self.index() < start {
            self.next_match = Some((start, end));
            return self.reject_to(start);
        }

        debug_assert_eq!(self.index(), start);

        self.any_step(SearchStep::Match(start, end))
    }

    #[must_use]
    pub fn reject_to(&mut self, end: usize) -> SearchStep {
        self.any_step(SearchStep::Reject(self.index(), end))
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for InternalSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        if let Some((start, end)) = self.next_internal_match() {
            return SearchStep::Match(start, end);
        }

        unimplemented!()
    }
}
