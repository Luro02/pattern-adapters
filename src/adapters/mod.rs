mod fused;
mod indexed;
mod limited;
mod peekable;
mod simplify;
mod skip;

pub use fused::{FusedPattern, FusedSearcher};
pub use indexed::{IndexedPattern, IndexedSearcher};
pub use limited::{LimitedPattern, LimitedSearcher};
pub use peekable::{PeekablePattern, PeekableSearcher};
pub use simplify::{SimplifyingPattern, SimplifyingSearcher};
pub use skip::{SkipPattern, SkipSearcher};

use core::str::pattern::Pattern;
use core::str::pattern::Searcher;

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
    fn limited(self, max: usize) -> LimitedPattern<Self> {
        LimitedPattern::new(self, max)
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
}

impl<'a, P: Pattern<'a>> PatternExt<'a> for P {}

pub trait SearcherExt<'a>: Searcher<'a>
where
    Self: Sized,
{
    #[must_use]
    fn fused(self) -> FusedSearcher<Self> {
        FusedSearcher::new(self)
    }

    #[must_use]
    fn indexed(self) -> IndexedSearcher<Self> {
        IndexedSearcher::new(self)
    }

    #[must_use]
    fn limited(self, max: usize) -> LimitedSearcher<Self> {
        LimitedSearcher::new(self, max)
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
