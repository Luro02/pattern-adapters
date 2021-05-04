use core::ops::Deref;
use core::str::pattern::{Pattern, SearchStep, Searcher};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeekablePattern<P>(P);

impl<P> PeekablePattern<P> {
    #[must_use]
    pub const fn new(pattern: P) -> Self {
        Self(pattern)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for PeekablePattern<P> {
    type Searcher = PeekableSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        PeekableSearcher::new(self.0.into_searcher(haystack))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeekableSearcher<S> {
    searcher: S,
    peeked: Option<SearchStep>,
}

impl<S> PeekableSearcher<S> {
    #[must_use]
    const fn new(searcher: S) -> Self {
        Self {
            searcher,
            peeked: None,
        }
    }
}

impl<'a, S: Searcher<'a>> PeekableSearcher<S> {
    #[must_use]
    pub fn peek(&mut self) -> SearchStep {
        let searcher = &mut self.searcher;

        *self.peeked.get_or_insert_with(|| searcher.next())
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for PeekableSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        match self.peeked.take() {
            Some(value) => value,
            None => self.searcher.next(),
        }
    }
}

impl<'a, S: Searcher<'a>> Deref for PeekableSearcher<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.searcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple() {
        let haystack = "heLLo";
        let mut searcher = PeekablePattern::new(char::is_lowercase).into_searcher(haystack);

        assert_eq!(searcher.peek(), SearchStep::Match(0, 1));
        assert_eq!(searcher.peek(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(0, 1));

        assert_eq!(searcher.peek(), SearchStep::Match(1, 2));
        assert_eq!(searcher.peek(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));

        assert_eq!(searcher.peek(), SearchStep::Reject(2, 3));
        assert_eq!(searcher.peek(), SearchStep::Reject(2, 3));
        assert_eq!(searcher.next(), SearchStep::Reject(2, 3));

        assert_eq!(searcher.next(), SearchStep::Reject(3, 4));

        assert_eq!(searcher.peek(), SearchStep::Match(4, 5));
        assert_eq!(searcher.next(), SearchStep::Match(4, 5));

        assert_eq!(searcher.peek(), SearchStep::Done);
        assert_eq!(searcher.peek(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
