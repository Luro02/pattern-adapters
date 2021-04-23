mod indexed;
mod limited;
mod peekable;
mod simplify;
mod skip;

pub use indexed::{IndexedPattern, IndexedSearcher};
pub use limited::{LimitedPattern, LimitedSearcher};
pub use peekable::{PeekablePattern, PeekableSearcher};
pub use simplify::{SimplifyingPattern, SimplifyingSearcher};
pub use skip::{SkipPattern, SkipSearcher};

use core::str::pattern::Pattern;

pub trait PatternExt<'a>: Pattern<'a> {
    #[must_use]
    fn indexed(self) -> IndexedPattern<Self> { IndexedPattern::new(self) }

    #[must_use]
    fn limited(self, max: usize) -> LimitedPattern<Self> { LimitedPattern::new(self, max) }

    #[must_use]
    fn peekable(self) -> PeekablePattern<Self> { PeekablePattern::new(self) }

    #[must_use]
    fn simplify(self) -> SimplifyingPattern<Self> { SimplifyingPattern::new(self) }

    #[must_use]
    fn skip(self, n: usize) -> SkipPattern<Self> { SkipPattern::new(self, n) }
}

impl<'a, P: Pattern<'a>> PatternExt<'a> for P {}