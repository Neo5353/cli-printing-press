use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::json;
use tokio::fs;
use tracing::info;

use crate::catalog::resolve_target;
use crate::cli::{Cli, Command, GenerateArgs, PublishArgs, TargetArgs, VerifyArgs};
use crate::generator::{generate_project, load_manifest};
use crate::pipeline::WorkspacePaths;
use crate::spec::load_spec_from_target;
use crate::verify::{score_manifest, verify_generated};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Research(args) => research(args).await,
        Command::Generate(args) => generate(args).await,
        Command::Verify(args) => verify(args).await,
        Command::Scorecard(args) => scorecard(args).await,
        Command::Dogfood(args) => dogfood(args).await,
        Command::Publish(args) => publish(args).await,
        Command::Sniff(args) => sniff(args).await,
    }
}

async fn research(args: TargetArgs) -> Result<()> {
    let workspace = WorkspacePaths::discover()?;
    let resolved = resolve_target(&workspace.repo_root, &args.target).await?;
    let mut payload = serde_json::to_value(&resolved).context("serializing research result")?;
    if let Some(spec_ref) = resolved.spec_ref() {
        let spec = load_spec_from_target(spec_ref).await?;
        payload["operation_count"] = json!(spec.operations().len());
        payload["base_url"] = json!(spec.base_url());
        payload["title"] = json!(spec.title_or_default());
    }
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

async fn generate(args: GenerateArgs) -> Result<()> {
    let workspace = WorkspacePaths::discover()?;
    let resolved = resolve_target(&workspace.repo_root, &args.target).await?;
    let spec_ref = resolved.spec_ref().context(
        "generate requires a resolvable spec source; plain websites are not implemented yet",
    )?;
    let spec = load_spec_from_target(spec_ref).await?;
    let result = generate_project(&workspace, &resolved, &spec, args.output, args.force).await?;
    info!(target = %resolved.input, output = %result.output_dir.display(), "generated rust printed cli");
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "name": resolved.slug,
            "display_name": resolved.display_name,
            "output_dir": result.output_dir,
            "manifest_path": result.manifest_path,
            "crate_dir": result.crate_dir,
            "operation_count": result.operation_count,
        }))?
    );
    Ok(())
}

async fn verify(args: VerifyArgs) -> Result<()> {
    let report = verify_generated(&args.target, args.spec.as_deref()).await?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn scorecard(args: VerifyArgs) -> Result<()> {
    let manifest = load_manifest(&args.target).await?;
    let spec_path = args
        .spec
        .as_deref()
        .context("scorecard requires --spec <path>")?;
    let spec = crate::spec::load_spec_from_path(spec_path).await?;
    let score = score_manifest(&manifest, &spec);
    println!("{}", serde_json::to_string_pretty(&score)?);
    Ok(())
}

async fn dogfood(args: VerifyArgs) -> Result<()> {
    let report = verify_generated(&args.target, args.spec.as_deref()).await?;
    let verdict = json!({
        "status": "pass",
        "operation_count": report.operation_count,
        "crate_dir": report.crate_dir,
    });
    println!("{}", serde_json::to_string_pretty(&verdict)?);
    Ok(())
}

async fn publish(args: PublishArgs) -> Result<()> {
    let workspace = WorkspacePaths::discover()?;
    let manifest = load_manifest(&args.target).await?;
    let destination = args.destination.unwrap_or_else(|| {
        workspace
            .repo_root
            .join("rust-port-library")
            .join(&manifest.name)
    });
    if destination.exists() {
        fs::remove_dir_all(&destination).await.with_context(|| {
            format!(
                "removing existing publish destination {}",
                destination.display()
            )
        })?;
    }
    copy_dir_all(&args.target, &destination).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "published": true,
            "destination": destination,
            "name": manifest.name,
        }))?
    );
    Ok(())
}

async fn sniff(args: TargetArgs) -> Result<()> {
    let workspace = WorkspacePaths::discover()?;
    let resolved = resolve_target(&workspace.repo_root, &args.target).await?;
    if resolved.spec_source.is_some() {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({"status": "already-has-spec", "target": resolved.input})
            )?
        );
        return Ok(());
    }
    bail!(
        "browser sniffing is not implemented in rust-port v0.1; use a local or remote spec target"
    )
}

async fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    let mut stack: Vec<(PathBuf, PathBuf)> = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((current_src, current_dst)) = stack.pop() {
        fs::create_dir_all(&current_dst)
            .await
            .with_context(|| format!("creating directory {}", current_dst.display()))?;
        let mut entries = fs::read_dir(&current_src)
            .await
            .with_context(|| format!("reading directory {}", current_src.display()))?;
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            let from = entry.path();
            let to = current_dst.join(entry.file_name());
            if file_type.is_dir() {
                stack.push((from, to));
            } else {
                fs::copy(&from, &to)
                    .await
                    .with_context(|| format!("copying {} to {}", from.display(), to.display()))?;
            }
        }
    }
    Ok(())
}
