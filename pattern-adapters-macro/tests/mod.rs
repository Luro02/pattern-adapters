use trybuild::TestCases;

#[test]
fn tests() {
    let test = TestCases::new();

    test.pass("tests/empty_string.rs");
    test.pass("tests/class.rs");
}
