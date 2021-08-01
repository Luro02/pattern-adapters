use core::str::pattern::{Pattern, SearchStep, Searcher};

#[macro_export]
macro_rules! assert_searcher_eq {
    ( $first:ident $(, $next:ident)+ ) => {
        loop {
            let first_step = $first.next();

            $(
                assert_eq!(first_step, $next.next());
            )+

            if first_step == ::core::str::pattern::SearchStep::Done {
                break;
            }
        }
    };
}

#[macro_export]
macro_rules! assert_matches_eq {
    ( $first:ident $(, $next:ident)+ ) => {
        let very_first_step = $first.next_match();
        $(
            assert_eq!(very_first_step, $next.next_match());
        )+
        while let Some(first_step) = $first.next_match() {
            $(
                assert_eq!(Some(first_step), $next.next_match());
            )+
        }
        let very_last_step = $first.next_match();
        $(
            assert_eq!(very_last_step, $next.next_match());
        )+
    };
}

pub fn assert_integrity<'a, P: Pattern<'a>>(haystack: &'a str, pattern: P) {
    let mut searcher = pattern.into_searcher(haystack);

    let mut last_end = 0;
    while let SearchStep::Match(start, end) | SearchStep::Reject(start, end) = searcher.next() {
        assert!(start <= end);
        // ensure that there are no spaces between the steps
        assert_eq!(last_end, start);
        last_end = end;

        // the indices must lie on valid char boundaries:
        assert!(haystack.is_char_boundary(start));
        assert!(haystack.is_char_boundary(end));
    }

    for _ in 0..3 {
        assert_eq!(searcher.next(), SearchStep::Done);
    }
}

#[must_use]
pub fn count_searcher<'a>(mut searcher: impl Searcher<'a>) -> (usize, usize) {
    let mut number_of_matches = 0;
    let mut number_of_rejects = 0;

    loop {
        match searcher.next() {
            SearchStep::Match(_, _) => number_of_matches += 1,
            SearchStep::Reject(_, _) => number_of_rejects += 1,
            SearchStep::Done => break,
        }
    }

    (number_of_matches, number_of_rejects)
}
