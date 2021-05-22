use core::str::pattern::{Pattern, SearchStep, Searcher};

use crate::utils::Range;

// This pattern will reject as much as possible, instead of returning multiple
// small rejects
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimplifyingPattern<P>(P);

impl<P> SimplifyingPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P) -> Self {
        Self(pattern)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for SimplifyingPattern<P> {
    type Searcher = SimplifyingSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        SimplifyingSearcher::new(self.0.into_searcher(haystack))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimplifyingSearcher<S> {
    searcher: S,
    index: usize,
    next_match: Option<(usize, usize)>,
}

impl<S> SimplifyingSearcher<S> {
    #[must_use]
    pub(super) const fn new(searcher: S) -> Self {
        Self {
            searcher,
            index: 0,
            next_match: None,
        }
    }
}

impl<'a, S: Searcher<'a>> SimplifyingSearcher<S> {
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    fn any_step(&mut self, step: SearchStep) -> SearchStep {
        if let SearchStep::Match(_, end) | SearchStep::Reject(_, end) = step {
            self.index = end;
        }

        step
    }

    #[must_use]
    fn match_step(&mut self, start: usize, end: usize) -> SearchStep {
        if self.index() < start {
            self.next_match = Some((start, end));
            return self.reject_to(start);
        }

        assert_eq!(self.index(), start);

        self.any_step(SearchStep::Match(start, end))
    }

    #[must_use]
    fn reject_to(&mut self, end: usize) -> SearchStep {
        debug_assert!(self.index() <= end);
        let start = self.index();
        self.index = end;
        SearchStep::Reject(start, end)
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for SimplifyingSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        if let Some((start, end)) = self.next_match.take() {
            return self.match_step(start, end);
        }

        if let Some((start, end)) = self.searcher.next_match() {
            self.match_step(start, end)
        } else {
            SearchStep::Done
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_search() {
        let haystack = "aabbbba";
        let mut searcher = SimplifyingPattern::new('a').into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Reject(2, 6));
        assert_eq!(searcher.next(), SearchStep::Match(6, 7));
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
