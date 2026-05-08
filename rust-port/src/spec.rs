use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    #[serde(default)]
    pub openapi: String,
    #[serde(default)]
    pub info: Info,
    #[serde(default)]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub paths: BTreeMap<String, PathItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Info {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Server {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(default)]
    pub get: Option<Operation>,
    #[serde(default)]
    pub post: Option<Operation>,
    #[serde(default)]
    pub put: Option<Operation>,
    #[serde(default)]
    pub patch: Option<Operation>,
    #[serde(default)]
    pub delete: Option<Operation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Operation {
    #[serde(default, rename = "operationId")]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default, rename = "requestBody")]
    pub request_body: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "in")]
    pub location: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    pub command_name: String,
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub summary: String,
    pub tag: String,
    pub has_body: bool,
    pub path_params: Vec<String>,
    pub query_params: Vec<String>,
}

pub async fn load_spec_from_target(target: &str) -> Result<OpenApiSpec> {
    if target.starts_with("http://") || target.starts_with("https://") {
        let response = reqwest::get(target)
            .await
            .with_context(|| format!("fetching spec {target}"))?;
        let text = response.text().await.context("reading remote spec body")?;
        parse_spec(&text)
    } else {
        let text = tokio::fs::read_to_string(target)
            .await
            .with_context(|| format!("reading spec {}", target))?;
        parse_spec(&text)
    }
}

pub async fn load_spec_from_path(path: &Path) -> Result<OpenApiSpec> {
    let text = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("reading spec {}", path.display()))?;
    parse_spec(&text)
}

pub fn parse_spec(contents: &str) -> Result<OpenApiSpec> {
    let spec: OpenApiSpec = if contents.trim_start().starts_with('{') {
        serde_json::from_str(contents).context("parsing JSON spec")?
    } else {
        serde_yaml::from_str(contents).context("parsing YAML spec")?
    };
    if spec.paths.is_empty() {
        bail!("spec did not contain any paths");
    }
    Ok(spec)
}

impl OpenApiSpec {
    pub fn title_or_default(&self) -> String {
        if self.info.title.trim().is_empty() {
            "Printed API".to_string()
        } else {
            self.info.title.clone()
        }
    }

    pub fn base_url(&self) -> String {
        self.servers
            .iter()
            .find(|server| !server.url.trim().is_empty())
            .map(|server| server.url.clone())
            .unwrap_or_else(|| "https://example.invalid".to_string())
    }

    pub fn operations(&self) -> Vec<OperationRecord> {
        let mut operations = Vec::new();
        for (path, item) in &self.paths {
            push_operation(&mut operations, path, "GET", item.get.as_ref());
            push_operation(&mut operations, path, "POST", item.post.as_ref());
            push_operation(&mut operations, path, "PUT", item.put.as_ref());
            push_operation(&mut operations, path, "PATCH", item.patch.as_ref());
            push_operation(&mut operations, path, "DELETE", item.delete.as_ref());
        }
        operations
    }
}

fn push_operation(
    operations: &mut Vec<OperationRecord>,
    path: &str,
    method: &str,
    operation: Option<&Operation>,
) {
    let Some(operation) = operation else {
        return;
    };
    let operation_id = operation
        .operation_id
        .clone()
        .unwrap_or_else(|| fallback_operation_id(method, path));
    let command_name = sanitize_identifier(&operation_id);
    let summary = operation
        .summary
        .clone()
        .or_else(|| operation.description.clone())
        .unwrap_or_else(|| format!("{method} {path}"));
    let tag = operation
        .tags
        .first()
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    for param in &operation.parameters {
        match param.location.as_str() {
            "path" => path_params.push(param.name.clone()),
            "query" => query_params.push(param.name.clone()),
            _ => {}
        }
    }
    operations.push(OperationRecord {
        command_name,
        operation_id,
        method: method.to_string(),
        path: path.to_string(),
        summary,
        tag,
        has_body: operation.request_body.is_some(),
        path_params,
        query_params,
    });
}

fn fallback_operation_id(method: &str, path: &str) -> String {
    sanitize_identifier(&format!("{method}-{}", path.replace('/', "-")))
}

pub fn sanitize_identifier(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}
