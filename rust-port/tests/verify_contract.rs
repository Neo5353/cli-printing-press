use std::path::PathBuf;

use printing_press_rs::catalog::resolve_target;
use printing_press_rs::generator::generate_project;
use printing_press_rs::pipeline::WorkspacePaths;
use printing_press_rs::spec::load_spec_from_target;
use printing_press_rs::verify::verify_generated;

#[tokio::test]
async fn verify_checks_generated_manifest_against_spec() {
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
    let spec_ref = resolved.spec_ref().unwrap().to_string();
    let spec = load_spec_from_target(&spec_ref).await.unwrap();

    let out_dir = workspace.library_root.join("petstore-output");
    generate_project(&workspace, &resolved, &spec, Some(out_dir.clone()), true)
        .await
        .unwrap();

    let spec_path = PathBuf::from(spec_ref);
    let report = verify_generated(&out_dir, Some(spec_path.as_path()))
        .await
        .unwrap();
    assert!(report.cargo_check_ok);
    assert_eq!(report.missing_in_spec.len(), 0);
}
