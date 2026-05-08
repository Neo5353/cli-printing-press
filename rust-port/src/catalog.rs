use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::fs;
use url::Url;

use crate::pipeline::slugify;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub spec_url: Option<String>,
    #[serde(default)]
    pub spec_format: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ResolvedTargetKind {
    Catalog,
    LocalFile,
    RemoteSpec,
    Url,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedTarget {
    pub input: String,
    pub kind: ResolvedTargetKind,
    pub name: String,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub spec_source: Option<String>,
    pub homepage: Option<String>,
}

impl ResolvedTarget {
    pub fn spec_ref(&self) -> Option<&str> {
        self.spec_source.as_deref()
    }
}

pub async fn resolve_target(repo_root: &Path, target: &str) -> Result<ResolvedTarget> {
    let input_path = PathBuf::from(target);
    if input_path.exists() {
        let absolute = input_path
            .canonicalize()
            .with_context(|| format!("canonicalizing local target {target}"))?;
        let stem = absolute
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("local-spec");
        let slug = slugify(stem);
        return Ok(ResolvedTarget {
            input: target.to_string(),
            kind: ResolvedTargetKind::LocalFile,
            name: stem.to_string(),
            slug: slug.clone(),
            display_name: humanize_slug(&slug),
            description: Some("Local spec file".to_string()),
            spec_source: Some(absolute.display().to_string()),
            homepage: None,
        });
    }

    if let Ok(url) = Url::parse(target) {
        let slug = slug_from_url(&url);
        let looks_like_spec = target.ends_with(".yaml")
            || target.ends_with(".yml")
            || target.ends_with(".json")
            || target.contains("openapi");
        return Ok(ResolvedTarget {
            input: target.to_string(),
            kind: if looks_like_spec {
                ResolvedTargetKind::RemoteSpec
            } else {
                ResolvedTargetKind::Url
            },
            name: slug.clone(),
            slug: slug.clone(),
            display_name: humanize_slug(&slug),
            description: Some(if looks_like_spec {
                "Remote spec URL".to_string()
            } else {
                "Website target".to_string()
            }),
            spec_source: looks_like_spec.then(|| target.to_string()),
            homepage: Some(target.to_string()),
        });
    }

    let catalog_path = repo_root.join("catalog").join(format!("{target}.yaml"));
    if !catalog_path.exists() {
        bail!("target {target:?} is not a local file, URL, or known catalog entry");
    }

    let contents = fs::read_to_string(&catalog_path)
        .await
        .with_context(|| format!("reading catalog entry {}", catalog_path.display()))?;
    let entry: CatalogEntry = serde_yaml::from_str(&contents)
        .with_context(|| format!("parsing catalog entry {}", catalog_path.display()))?;
    let slug = slugify(&entry.name);
    Ok(ResolvedTarget {
        input: target.to_string(),
        kind: ResolvedTargetKind::Catalog,
        name: entry.name.clone(),
        slug: slug.clone(),
        display_name: entry
            .display_name
            .clone()
            .unwrap_or_else(|| humanize_slug(&slug)),
        description: entry.description.clone(),
        spec_source: entry.spec_url.clone(),
        homepage: entry.homepage.clone(),
    })
}

fn slug_from_url(url: &Url) -> String {
    let mut candidate = url
        .path_segments()
        .and_then(|segments| segments.rev().find(|segment| !segment.is_empty()))
        .unwrap_or(url.host_str().unwrap_or("target"))
        .replace(".yaml", "")
        .replace(".yml", "")
        .replace(".json", "");
    if candidate.is_empty() {
        candidate = url.host_str().unwrap_or("target").to_string();
    }
    slugify(&candidate)
}

fn humanize_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
