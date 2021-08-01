use core::str::pattern::{Pattern, SearchStep, Searcher};

/// Matches only if the first [`Pattern`] matches and then the second [`Pattern`] matches.
///
/// # Example
///
/// Matches only if `"ab"` is followed by a number:
///
/// ```
/// #![feature(pattern)]
/// use core::str::pattern::{SearchStep, Searcher, Pattern};
/// use pattern_adapters::adapters::PatternExt;
///
/// // the string one wants to search through:
/// let haystack = "ab1abcab9d";
/// let mut searcher = "ab".then(|c: char| c.is_ascii_digit()).into_searcher(haystack);
///
/// assert_eq!(searcher.next_match(), Some((0, 3))); // matches "ab1"
/// assert_eq!(searcher.next_match(), Some((6, 9))); // matches "ab9"
/// assert_eq!(searcher.next_match(), None);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThenPattern<P, T>(P, T);

impl<P, T> ThenPattern<P, T> {
    #[must_use]
    pub(super) const fn new(first: P, then: T) -> Self {
        Self(first, then)
    }
}

impl<'a, P: Pattern<'a>, T: Pattern<'a>> Pattern<'a> for ThenPattern<P, T> {
    type Searcher = ThenSearcher<P::Searcher, T::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        ThenSearcher::new(
            self.0.into_searcher(haystack),
            self.1.into_searcher(haystack),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThenSearcher<S, T> {
    first: S,
    then: T,
    index: usize,
    next_then: Option<(usize, usize)>,
    next_match: Option<(usize, usize)>,
}

impl<S, T> ThenSearcher<S, T> {
    #[must_use]
    pub(super) const fn new(first: S, then: T) -> Self {
        // TODO: enforce that they have the same haystack!
        Self {
            first,
            then,
            next_then: None,
            index: 0,
            next_match: None,
        }
    }
}

impl<'a, S: Searcher<'a>, T: Searcher<'a>> ThenSearcher<S, T> {
    /// Returns the index of the searcher in the haystack.
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the currently valid match for self.then.
    /// The returned value will have its end after the variable.
    #[must_use]
    fn next_then_match(&mut self, after: usize) -> Option<(usize, usize)> {
        // get the cached match or if it does not exist, get a new match
        if let Some((start, end)) = self.next_then.or_else(|| self.then.next_match()) {
            // check if the match is before the index
            if end < after || start < after {
                // if so get the next match that is in bounds
                while let Some((start, end)) = self.then.next_match() {
                    if end > after {
                        self.next_then = Some((start, end));
                        return self.next_then;
                    }
                }
            } else {
                self.next_then = Some((start, end));
                return Some((start, end));
            }
        }

        None
    }

    #[must_use]
    fn next_internal_match(&mut self) -> Option<(usize, usize)> {
        while let Some((start, end)) = self.first.next_match() {
            if start >= self.index() {
                return Some((start, end));
            }
        }

        None
    }

    #[must_use]
    fn any_step(&mut self, step: SearchStep) -> SearchStep {
        if let SearchStep::Match(_, end) | SearchStep::Reject(_, end) = step {
            self.index = end;
        }

        step
    }

    #[must_use]
    fn reject_remaining(&mut self) -> SearchStep {
        self.any_step(SearchStep::Reject(self.index(), self.haystack().len()))
    }
}

unsafe impl<'a, S: Searcher<'a>, T: Searcher<'a>> Searcher<'a> for ThenSearcher<S, T> {
    fn haystack(&self) -> &'a str {
        debug_assert_eq!(self.first.haystack(), self.then.haystack());
        self.first.haystack()
    }

    // idea is to be able to do something like this:
    // 'a'.then('c')
    // which would match "ac"
    fn next(&mut self) -> SearchStep {
        // check if there is something that could not be matched in the last call (because one had to reject first)
        if let Some((start, end)) = self.next_match.take() {
            return self.any_step(SearchStep::Match(start, end));
        }

        if self.index() >= self.haystack().len() {
            return SearchStep::Done;
        }

        if let Some((start, end)) = self.next_internal_match() {
            if let Some((tstart, tend)) = self.next_then_match(end) {
                if end == tstart {
                    if self.index() < start {
                        self.next_match = Some((start, tend));
                        return self.any_step(SearchStep::Reject(self.index(), start));
                    }

                    debug_assert_eq!(self.index(), start);

                    self.any_step(SearchStep::Match(start, tend))
                } else {
                    self.any_step(SearchStep::Reject(self.index(), end))
                }
            } else {
                self.reject_remaining()
            }
        } else if self.index() < self.haystack().len() {
            self.reject_remaining()
        } else {
            unreachable!("SearchStep::Done")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple() {
        let haystack = "abbaab";
        //              012345
        let mut searcher = ThenPattern::new('a', 'b').into_searcher(haystack);

        assert_eq!(searcher.index(), 0);
        assert_eq!(searcher.next(), SearchStep::Match(0, 2));
        assert_eq!(searcher.index(), 2);
        assert_eq!(searcher.next(), SearchStep::Reject(2, 4));
        assert_eq!(searcher.index(), 4);
        assert_eq!(searcher.next(), SearchStep::Match(4, 6));
        assert_eq!(searcher.index(), 6);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.index(), searcher.haystack().len());
    }

    #[test]
    fn test_lines() {
        let haystack = "hello\n\r is \r\n this \r\n\rworking?";
        let mut searcher = ThenPattern::new('\n', '\r').into_searcher(haystack);

        assert_eq!(searcher.index(), 0);
        assert_eq!(searcher.next(), SearchStep::Reject(0, 5));
        assert_eq!(searcher.index(), 5);
        assert_eq!(searcher.next(), SearchStep::Match(5, 7));
        assert_eq!(searcher.index(), 7);
        assert_eq!(searcher.next(), SearchStep::Reject(7, 13));
        assert_eq!(searcher.index(), 13);
        assert_eq!(searcher.next(), SearchStep::Reject(13, 20));
        assert_eq!(searcher.index(), 20);
        assert_eq!(searcher.next(), SearchStep::Match(20, 22));
        assert_eq!(searcher.index(), 22);
        assert_eq!(searcher.next(), SearchStep::Reject(22, 30));
        assert_eq!(searcher.index(), 30);
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.index(), searcher.haystack().len());
    }

    #[test]
    fn test_no_match() {
        let haystack = "hello world!";
        let mut searcher = ThenPattern::new('l', 'ö').into_searcher(haystack);

        assert_eq!(searcher.index(), 0);
        assert_eq!(
            searcher.next(),
            SearchStep::Reject(0, searcher.haystack().len())
        );
        assert_eq!(searcher.index(), searcher.haystack().len());
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.index(), searcher.haystack().len());
    }

    #[test]
    fn test_any() {
        let haystack = "h(ello worl)d!";
        let mut searcher =
            ThenPattern::new(ThenPattern::new('(', |_| true), ')').into_searcher(haystack);

        // first test the underlying then pattern:
        {
            let mut searcher = ThenPattern::new('(', |_| true).into_searcher(haystack);

            assert_eq!(searcher.index(), 0);
            assert_eq!(searcher.next(), SearchStep::Reject(0, 1));

            assert_eq!(searcher.index(), 1);
            assert_eq!(searcher.next(), SearchStep::Match(1, 3));

            assert_eq!(searcher.index(), 3);
            assert_eq!(
                searcher.next(),
                SearchStep::Reject(3, searcher.haystack().len())
            );

            assert_eq!(searcher.index(), searcher.haystack().len());
            assert_eq!(searcher.next(), SearchStep::Done);
        }

        assert_eq!(searcher.index(), 0);
        assert_eq!(searcher.next(), SearchStep::Reject(0, 3));

        assert_eq!(searcher.index(), 3);
        assert_eq!(
            searcher.next(),
            SearchStep::Reject(3, searcher.haystack().len())
        );

        assert_eq!(searcher.index(), searcher.haystack().len());
        assert_eq!(searcher.next(), SearchStep::Done);
        assert_eq!(searcher.index(), searcher.haystack().len());
    }

    #[test]
    fn test_fuzzer_failure_01() {
        let haystack = "\u{1}\u{0}\u{0}\u{0}/\u{0}/";
        let needle_1 = '\u{0}';
        let needle_2 = "\u{0}\u{0}";

        let mut then_searcher = ThenPattern::new(needle_1, needle_1).into_searcher(haystack);
        let mut str_searcher = needle_2.into_searcher(haystack);

        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
    }

    #[test]
    fn test_fuzzer_failure_02() {
        let haystack = "[///\n\u{13}*\u{0}\u{0}\u{0}";
        let needle_1 = '\u{0}';
        let needle_2 = "\u{0}\u{0}";

        let mut then_searcher = ThenPattern::new(needle_1, needle_1).into_searcher(haystack);
        let mut str_searcher = needle_2.into_searcher(haystack);

        // This test triggered a debug_assertion
        // TODO: poll until both are done?
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
    }

    #[test]
    fn test_fuzzer_failure_03() {
        let haystack = "\u{e}///\n\u{0}\u{0}\u{0}\u{0}//\u{13}\u{0}\u{0}\u{0}\u{0}\u{0}";
        let needle_1 = '\u{0}';
        let needle_2 = "\u{0}\u{0}";

        let mut then_searcher = ThenPattern::new(needle_1, needle_1).into_searcher(haystack);
        let mut str_searcher = needle_2.into_searcher(haystack);

        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
        assert_eq!(then_searcher.next_match(), str_searcher.next_match());
    }
}
