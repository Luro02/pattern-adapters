use core::{
    str::pattern::{Pattern, SearchStep, Searcher},
    usize,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepeatPattern<P> {
    pattern: P,
    min: usize,
    max: usize,
}

impl<P> RepeatPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P, min: usize, max: usize) -> Self {
        Self { pattern, min, max }
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for RepeatPattern<P> {
    type Searcher = RepeatSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        RepeatSearcher::new(self.pattern.into_searcher(haystack), self.min, self.max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatSearcher<S> {
    searcher: S,
    min: usize,
    max: usize,
    cached: Option<(usize, usize)>,
    next_match: Option<(usize, usize)>,
    index: usize,
}

impl<S> RepeatSearcher<S> {
    #[must_use]
    pub(super) const fn new(searcher: S, min: usize, max: usize) -> Self {
        Self {
            searcher,
            min,
            max,
            cached: None,
            next_match: None,
            index: 0,
        }
    }
}

impl<'a, S: Searcher<'a>> RepeatSearcher<S> {
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    fn next_internal_match(&mut self) -> Option<(usize, usize)> {
        self.cached.take().or_else(|| self.searcher.next_match())
    }

    fn cache_match(&mut self, start: usize, end: usize) {
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
    fn match_step(&mut self, start: usize, end: usize) -> SearchStep {
        if self.index() < start {
            self.next_match = Some((start, end));
            return self.reject_to(start);
        }

        debug_assert_eq!(self.index(), start);

        self.any_step(SearchStep::Match(start, end))
    }

    #[must_use]
    fn reject_to(&mut self, end: usize) -> SearchStep {
        self.any_step(SearchStep::Reject(self.index(), end))
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for RepeatSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        if let Some((start, end)) = self.next_match.take() {
            return self.any_step(SearchStep::Match(start, end));
        }

        if self.index() >= self.haystack().len() {
            return SearchStep::Done;
        }

        if let Some((start, end)) = self.next_internal_match() {
            let mut end = end;
            let mut matches = 1;

            for _ in 1..self.max {
                if let Some((next_start, next_end)) = self.next_internal_match() {
                    // check that the next match starts at the end of the previous match:
                    if next_start == end {
                        matches += 1;
                        end = next_end;
                    } else {
                        // discontinuity between the matches
                        self.cache_match(next_start, next_end);

                        if matches <= self.max && matches >= self.min {
                            return self.match_step(start, end);
                        } else {
                            return self.reject_to(next_start);
                        }
                    }
                } else {
                    break;
                }
            }

            if matches < self.min {
                return self.reject_to(end);
            }

            return self.match_step(start, end);
        }

        SearchStep::Done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // TODO: this should be used in all tests!
    fn assert_continuity<'a, P: Pattern<'a>>(haystack: &'a str, pattern: P) {
        let mut searcher = pattern.into_searcher(haystack);

        let mut last_end = 0;
        loop {
            if let SearchStep::Match(start, end) | SearchStep::Reject(start, end) = searcher.next()
            {
                assert_eq!(last_end, start);
                last_end = end;
            } else {
                break;
            }
        }

        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_continuity() {
        // TODO: add more tests/strings
        assert_continuity(
            "1 2 3 4aäalpqkdpawdjap 1320pjf.-as ,m",
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 1),
        );
    }

    #[test]
    fn test_simple() {
        let haystack = "0123456789";
        let mut searcher =
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 5).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 5));
        assert_eq!(searcher.next(), SearchStep::Match(5, 10));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_exactly_one() {
        let haystack = "012";
        let mut searcher =
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 1).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_unicode() {
        let haystack = "0 äö 13 hello 2";
        let mut searcher =
            RepeatPattern::new(|c: char| c.is_ascii_digit(), 1, 3).into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 7));
        assert_eq!(searcher.next(), SearchStep::Match(7, 9));
        assert_eq!(searcher.next(), SearchStep::Reject(9, 16));
        assert_eq!(searcher.next(), SearchStep::Match(16, 17));
        assert_eq!(searcher.next(), SearchStep::Done);
    }

    #[test]
    fn test_fuzzer_failure_01() {
        let haystack = concat!(
            "\u{1c},\u{0}\u{0}\u{0}\u{0}\u{0}=\u{0}\u{0}",
            "\u{0}\u{0}\u{1e}\u{0}\u{0},\u{0}\u{0}\u{0}\u{0}",
            "\u{0}\u{0}#\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}#,,\u{0},\u{0},",
            "\u{0},,\u{0}\u{0}\u{0}=\u{0}\u{0}\u{10}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0},,",
            ",\u{0}\u{0}\u{1a}\u{1c};\u{0}\u{1a}\u{1}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{4}",
            "\u{0}\u{1c}\u{0}\u{1a},\u{0}\u{4}\u{0}\u{1c}\u{0}\u{1a}\u{1e}\u{0}/\u{0}#;\u{0}\u{0}J\u{0}#,",
            ",\u{0},\u{0}\u{1a},\u{0}\u{4}\u{0}\u{1c}\u{0}\u{1a}\u{1e}\u{0}/\u{0}\u{1c}\u{0}\u{1a}\u{1c};",
            "\u{0}\u{0}J\u{0}#,,\u{0},\u{0},\u{1c}\u{0}\u{1a},\u{0}\u{4}\u{0}\u{1c}\u{0}\u{1a}\u{1e}\u{0}/",
            "\u{0}\u{1c}\u{0}\u{1a}\u{1c};\u{0}\u{7},\u{0}$\u{0}\u{0}\u{0}\u{0}\u{0}=)\u{0}\u{0}\u{0}\u{0}\u{0}"
        );

        let pattern = RepeatPattern::new('\u{0}', 169618582, 3170534138692239568);
        assert_continuity(haystack, pattern);
    }
}
