use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    #[serde(default)]
    pub openapi: String,
    #[serde(default)]
    pub info: Info,
    #[serde(default)]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub security: Vec<SecurityRequirement>,
    #[serde(default)]
    pub components: Components,
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
pub struct Components {
    #[serde(default, rename = "securitySchemes")]
    pub security_schemes: BTreeMap<String, SecurityScheme>,
    #[serde(default)]
    pub schemas: BTreeMap<String, Schema>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(default)]
    pub parameters: Vec<Parameter>,
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
    pub request_body: Option<RequestBody>,
    #[serde(default)]
    pub security: Option<Vec<SecurityRequirement>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "in")]
    pub location: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub schema: Schema,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestBody {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaType {
    #[serde(default)]
    pub schema: Schema,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Schema {
    #[serde(default, rename = "type")]
    pub schema_type: String,
    #[serde(default)]
    pub properties: BTreeMap<String, Schema>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default, rename = "$ref")]
    pub ref_path: Option<String>,
}

pub type SecurityRequirement = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(default, rename = "type")]
    pub scheme_type: String,
    #[serde(default, rename = "in")]
    pub location: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub flows: OAuthFlows,
    #[serde(default, rename = "x-auth-vars")]
    pub auth_vars: Vec<AuthEnvVar>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OAuthFlows {
    #[serde(default, rename = "clientCredentials")]
    pub client_credentials: Option<OAuthFlow>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OAuthFlow {
    #[serde(default, rename = "tokenUrl")]
    pub token_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthEnvVar {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthModel {
    pub scheme_name: String,
    pub kind: String,
    pub header_name: String,
    pub env_vars: Vec<AuthEnvVar>,
    pub token_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationField {
    pub name: String,
    pub flag_name: String,
    pub rust_name: String,
    pub required: bool,
    pub field_type: String,
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
    pub path_fields: Vec<OperationField>,
    pub query_fields: Vec<OperationField>,
    pub body_fields: Vec<OperationField>,
    pub requires_auth: bool,
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
            .with_context(|| format!("reading spec {target}"))?;
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

    pub fn auth_model(&self) -> Option<AuthModel> {
        let scheme_name = self
            .security
            .iter()
            .find_map(first_requirement_name)
            .or_else(|| self.components.security_schemes.keys().next().cloned())?;
        let scheme = self.components.security_schemes.get(&scheme_name)?;
        let (kind, token_url) = match scheme.scheme_type.as_str() {
            "oauth2" if scheme.flows.client_credentials.is_some() => (
                "oauth2-client-credentials".to_string(),
                scheme
                    .flows
                    .client_credentials
                    .as_ref()
                    .map(|flow| flow.token_url.clone())
                    .filter(|value| !value.trim().is_empty()),
            ),
            other => (other.to_string(), None),
        };
        Some(AuthModel {
            scheme_name,
            kind,
            header_name: if scheme.name.trim().is_empty() {
                "Authorization".to_string()
            } else {
                scheme.name.clone()
            },
            env_vars: scheme.auth_vars.clone(),
            token_url,
        })
    }

    pub fn operations(&self) -> Vec<OperationRecord> {
        let mut operations = Vec::new();
        for (path, item) in &self.paths {
            push_operation(self, &mut operations, path, &item.parameters, "GET", item.get.as_ref());
            push_operation(self, &mut operations, path, &item.parameters, "POST", item.post.as_ref());
            push_operation(self, &mut operations, path, &item.parameters, "PUT", item.put.as_ref());
            push_operation(self, &mut operations, path, &item.parameters, "PATCH", item.patch.as_ref());
            push_operation(self, &mut operations, path, &item.parameters, "DELETE", item.delete.as_ref());
        }
        operations
    }
}

fn push_operation(
    spec: &OpenApiSpec,
    operations: &mut Vec<OperationRecord>,
    path: &str,
    path_level_parameters: &[Parameter],
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

    let merged_parameters = merge_parameters(path_level_parameters, &operation.parameters);
    let path_fields = merged_parameters
        .iter()
        .filter(|param| param.location == "path")
        .map(parameter_to_field)
        .collect::<Vec<_>>();
    let query_fields = merged_parameters
        .iter()
        .filter(|param| param.location == "query")
        .map(parameter_to_field)
        .collect::<Vec<_>>();
    let body_fields = extract_body_fields(spec, operation);

    operations.push(OperationRecord {
        command_name,
        operation_id,
        method: method.to_string(),
        path: path.to_string(),
        summary,
        tag,
        has_body: operation.request_body.is_some(),
        path_params: path_fields.iter().map(|field| field.name.clone()).collect(),
        query_params: query_fields.iter().map(|field| field.name.clone()).collect(),
        path_fields,
        query_fields,
        body_fields,
        requires_auth: operation_requires_auth(spec, operation),
    });
}

fn merge_parameters(path_level: &[Parameter], operation_level: &[Parameter]) -> Vec<Parameter> {
    let mut merged = path_level.to_vec();
    for parameter in operation_level {
        if let Some(position) = merged.iter().position(|existing| {
            existing.location == parameter.location && existing.name == parameter.name
        }) {
            merged[position] = parameter.clone();
        } else {
            merged.push(parameter.clone());
        }
    }
    merged
}

fn parameter_to_field(parameter: &Parameter) -> OperationField {
    let words = split_words(&parameter.name);
    OperationField {
        name: parameter.name.clone(),
        flag_name: words.join("-"),
        rust_name: format!("{}_{}", parameter.location, words.join("_")),
        required: parameter.required,
        field_type: schema_kind(&parameter.schema),
    }
}

fn extract_body_fields(spec: &OpenApiSpec, operation: &Operation) -> Vec<OperationField> {
    let Some(request_body) = &operation.request_body else {
        return Vec::new();
    };
    let Some(media_type) = request_body
        .content
        .get("application/json")
        .or_else(|| request_body.content.values().next())
    else {
        return Vec::new();
    };
    let Some(schema) = resolve_schema(spec, &media_type.schema) else {
        return Vec::new();
    };
    let required = schema.required.iter().cloned().collect::<BTreeSet<_>>();
    schema
        .properties
        .iter()
        .map(|(name, property)| {
            let words = split_words(name);
            OperationField {
                name: name.clone(),
                flag_name: words.join("-"),
                rust_name: format!("body_{}", words.join("_")),
                required: required.contains(name),
                field_type: schema_kind(property),
            }
        })
        .collect()
}

fn resolve_schema<'a>(spec: &'a OpenApiSpec, schema: &'a Schema) -> Option<&'a Schema> {
    if let Some(reference) = &schema.ref_path {
        let name = reference.strip_prefix("#/components/schemas/")?;
        spec.components.schemas.get(name)
    } else {
        Some(schema)
    }
}

fn schema_kind(schema: &Schema) -> String {
    match schema.schema_type.as_str() {
        "boolean" => "boolean".to_string(),
        "integer" => "integer".to_string(),
        "number" => "number".to_string(),
        _ => "string".to_string(),
    }
}

fn operation_requires_auth(spec: &OpenApiSpec, operation: &Operation) -> bool {
    match operation.security.as_ref() {
        Some(requirements) => security_requirements_require_auth(requirements),
        None => security_requirements_require_auth(&spec.security),
    }
}

fn security_requirements_require_auth(requirements: &[SecurityRequirement]) -> bool {
    if requirements.is_empty() {
        return false;
    }
    if requirements.iter().any(|requirement| requirement.is_empty()) {
        return false;
    }
    requirements.iter().any(|requirement| !requirement.is_empty())
}

fn first_requirement_name(requirement: &SecurityRequirement) -> Option<String> {
    requirement.keys().next().cloned()
}

fn fallback_operation_id(method: &str, path: &str) -> String {
    sanitize_identifier(&format!("{method}-{}", path.replace('/', "-")))
}

pub fn sanitize_identifier(value: &str) -> String {
    split_words(value).join("-")
}

pub fn upper_snake(value: &str) -> String {
    split_words(value).join("_").to_ascii_uppercase()
}

fn split_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    for (idx, ch) in chars.iter().enumerate() {
        if ch.is_ascii_alphanumeric() {
            let boundary = idx > 0
                && ch.is_ascii_uppercase()
                && chars[idx - 1].is_ascii_lowercase()
                && !current.is_empty();
            if boundary {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(*ch);
        } else if !current.is_empty() {
            words.push(current.to_ascii_lowercase());
            current.clear();
        }
    }
    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }
    if words.is_empty() {
        vec!["value".to_string()]
    } else {
        words
    }
}
