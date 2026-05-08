use std::path::PathBuf;

use printing_press_rs::catalog::{ResolvedTarget, ResolvedTargetKind, resolve_target};
use printing_press_rs::generator::generate_project;
use printing_press_rs::pipeline::WorkspacePaths;
use printing_press_rs::spec::{load_spec_from_target, parse_spec};

#[tokio::test]
async fn generate_creates_manifest_and_compilable_crate() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let workspace = WorkspacePaths {
        repo_root: repo_root.clone(),
        library_root: temp.keep(),
    };
    let fixture = repo_root.join("testdata/openapi/petstore.yaml");
    let resolved = resolve_target(&repo_root, fixture.to_str().unwrap())
        .await
        .unwrap();
    let spec = load_spec_from_target(resolved.spec_ref().unwrap())
        .await
        .unwrap();

    let out_dir = workspace.library_root.join("petstore-output");
    let result = generate_project(&workspace, &resolved, &spec, Some(out_dir.clone()), true)
        .await
        .unwrap();

    assert!(result.manifest_path.exists());
    assert!(result.crate_dir.join("Cargo.toml").exists());
    assert!(result.crate_dir.join("src/main.rs").exists());
    assert!(result.operation_count > 0);
}

#[tokio::test]
async fn generate_emits_auth_and_parameterized_commands() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let workspace = WorkspacePaths {
        repo_root,
        library_root: temp.keep(),
    };

    let spec_text = r#"
openapi: "3.0.3"
info:
  title: Param Auth API
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
    let spec = parse_spec(spec_text).unwrap();
    let resolved = ResolvedTarget {
        input: "param-auth-api".to_string(),
        kind: ResolvedTargetKind::LocalFile,
        name: "param-auth-api".to_string(),
        slug: "param-auth-api".to_string(),
        display_name: "Param Auth API".to_string(),
        description: Some("Test fixture".to_string()),
        spec_source: Some("inline".to_string()),
        homepage: None,
    };

    let out_dir = workspace.library_root.join("param-auth-output");
    let result = generate_project(&workspace, &resolved, &spec, Some(out_dir), true)
        .await
        .unwrap();

    let main_rs = std::fs::read_to_string(result.crate_dir.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("Auth(AuthCommand)"));
    assert!(main_rs.contains("struct GetStoreArgs"));
    assert!(main_rs.contains("#[arg(long = \"store-id\""));
    assert!(main_rs.contains("#[arg(long = \"include-inactive\""));
    assert!(main_rs.contains("#[arg(long = \"store-code\""));
    assert!(main_rs.contains("#[arg(long = \"is-active\""));
    assert!(main_rs.contains("resolve_auth_header()?"));
    assert!(main_rs.contains("config_file_path()"));
}
