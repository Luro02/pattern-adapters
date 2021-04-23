use core::str::pattern::{Pattern, SearchStep, Searcher};

use crate::utils::Range;
// TODO: better name?

// This pattern will reject as much as possible, instead of returning multiple
// small rejects
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimplifyingPattern<P>(P);

impl<P> SimplifyingPattern<P> {
    #[must_use]
    pub const fn new(pattern: P) -> Self { Self(pattern) }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for SimplifyingPattern<P> {
    type Searcher = SimplifyingSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        SimplifyingSearcher::new(self.0.into_searcher(haystack))
    }
}

// TODO: simplify with IndexedSearcher

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimplifyingSearcher<S> {
    searcher: S,
    index: usize,
    next_match: Option<(usize, usize)>,
}

impl<S> SimplifyingSearcher<S> {
    #[must_use]
    const fn new(searcher: S) -> Self {
        Self {
            searcher,
            index: 0,
            next_match: None,
        }
    }
}

impl<'a, S: Searcher<'a>> SimplifyingSearcher<S> {
    #[must_use]
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
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for SimplifyingSearcher<S> {
    fn haystack(&self) -> &'a str { self.searcher.haystack() }

    fn next(&mut self) -> SearchStep {
        if let Some((start, end)) = self.next_match.take() {
            debug_assert_eq!(start, self.index);
            self.index = end;
            return SearchStep::Match(start, end);
        }

        if let Some((start, end)) = self.searcher.next_match() {
            self.match_step((start, end).into())
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
