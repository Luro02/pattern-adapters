use core::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, SearchStep, Searcher};
use core::str::CharIndices;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharPattern<F, T>(F, T);

impl<F, T> CharPattern<F, T>
where
    F: FnMut(char, &mut T) -> bool,
{
    #[must_use]
    pub fn new(f: F, state: T) -> Self {
        Self(f, state)
    }
}

impl<'a, F, T> Pattern<'a> for CharPattern<F, T>
where
    F: FnMut(char, &mut T) -> bool,
{
    type Searcher = CharSearcher<'a, F, T>;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        CharSearcher::new(haystack, self.0, self.1)
    }
}

#[derive(Debug, Clone)]
pub struct CharSearcher<'a, F, T> {
    f: F,
    state: T,
    chars: CharIndices<'a>,
}

impl<'a, F, T> CharSearcher<'a, F, T>
where
    F: FnMut(char, &mut T) -> bool,
{
    #[must_use]
    pub(super) fn new(haystack: &'a str, f: F, state: T) -> Self {
        Self {
            f,
            state,
            chars: haystack.char_indices(),
        }
    }
}

unsafe impl<'a, F, T> Searcher<'a> for CharSearcher<'a, F, T>
where
    F: FnMut(char, &mut T) -> bool,
{
    fn haystack(&self) -> &'a str {
        self.chars.as_str()
    }

    fn next(&mut self) -> SearchStep {
        if let Some((start, c)) = self.chars.next() {
            let end = start + c.len_utf8();

            if (self.f)(c, &mut self.state) {
                SearchStep::Match(start, end)
            } else {
                SearchStep::Reject(start, end)
            }
        } else {
            SearchStep::Done
        }
    }
}

unsafe impl<'a, F, T> ReverseSearcher<'a> for CharSearcher<'a, F, T>
where
    F: FnMut(char, &mut T) -> bool,
{
    fn next_back(&mut self) -> SearchStep {
        if let Some((start, c)) = self.chars.next_back() {
            let end = start + c.len_utf8();

            if (self.f)(c, &mut self.state) {
                SearchStep::Match(start, end)
            } else {
                SearchStep::Reject(start, end)
            }
        } else {
            SearchStep::Done
        }
    }
}

impl<'a, F, T> DoubleEndedSearcher<'a> for CharSearcher<'a, F, T> where
    F: FnMut(char, &mut T) -> bool
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parentheses() {
        let haystack = "hey((hw)#";
        //                   0123456789
        let mut searcher = CharPattern::new(
            // function that rejects everything between open and close parentheses
            |c, inside_parentheses| {
                if c == '(' && !*inside_parentheses {
                    *inside_parentheses = true;
                } else if c == ')' && *inside_parentheses {
                    *inside_parentheses = false;
                    return false;
                }

                !*inside_parentheses
            },
            false,
        )
        .into_searcher(haystack);

        assert_eq!(searcher.next(), SearchStep::Match(0, 1));
        assert_eq!(searcher.next(), SearchStep::Match(1, 2));
        assert_eq!(searcher.next(), SearchStep::Match(2, 3));

        assert_eq!(searcher.next(), SearchStep::Reject(3, 4));
        assert_eq!(searcher.next(), SearchStep::Reject(4, 5));
        assert_eq!(searcher.next(), SearchStep::Reject(5, 6));
        assert_eq!(searcher.next(), SearchStep::Reject(6, 7));
        assert_eq!(searcher.next(), SearchStep::Reject(7, 8));

        assert_eq!(searcher.next(), SearchStep::Match(8, 9));
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}
