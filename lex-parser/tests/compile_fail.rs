#[test]
fn container_type_safety() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/container_type_safety.rs");
}
