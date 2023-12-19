use trycmd::TestCases;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn cli_tests() {
  TestCases::new()
    .case("./tests/expected_result.md")
    .insert_var("[VERSION]", VERSION)
    .unwrap();
}
