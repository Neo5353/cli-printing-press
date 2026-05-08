use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePaths {
    pub repo_root: PathBuf,
    pub library_root: PathBuf,
}

impl WorkspacePaths {
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir().context("reading current working directory")?;
        let repo_root = cwd
            .parent()
            .map(Path::to_path_buf)
            .context("rust-port must run from inside the repo")?;
        let library_root = repo_root.join("rust-port-out");
        Ok(Self {
            repo_root,
            library_root,
        })
    }
}

pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "printed-api".to_string()
    } else {
        trimmed
    }
}

pub fn default_output_dir(paths: &WorkspacePaths, slug: &str) -> PathBuf {
    paths.library_root.join(slug)
}

pub async fn ensure_clean_dir(path: &Path, force: bool) -> Result<()> {
    if path.exists() {
        if force {
            fs::remove_dir_all(path)
                .await
                .with_context(|| format!("removing existing directory {}", path.display()))?;
        } else {
            bail!(
                "output directory {} already exists; pass --force to replace it",
                path.display()
            );
        }
    }
    fs::create_dir_all(path)
        .await
        .with_context(|| format!("creating directory {}", path.display()))?;
    Ok(())
}

pub async fn write_string(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("creating parent directory {}", parent.display()))?;
    }
    let mut file = fs::File::create(path)
        .await
        .with_context(|| format!("creating file {}", path.display()))?;
    file.write_all(contents.as_bytes())
        .await
        .with_context(|| format!("writing file {}", path.display()))?;
    Ok(())
}
