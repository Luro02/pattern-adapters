#[cfg(test)]
use pattern_adapters_macro::regex_pattern;

#[test]
fn test_empty_string() {
    let pattern = regex_pattern!("");
    assert_eq!(pattern, "");
}

fn main() {}
