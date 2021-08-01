#[cfg(test)]
use pattern_adapters_macro::regex_pattern;

#[test]
fn test_lowercase_decimal_class() {
    let haystack = "0123456789a";

    let pattern = regex_pattern!("\\d");
    let mut matches = haystack.matches(pattern);

    assert_eq!(matches.next(), Some("0"));
    assert_eq!(matches.next(), Some("1"));
    assert_eq!(matches.next(), Some("2"));
    assert_eq!(matches.next(), Some("3"));
    assert_eq!(matches.next(), Some("4"));
    assert_eq!(matches.next(), Some("5"));
    assert_eq!(matches.next(), Some("6"));
    assert_eq!(matches.next(), Some("7"));
    assert_eq!(matches.next(), Some("8"));
    assert_eq!(matches.next(), Some("9"));
    assert_eq!(matches.next(), None);
}

fn main() {}
