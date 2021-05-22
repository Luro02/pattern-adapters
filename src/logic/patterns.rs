use super::{LOrPattern, NotPattern};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NorPattern<A, B>(NotPattern<LOrPattern<A, B>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndPattern<A, B>(NorPattern<NotPattern<A>, NotPattern<B>>);

macro_rules! generate_pattern {
    ($($name:ident { constructor => $f:expr, inner_type => $($inner_type:tt)+ }),+) => {
        $(
            impl<A, B> $name<A, B> {
                #[must_use]
                pub(super) fn new(a: A, b: B) -> Self { Self($f(a, b)) }
            }

            impl<'a, A, B> ::core::str::pattern::Pattern<'a> for $name<A, B>
            where
                A: ::core::str::pattern::Pattern<'a>,
                B: ::core::str::pattern::Pattern<'a>,
            {
                type Searcher = <$($inner_type)+ as ::core::str::pattern::Pattern<'a>>::Searcher;

                fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
                    self.0.into_searcher(haystack)
                }
            }
        )+
    };
}

generate_pattern!(
    AndPattern {
        constructor => |a, b| NorPattern::new(NotPattern::new(a), NotPattern::new(b)),
        inner_type => NorPattern<NotPattern<A>, NotPattern<B>>
    },
    NorPattern {
        constructor => |a, b| NotPattern::new(LOrPattern::new(a, b)),
        inner_type => NotPattern<LOrPattern<A, B>>
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use core::str::pattern::{Pattern, SearchStep, Searcher};

    #[test]
    fn test_fuzzer_failure_01() {
        let haystack = "\nP\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}@\u{0}\u{0}\u{0}\u{0}";
        let needle = "\u{0}\u{0}\u{0}";

        // TODO: and pattern behaves like a then
        let mut searcher = AndPattern::new(needle, needle).into_searcher(haystack);
        assert_eq!(searcher.next(), SearchStep::Reject(0, 1));
        assert_eq!(searcher.next(), SearchStep::Reject(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 5));
        assert_eq!(searcher.next(), SearchStep::Match(5, 8));
        assert_eq!(searcher.next(), SearchStep::Reject(8, 9));
        assert_eq!(searcher.next(), SearchStep::Reject(9, 10));
    }
}
