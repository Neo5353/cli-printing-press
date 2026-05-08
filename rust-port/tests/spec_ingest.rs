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

#[test]
fn parses_security_overrides_and_request_body_fields() {
    let spec_text = r#"
openapi: "3.0.3"
info:
  title: Test API
  version: "1.0.0"
servers:
  - url: https://api.example.com
security:
  - ApiKeyAuth: []
components:
  securitySchemes:
    ApiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
paths:
  /stores/{storeId}:
    get:
      operationId: getStore
      summary: Get a store
      security: []
      parameters:
        - name: storeId
          in: path
          required: true
          schema:
            type: string
        - name: includeInactive
          in: query
          required: false
          schema:
            type: boolean
  /stores:
    post:
      operationId: createStore
      summary: Create a store
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [store_code]
              properties:
                store_code:
                  type: string
                is_active:
                  type: boolean
"#;

    let spec = parse_spec(spec_text).expect("spec should parse");
    let operations = spec.operations();

    let get_store = operations
        .iter()
        .find(|op| op.operation_id == "getStore")
        .expect("getStore operation present");
    assert_eq!(get_store.path_params, vec!["storeId"]);
    assert_eq!(get_store.query_params, vec!["includeInactive"]);
    assert!(!get_store.requires_auth);

    let create_store = operations
        .iter()
        .find(|op| op.operation_id == "createStore")
        .expect("createStore operation present");
    assert!(create_store.requires_auth);
    assert!(create_store.has_body);
    assert_eq!(create_store.body_fields.len(), 2);
    let store_code = create_store
        .body_fields
        .iter()
        .find(|field| field.name == "store_code")
        .expect("store_code body field present");
    assert!(store_code.required);
    assert!(create_store
        .body_fields
        .iter()
        .any(|field| field.name == "is_active"));
}

#[test]
fn operation_parameters_override_path_level_parameters() {
    let spec_text = r#"
openapi: "3.0.3"
info:
  title: Override API
  version: "1.0.0"
paths:
  /items/{itemId}:
    parameters:
      - name: itemId
        in: path
        required: true
        schema:
          type: string
      - name: expand
        in: query
        required: false
        schema:
          type: string
    get:
      operationId: getItem
      parameters:
        - name: expand
          in: query
          required: true
          schema:
            type: boolean
"#;

    let spec = parse_spec(spec_text).expect("spec should parse");
    let operation = spec
        .operations()
        .into_iter()
        .find(|op| op.operation_id == "getItem")
        .expect("getItem operation present");
    let expand = operation
        .query_fields
        .iter()
        .find(|field| field.name == "expand")
        .expect("expand query field present");

    assert!(expand.required);
    assert_eq!(expand.field_type, "boolean");
}
