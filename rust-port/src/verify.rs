use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::generator::{GeneratedManifest, load_manifest};
use crate::spec::OpenApiSpec;

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub manifest_path: PathBuf,
    pub crate_dir: PathBuf,
    pub operation_count: usize,
    pub missing_in_spec: Vec<String>,
    pub cargo_check_ok: bool,
}

pub async fn verify_generated(dir: &Path, spec_path: Option<&Path>) -> Result<VerifyReport> {
    let manifest = load_manifest(dir).await?;
    verify_manifest(dir, &manifest, spec_path).await
}

pub async fn verify_manifest(
    dir: &Path,
    manifest: &GeneratedManifest,
    spec_path: Option<&Path>,
) -> Result<VerifyReport> {
    let crate_dir = dir.join(&manifest.generated_crate);
    if !crate_dir.join("Cargo.toml").exists() {
        bail!(
            "generated crate missing Cargo.toml at {}",
            crate_dir.display()
        );
    }
    if !crate_dir.join("src").join("main.rs").exists() {
        bail!(
            "generated crate missing src/main.rs at {}",
            crate_dir.display()
        );
    }

    let mut missing = Vec::new();
    if let Some(spec_path) = spec_path {
        let spec = crate::spec::load_spec_from_path(spec_path).await?;
        let declared: HashSet<(String, String)> = spec
            .operations()
            .into_iter()
            .map(|op| (op.method, op.path))
            .collect();
        for operation in &manifest.operations {
            if !declared.contains(&(operation.method.clone(), operation.path.clone())) {
                missing.push(format!("{} {}", operation.method, operation.path));
            }
        }
    }

    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&crate_dir)
        .output()
        .with_context(|| format!("running cargo check in {}", crate_dir.display()))?;
    let cargo_check_ok = output.status.success();
    if !cargo_check_ok {
        bail!(
            "cargo check failed for generated crate:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if !missing.is_empty() {
        bail!(
            "manifest operations missing from spec: {}",
            missing.join(", ")
        );
    }

    Ok(VerifyReport {
        manifest_path: dir.join(crate::generator::MANIFEST_FILENAME),
        crate_dir,
        operation_count: manifest.operation_count,
        missing_in_spec: missing,
        cargo_check_ok,
    })
}

pub fn score_manifest(manifest: &GeneratedManifest, spec: &OpenApiSpec) -> serde_json::Value {
    let total_spec_ops = spec.operations().len();
    let implemented = manifest.operation_count;
    let coverage = if total_spec_ops == 0 {
        0.0
    } else {
        implemented as f64 / total_spec_ops as f64
    };
    serde_json::json!({
        "operation_count": implemented,
        "spec_operation_count": total_spec_ops,
        "coverage": coverage,
        "base_url": manifest.base_url,
    })
}
