mod fused;
mod greedy_reject;
mod indexed;
mod internal;
mod limit;
mod peekable;
mod repeat;
mod skip;
mod stateful;
mod then;

pub use fused::{FusedPattern, FusedSearcher};
pub use greedy_reject::{SimplifyingPattern, SimplifyingSearcher};
pub use indexed::{IndexedPattern, IndexedSearcher};
pub use limit::{LimitPattern, LimitSearcher};
pub use peekable::{PeekablePattern, PeekableSearcher};
pub use repeat::{RepeatPattern, RepeatSearcher};
pub use skip::{SkipPattern, SkipSearcher};
pub use stateful::{CharPattern, CharSearcher};
pub use then::{ThenPattern, ThenSearcher};

use core::str::pattern::Pattern;
use core::str::pattern::Searcher;

// TODO: adapt patterns from https://github.com/VerbalExpressions/RustVerbalExpressions

pub trait PatternExt<'a>: Pattern<'a> {
    #[must_use]
    fn fused(self) -> FusedPattern<Self> {
        FusedPattern::new(self)
    }

    #[must_use]
    fn indexed(self) -> IndexedPattern<Self> {
        IndexedPattern::new(self)
    }

    #[must_use]
    fn limit(self, max: usize) -> LimitPattern<Self> {
        LimitPattern::new(self, max)
    }

    #[must_use]
    fn peekable(self) -> PeekablePattern<Self> {
        PeekablePattern::new(self)
    }

    #[must_use]
    fn simplify(self) -> SimplifyingPattern<Self> {
        SimplifyingPattern::new(self)
    }

    #[must_use]
    fn skip(self, n: usize) -> SkipPattern<Self> {
        SkipPattern::new(self, n)
    }

    #[must_use]
    fn then<P: Pattern<'a>>(self, then: P) -> ThenPattern<Self, P> {
        ThenPattern::new(self, then)
    }

    #[must_use]
    fn repeat(self, min: usize, max: usize) -> RepeatPattern<Self> {
        RepeatPattern::new(self, min, max)
    }
}

impl<'a, P: Pattern<'a>> PatternExt<'a> for P {}

// TODO: rename functions to be consistent (everything in present) (e.g. `fuse`, `limit`, `skip`)
pub trait SearcherExt<'a>: Searcher<'a>
where
    Self: Sized,
{
    /// A searcher that will always return `SearchStep::Done` after `SearchStep::Done`
    /// has been returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(pattern)]
    /// use core::str::pattern::{Pattern, Searcher, SearchStep};
    /// use pattern_adapters::adapters::SearcherExt;
    ///
    /// let haystack = "h";
    /// let mut searcher = 'a'.into_searcher(haystack).fused();
    ///
    /// assert_eq!(searcher.next(), SearchStep::Reject(0, 1));
    /// assert_eq!(searcher.next(), SearchStep::Done);
    /// // it is guranteed that `SearchStep::Done` will always be returned,
    /// // after it has been returned once
    /// assert_eq!(searcher.next(), SearchStep::Done);
    /// ```
    #[must_use]
    fn fused(self) -> FusedSearcher<Self> {
        FusedSearcher::new(self)
    }

    #[must_use]
    fn indexed(self) -> IndexedSearcher<Self> {
        IndexedSearcher::new(self)
    }

    /// Limits the `Searcher` to match at most `max` times.
    ///
    /// ```
    /// # #![feature(pattern)]
    /// use core::str::pattern::{Pattern, Searcher, SearchStep};
    /// use pattern_adapters::adapters::SearcherExt;
    ///
    /// let haystack = "ababab";
    /// let mut searcher = "ab".into_searcher(haystack).limit(2);
    ///
    /// assert_eq!(searcher.next(), SearchStep::Match(0, 2));
    /// assert_eq!(searcher.next(), SearchStep::Match(2, 4));
    /// // this would be the third match, but only 2 matches are allowed, so it is rejected:
    /// assert_eq!(searcher.next(), SearchStep::Reject(4, 6));
    /// assert_eq!(searcher.next(), SearchStep::Done);
    /// ```
    #[must_use]
    fn limit(self, max: usize) -> LimitPattern<Self> {
        LimitPattern::new(self, max)
    }

    #[must_use]
    fn peekable(self) -> PeekableSearcher<Self> {
        PeekableSearcher::new(self)
    }

    #[must_use]
    fn simplify(self) -> SimplifyingSearcher<Self> {
        SimplifyingSearcher::new(self)
    }

    #[must_use]
    fn skip(self, n: usize) -> SkipSearcher<Self> {
        SkipSearcher::new(self, n)
    }
}

impl<'a, S: Searcher<'a>> SearcherExt<'a> for S {}
