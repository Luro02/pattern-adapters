use core::ops::Deref;
use core::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, SearchStep, Searcher};

/// A pattern with `peek()` that returns the next [`SearchStep`] without advancing the [`Searcher`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeekablePattern<P>(P);

impl<P> PeekablePattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P) -> Self {
        Self(pattern)
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for PeekablePattern<P> {
    type Searcher = PeekableSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        PeekableSearcher::new(self.0.into_searcher(haystack))
    }
}

/// A searcher with `peek()` that returns the next [`SearchStep`] without advancing the [`Searcher`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeekableSearcher<S> {
    searcher: S,
    peeked: Option<SearchStep>,
    peeked_back: Option<SearchStep>,
}

impl<S> PeekableSearcher<S> {
    #[must_use]
    pub(super) const fn new(searcher: S) -> Self {
        Self {
            searcher,
            peeked: None,
            peeked_back: None,
        }
    }
}

impl<'a, S: Searcher<'a>> PeekableSearcher<S> {
    /// Returns the next [`SearchStep`] without advancing the [`Searcher`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(pattern)]
    /// use core::str::pattern::{Searcher, SearchStep, Pattern};
    /// use pattern_adapters::adapters::PatternExt;
    ///
    /// let haystack = "hi hi ho";
    /// let mut searcher = "hi".peekable().into_searcher(haystack);
    ///
    /// // with peek() one can see which SearchStep will be returned in the future:
    /// assert_eq!(searcher.peek(), SearchStep::Match(0, 2));
    /// assert_eq!(searcher.next(), SearchStep::Match(0, 2));
    ///
    /// // you can also peek multiple times (searcher will not advance through peek)
    /// assert_eq!(searcher.peek(), SearchStep::Reject(2, 3));
    /// assert_eq!(searcher.peek(), SearchStep::Reject(2, 3));
    /// assert_eq!(searcher.next(), SearchStep::Reject(2, 3));
    ///
    /// assert_eq!(searcher.next(), SearchStep::Match(3, 5));
    ///
    /// assert_eq!(searcher.next(), SearchStep::Reject(5, 6));
    ///
    /// assert_eq!(searcher.next(), SearchStep::Reject(6, 8));
    ///
    /// assert_eq!(searcher.peek(), SearchStep::Done);
    /// assert_eq!(searcher.next(), SearchStep::Done);
    /// ```
    #[must_use]
    pub fn peek(&mut self) -> SearchStep {
        let searcher = &mut self.searcher;

        *self.peeked.get_or_insert_with(|| searcher.next())
    }
}

impl<'a, S: ReverseSearcher<'a>> PeekableSearcher<S> {
    /// Returns the next [`SearchStep`] from the back without advancing the [`Searcher`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(pattern)]
    /// use core::str::pattern::{ReverseSearcher, Searcher, SearchStep, Pattern};
    /// use pattern_adapters::adapters::PatternExt;
    ///
    /// let haystack = "hi hi ho";
    /// let mut searcher = "hi".peekable().into_searcher(haystack);
    ///
    /// // one can still peek normally:
    /// assert_eq!(searcher.peek(), SearchStep::Match(0, 2));
    /// assert_eq!(searcher.peek_back(), SearchStep::Reject(6, 8));
    /// assert_eq!(searcher.next(), SearchStep::Match(0, 2));
    /// assert_eq!(searcher.next_back(), SearchStep::Reject(6, 8));
    ///
    /// // you can also peek multiple times (searcher will not advance through peek)
    /// assert_eq!(searcher.peek_back(), SearchStep::Reject(5, 6));
    /// assert_eq!(searcher.peek_back(), SearchStep::Reject(5, 6));
    /// assert_eq!(searcher.next_back(), SearchStep::Reject(5, 6));
    ///
    /// assert_eq!(searcher.next_back(), SearchStep::Match(3, 5));
    ///
    /// assert_eq!(searcher.next_back(), SearchStep::Reject(2, 3));
    ///
    /// assert_eq!(searcher.next_back(), SearchStep::Match(0, 2));
    ///
    /// assert_eq!(searcher.peek_back(), SearchStep::Done);
    /// assert_eq!(searcher.next_back(), SearchStep::Done);
    /// ```
    #[must_use]
    pub fn peek_back(&mut self) -> SearchStep {
        let searcher = &mut self.searcher;

        *self.peeked_back.get_or_insert_with(|| searcher.next_back())
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

unsafe impl<'a, S: ReverseSearcher<'a>> ReverseSearcher<'a> for PeekableSearcher<S> {
    fn next_back(&mut self) -> SearchStep {
        match self.peeked_back.take() {
            Some(value) => value,
            None => self.searcher.next_back(),
        }
    }
}

impl<'a, S: DoubleEndedSearcher<'a>> DoubleEndedSearcher<'a> for PeekableSearcher<S> {}

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
