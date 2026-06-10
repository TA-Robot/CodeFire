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
use serde_json::{json, Value};
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

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Myers => "myers",
            Self::Patience => "patience",
            Self::Histogram => "histogram",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffOptions {
    pub(crate) algorithm: DiffAlgorithm,
    pub(crate) context_lines: usize,
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
            context_lines: 3,
            rename_detection: false,
            atom_diff: false,
            trace_diff: false,
            impact_diff: false,
            json_output: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedCommitish {
    objects: PathBuf,
    commit_id: String,
    label: String,
}

#[derive(Debug)]
pub(crate) struct ReviewPackOptions {
    pub(crate) source: String,
    pub(crate) base: Option<String>,
    pub(crate) algorithm: DiffAlgorithm,
    pub(crate) rename_detection: bool,
}

#[derive(Debug)]
pub(crate) struct PatchExportOptions {
    pub(crate) source: String,
    pub(crate) base: Option<String>,
}

pub(crate) fn show_commitish(repo_root: Option<&Path>, value: &str) -> Result<String, CliError> {
    let data = show_commitish_data(repo_root, value)?;
    let show = data
        .get("show")
        .and_then(Value::as_object)
        .ok_or_else(|| CliError::InvalidRepository("invalid show result".to_string()))?;
    let label = show.get("label").and_then(Value::as_str).unwrap_or("");
    let commit_id = show.get("commit").and_then(Value::as_str).unwrap_or("");
    let message = show.get("message").and_then(Value::as_str).unwrap_or("");
    let parents = show
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
    let files = show.get("files").and_then(Value::as_u64).unwrap_or(0);
    let atoms = show.get("atoms").and_then(Value::as_u64).unwrap_or(0);
    let certificate = show
        .get("certificate")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let signature = show
        .get("signature")
        .and_then(|value| value.get("summary"))
        .and_then(Value::as_str)
        .unwrap_or("(none)");

    Ok(format!(
        "Object: {label}\nCommit: {commit_id}\nMessage: {message}\nParents: {parents}\nFiles: {files}\nAtoms: {atoms}\nCertificate: {certificate}\nSignature: {signature}\n",
    ))
}

pub(crate) fn show_commitish_data(
    repo_root: Option<&Path>,
    value: &str,
) -> Result<Value, CliError> {
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
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    let certificate = commit
        .get("certificate")
        .and_then(|value| value.get("result"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let signature = commit.get("signature").and_then(Value::as_object);
    let signature_summary = signature
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

    Ok(json!({
        "type": "codefire_show_result",
        "version": 1,
        "show": {
            "target": value,
            "label": resolved.label,
            "commit": resolved.commit_id,
            "message": message,
            "parents": parents,
            "files": files,
            "atoms": atoms,
            "certificate": certificate,
            "roots": commit.get("roots").cloned().unwrap_or_else(|| json!({})),
            "signature": {
                "present": signature.is_some(),
                "summary": signature_summary,
                "details": signature.cloned().map(Value::Object).unwrap_or(Value::Null),
            },
        },
    }))
}

pub(crate) fn diff_commitish_with_options(
    repo_root: Option<&Path>,
    left: &str,
    right: &str,
    options: &DiffOptions,
) -> Result<String, CliError> {
    let left = resolve_commitish_any(repo_root, left)?;
    let right = resolve_commitish_any(repo_root, right)?;
    diff_resolved_commitish_with_options(&left, &right, options)
}

fn diff_resolved_commitish_with_options(
    left: &ResolvedCommitish,
    right: &ResolvedCommitish,
    options: &DiffOptions,
) -> Result<String, CliError> {
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
                left,
                right,
                left_index.as_ref().expect("left atom index loaded"),
                left_graph.as_ref().expect("left trace graph loaded"),
                right_index.as_ref().expect("right atom index loaded"),
                right_graph.as_ref().expect("right trace graph loaded"),
            )
        })
        .transpose()?;
    if options.json_output {
        return render_diff_json(DiffJsonContext {
            left,
            right,
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

pub(crate) fn review_pack_with_options(
    repo_root: Option<&Path>,
    options: &ReviewPackOptions,
) -> Result<String, CliError> {
    let source = resolve_commitish_any(repo_root, &options.source)?;
    let base = if let Some(base) = &options.base {
        resolve_commitish_any(repo_root, base)?
    } else {
        first_parent_commitish(&source)?
    };
    let file_options = DiffOptions {
        algorithm: options.algorithm,
        rename_detection: options.rename_detection,
        ..DiffOptions::default()
    };
    let semantic_options = DiffOptions {
        algorithm: options.algorithm,
        rename_detection: options.rename_detection,
        atom_diff: true,
        trace_diff: true,
        impact_diff: true,
        json_output: true,
        ..DiffOptions::default()
    };
    let file_diff_text = diff_resolved_commitish_with_options(&base, &source, &file_options)?;
    let semantic_diff_text =
        diff_resolved_commitish_with_options(&base, &source, &semantic_options)?;
    let semantic_diff = serde_json::from_str::<Value>(&semantic_diff_text)?;
    let next_actions = semantic_diff
        .get("next_actions")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let base_summary = commit_review_summary(&base)?;
    let source_summary = commit_review_summary(&source)?;
    let value = json!({
        "type": "codefire_review_pack",
        "version": 1,
        "base": base_summary,
        "source": source_summary,
        "options": {
            "algorithm": options.algorithm.as_str(),
            "rename_detection": options.rename_detection,
        },
        "file_diff": {
            "format": "unified",
            "text": file_diff_text,
        },
        "semantic_diff": semantic_diff,
        "verification": {
            "base": verification_summary(&base)?,
            "source": verification_summary(&source)?,
        },
        "next_actions": next_actions,
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

pub(crate) fn patch_export_with_options(
    repo_root: Option<&Path>,
    options: &PatchExportOptions,
) -> Result<String, CliError> {
    let source = resolve_commitish_any(repo_root, &options.source)?;
    let base = if let Some(base) = &options.base {
        resolve_commitish_any(repo_root, base)?
    } else {
        first_parent_commitish(&source)?
    };
    let base_files = manifest_contents(&base.objects, &base.commit_id)?;
    let source_files = manifest_contents(&source.objects, &source.commit_id)?;
    let paths = base_files
        .keys()
        .chain(source_files.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut entries = Vec::new();
    for path in paths {
        let base_data = base_files.get(&path);
        let source_data = source_files.get(&path);
        if base_data == source_data {
            continue;
        }
        if let Some(data) = source_data {
            entries.push(json!({
                "path": path,
                "action": "write",
                "encoding": "base64",
                "content": encode_base64(data),
                "bytes": data.len(),
            }));
        } else {
            entries.push(json!({
                "path": path,
                "action": "delete",
            }));
        }
    }
    let value = json!({
        "type": "codefire_patch",
        "version": 1,
        "base": commit_review_summary(&base)?,
        "source": commit_review_summary(&source)?,
        "entries": entries,
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

fn first_parent_commitish(source: &ResolvedCommitish) -> Result<ResolvedCommitish, CliError> {
    let commit = codefire_store::read_object(&source.objects, &source.commit_id)?;
    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::InvalidRepository("commit parents must be a list".to_string()))?;
    let parent = parents.first().and_then(Value::as_str).ok_or_else(|| {
        CliError::Usage("review-pack requires --base for a root commit".to_string())
    })?;
    codefire_store::validate_sealed_commit(&source.objects, parent)?;
    Ok(ResolvedCommitish {
        objects: source.objects.clone(),
        commit_id: parent.to_string(),
        label: format!("parent@{parent}"),
    })
}

fn commit_review_summary(resolved: &ResolvedCommitish) -> Result<Value, CliError> {
    let commit = codefire_store::read_object(&resolved.objects, &resolved.commit_id)?;
    let manifest = root_object(&resolved.objects, &commit, "content_manifest")?;
    let atom_index = root_object(&resolved.objects, &commit, "atom_index")?;
    Ok(json!({
        "label": &resolved.label,
        "commit": &resolved.commit_id,
        "message": commit.get("message").and_then(Value::as_str).unwrap_or(""),
        "parents": commit.get("parents").cloned().unwrap_or_else(|| json!([])),
        "certificate": commit.get("certificate").cloned().unwrap_or_else(|| json!(null)),
        "files": manifest.get("entries").and_then(Value::as_array).map_or(0, Vec::len),
        "atoms": atom_index.get("atoms").and_then(Value::as_array).map_or(0, Vec::len),
        "duplicate_atom_ids": atom_index.get("duplicate_atom_ids").cloned().unwrap_or_else(|| json!([])),
        "verification": verification_summary(resolved)?,
    }))
}

fn verification_summary(resolved: &ResolvedCommitish) -> Result<Value, CliError> {
    let commit = codefire_store::read_object(&resolved.objects, &resolved.commit_id)?;
    let verification = root_object(&resolved.objects, &commit, "verification")?;
    Ok(json!({
        "result": verification.get("result").and_then(Value::as_str).unwrap_or("unknown"),
        "open_required_fires": verification.get("open_required_fires").and_then(Value::as_u64).unwrap_or(0),
        "failed_checks": verification.get("failed_checks").and_then(Value::as_array).map_or(0, Vec::len),
        "missing_required_links": verification.get("missing_required_links").and_then(Value::as_array).map_or(0, Vec::len),
        "stale_resolutions": verification.get("stale_resolutions").and_then(Value::as_array).map_or(0, Vec::len),
        "duplicate_atom_ids": verification.get("duplicate_atom_ids").and_then(Value::as_array).map_or(0, Vec::len),
        "verified_at": verification.get("verified_at").cloned().unwrap_or_else(|| json!(null)),
    }))
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
