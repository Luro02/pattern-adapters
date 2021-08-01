use core::str::pattern::{Pattern, SearchStep, Searcher};

use super::PeekableSearcher;

// TODO: steps should be kept as is and ideally would not be merged

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinPattern<P> {
    pattern: P,
    min: usize,
}

impl<P> MinPattern<P> {
    #[must_use]
    pub(super) const fn new(pattern: P, min: usize) -> Self {
        Self { pattern, min }
    }
}

impl<'a, P: Pattern<'a>> Pattern<'a> for MinPattern<P> {
    type Searcher = MinSearcher<P::Searcher>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        MinSearcher::new(self.pattern.into_searcher(haystack), self.min)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinSearcher<S> {
    searcher: PeekableSearcher<S>,
    min: usize,
}

impl<S> MinSearcher<S> {
    #[must_use]
    pub(super) fn new(searcher: S, min: usize) -> Self {
        Self {
            searcher: PeekableSearcher::new(searcher),
            min,
        }
    }
}

unsafe impl<'a, S: Searcher<'a>> Searcher<'a> for MinSearcher<S> {
    fn haystack(&self) -> &'a str {
        self.searcher.haystack()
    }

    fn next(&mut self) -> SearchStep {
        let step = self.searcher.next();

        if let SearchStep::Match(start, end) = step {
            let mut end = end;
            let mut matches = 1;

            for _ in 1..self.min {
                if let SearchStep::Match(next_start, next_end) = self.searcher.peek() {
                    // check that the next match starts at the end of the previous match:
                    if next_start == end {
                        // advance the searcher:
                        self.searcher.next();
                        matches += 1;
                        end = next_end;
                    } else {
                        // discontinuity between the matches

                        // check that enough has been matched to return something:
                        if matches == self.min {
                            return SearchStep::Match(start, end);
                        } else {
                            return SearchStep::Reject(start, next_start);
                        }
                    }
                } else {
                    break;
                }
            }

            if matches < self.min {
                return SearchStep::Reject(start, end);
            }

            SearchStep::Match(start, end)
        } else {
            step
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
