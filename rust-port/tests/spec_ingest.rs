use printing_press_rs::spec::parse_spec;

#[test]
fn parses_fixture_spec_and_discovers_operations() {
    let spec_text = include_str!("../../testdata/openapi/petstore.yaml");
    let spec = parse_spec(spec_text).expect("spec should parse");
    let operations = spec.operations();

    assert!(!operations.is_empty());
    assert!(
        operations
            .iter()
            .any(|op| op.method == "GET" && op.path == "/pet/{petId}")
    );
    assert_eq!(spec.base_url(), "/api/v3");
}
