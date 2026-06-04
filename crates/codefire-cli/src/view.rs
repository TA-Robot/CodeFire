use super::http::{http_json, http_remote_path, parse_cf_http_url};
use super::remote::{
    ensure_object_dirs, load_remote_branch, parse_cf_url, remote_dirs, write_object_records,
};
use super::*;
use file_diff::render_manifest_diff;
use semantic_diff::{
    diff_impact, load_atom_index, load_trace_graph, render_atom_diff, render_diff_json,
    render_impact_diff, render_trace_diff, DiffJsonContext,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod file_diff;
mod semantic_diff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffAlgorithm {
    Myers,
    Patience,
    Histogram,
}

impl DiffAlgorithm {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "myers" => Some(Self::Myers),
            "patience" => Some(Self::Patience),
            "histogram" => Some(Self::Histogram),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffOptions {
    pub(crate) algorithm: DiffAlgorithm,
    pub(crate) rename_detection: bool,
    pub(crate) atom_diff: bool,
    pub(crate) trace_diff: bool,
    pub(crate) impact_diff: bool,
    pub(crate) json_output: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            algorithm: DiffAlgorithm::Myers,
            rename_detection: false,
            atom_diff: false,
            trace_diff: false,
            impact_diff: false,
            json_output: false,
        }
    }
}

#[derive(Debug)]
struct ResolvedCommitish {
    objects: PathBuf,
    commit_id: String,
    label: String,
}

pub(crate) fn show_commitish(repo_root: Option<&Path>, value: &str) -> Result<String, CliError> {
    let resolved = resolve_commitish_any(repo_root, value)?;
    let commit = codefire_store::read_object(&resolved.objects, &resolved.commit_id)?;
    let manifest = root_object(&resolved.objects, &commit, "content_manifest")?;
    let atom_index = root_object(&resolved.objects, &commit, "atom_index")?;
    let files = manifest
        .get("entries")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let atoms = atom_index
        .get("atoms")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let message = commit.get("message").and_then(Value::as_str).unwrap_or("");
    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| "(none)".to_string());
    let certificate = commit
        .get("certificate")
        .and_then(|value| value.get("result"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let signature = commit.get("signature").and_then(Value::as_object);
    let signature = signature
        .map(|item| {
            format!(
                "{} ({})",
                item.get("signer")
                    .and_then(Value::as_str)
                    .unwrap_or("(unknown)"),
                item.get("algorithm")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            )
        })
        .unwrap_or_else(|| "(none)".to_string());

    Ok(format!(
        "Object: {}\nCommit: {}\nMessage: {message}\nParents: {parents}\nFiles: {files}\nAtoms: {atoms}\nCertificate: {certificate}\nSignature: {signature}\n",
        resolved.label, resolved.commit_id
    ))
}

pub(crate) fn diff_commitish_with_options(
    repo_root: Option<&Path>,
    left: &str,
    right: &str,
    options: &DiffOptions,
) -> Result<String, CliError> {
    let left = resolve_commitish_any(repo_root, left)?;
    let right = resolve_commitish_any(repo_root, right)?;
    let left_files = manifest_contents(&left.objects, &left.commit_id)?;
    let right_files = manifest_contents(&right.objects, &right.commit_id)?;
    let left_index = (options.atom_diff || options.impact_diff || options.json_output)
        .then(|| load_atom_index(&left.objects, &left.commit_id))
        .transpose()?;
    let right_index = (options.atom_diff || options.impact_diff || options.json_output)
        .then(|| load_atom_index(&right.objects, &right.commit_id))
        .transpose()?;
    let left_graph = (options.trace_diff || options.impact_diff || options.json_output)
        .then(|| load_trace_graph(&left.objects, &left.commit_id))
        .transpose()?;
    let right_graph = (options.trace_diff || options.impact_diff || options.json_output)
        .then(|| load_trace_graph(&right.objects, &right.commit_id))
        .transpose()?;
    let impact = (options.impact_diff || options.json_output)
        .then(|| {
            diff_impact(
                &left,
                &right,
                left_index.as_ref().expect("left atom index loaded"),
                left_graph.as_ref().expect("left trace graph loaded"),
                right_index.as_ref().expect("right atom index loaded"),
                right_graph.as_ref().expect("right trace graph loaded"),
            )
        })
        .transpose()?;
    if options.json_output {
        return render_diff_json(DiffJsonContext {
            left: &left,
            right: &right,
            left_index: left_index.as_ref(),
            right_index: right_index.as_ref(),
            left_graph: left_graph.as_ref(),
            right_graph: right_graph.as_ref(),
            impact: impact.as_ref(),
            options,
        });
    }
    let mut output = render_manifest_diff(
        &left.label,
        &left_files,
        &right.label,
        &right_files,
        options,
    );
    if options.atom_diff {
        render_atom_diff(
            left_index.as_ref().expect("left atom index loaded"),
            right_index.as_ref().expect("right atom index loaded"),
            &mut output,
        );
    }
    if options.trace_diff {
        render_trace_diff(
            left_graph.as_ref().expect("left trace graph loaded"),
            right_graph.as_ref().expect("right trace graph loaded"),
            &mut output,
        );
    }
    if let Some(impact) = &impact {
        render_impact_diff(impact, &mut output);
    }
    Ok(output)
}

fn resolve_local_commitish(repo_root: &Path, value: &str) -> Result<(String, String), CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    if value.starts_with("CF-COMMIT-") {
        codefire_store::validate_sealed_commit(&objects, value)?;
        return Ok((value.to_string(), value.to_string()));
    }
    let branch = load_branch_record(repo_root, value)?;
    let head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &head)?;
    Ok((head.clone(), format!("{value}@{head}")))
}

fn resolve_commitish_any(
    repo_root: Option<&Path>,
    value: &str,
) -> Result<ResolvedCommitish, CliError> {
    if value.starts_with("cf+http://") {
        let remote = parse_cf_http_url(value)?;
        let bundle = http_json(
            "GET",
            &http_remote_path(
                &remote.endpoint,
                &remote.org,
                &remote.app,
                &["branches", &remote.branch, "bundle"],
            ),
            None,
        )?;
        let branch = bundle.get("branch").ok_or_else(|| {
            CliError::InvalidRepository("HTTP remote returned an invalid branch bundle".to_string())
        })?;
        let records = bundle
            .get("objects")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid branch bundle".to_string(),
                )
            })?;
        let temp_objects = env::temp_dir().join(format!(
            "codefire-http-objects-{}-{}",
            std::process::id(),
            stable_hash_48(value.as_bytes())
        ));
        if temp_objects.exists() {
            fs::remove_dir_all(&temp_objects)?;
        }
        ensure_object_dirs(&temp_objects)?;
        write_object_records(&temp_objects, records.iter().cloned())?;
        let head = required_string(branch, &["head"])?;
        codefire_store::validate_sealed_commit(&temp_objects, &head)?;
        return Ok(ResolvedCommitish {
            objects: temp_objects,
            commit_id: head.clone(),
            label: format!("{value}@{head}"),
        });
    }
    if value.starts_with("cf://") {
        let remote = parse_cf_url(value)?;
        let branch = load_remote_branch(&remote)?;
        let head = required_string(&branch, &["head"])?;
        let objects = remote_dirs(&remote.project_root).objects;
        codefire_store::validate_sealed_commit(&objects, &head)?;
        return Ok(ResolvedCommitish {
            objects,
            commit_id: head.clone(),
            label: format!("{value}@{head}"),
        });
    }
    let repo_root = repo_root.ok_or_else(|| {
        CliError::Usage(
            "local branch or commit requires running inside a CodeFire repository".to_string(),
        )
    })?;
    let (commit_id, label) = resolve_local_commitish(repo_root, value)?;
    Ok(ResolvedCommitish {
        objects: repo_root.join(".codefire").join("objects"),
        commit_id,
        label,
    })
}

fn root_object(objects: &Path, commit: &Value, root_name: &str) -> Result<Value, CliError> {
    let object_id = required_string(commit, &["roots", root_name])?;
    Ok(codefire_store::read_object(objects, &object_id)?)
}

pub(crate) fn manifest_contents(
    objects: &Path,
    commit_id: &str,
) -> Result<BTreeMap<String, Vec<u8>>, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let manifest = root_object(objects, &commit, "content_manifest")?;
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
    let mut contents = BTreeMap::new();
    for entry in entries {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let rel_path = required_string(entry, &["path"])?;
        let blob_id = required_string(entry, &["blob"])?;
        let blob = codefire_store::read_object(objects, &blob_id)?;
        let encoding = required_string(&blob, &["encoding"])?;
        if encoding != "base64" {
            return Err(CliError::InvalidRepository(format!(
                "unsupported blob encoding: {encoding}"
            )));
        }
        let content = required_string(&blob, &["content"])?;
        contents.insert(rel_path, decode_base64(&content)?);
    }
    Ok(contents)
}
