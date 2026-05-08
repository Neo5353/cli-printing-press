use std::path::PathBuf;

use printing_press_rs::catalog::resolve_target;
use printing_press_rs::generator::generate_project;
use printing_press_rs::pipeline::WorkspacePaths;
use printing_press_rs::spec::load_spec_from_target;

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
