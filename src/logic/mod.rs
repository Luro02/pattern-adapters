mod not;
mod or;
mod patterns;

pub use not::{NotPattern, NotSearcher};
pub use or::{LOrPattern, OrSearcher, ROrPattern};
pub use patterns::*;

use core::str::pattern::{Pattern, Searcher};

pub trait LogicPatternExt<'a>: Pattern<'a> {
    #[must_use]
    fn lor<P: Pattern<'a>>(self, other: P) -> LOrPattern<Self, P> {
        LOrPattern::new(self, other)
    }

    #[must_use]
    fn ror<P: Pattern<'a>>(self, other: P) -> ROrPattern<Self, P> {
        ROrPattern::new(self, other)
    }

    #[must_use]
    fn not(self) -> NotPattern<Self> {
        NotPattern::new(self)
    }

    #[must_use]
    fn and<P: Pattern<'a>>(self, other: P) -> AndPattern<Self, P> {
        AndPattern::new(self, other)
    }

    #[must_use]
    fn nor<P: Pattern<'a>>(self, other: P) -> NorPattern<Self, P> {
        NorPattern::new(self, other)
    }
}

impl<'a, P: Pattern<'a>> LogicPatternExt<'a> for P {}

pub trait LogicSearcherExt<'a>: Searcher<'a>
where
    Self: Sized,
{
    #[must_use]
    fn not(self) -> NotSearcher<Self> {
        NotSearcher(self)
    }
}

impl<'a, S: Searcher<'a>> LogicSearcherExt<'a> for S {}
