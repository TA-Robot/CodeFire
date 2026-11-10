//! Creating, opening, and closing branches.

use crate::cli_error::{CliError, OpenTargetResolutionError};
use crate::cli_output::command_arg;
use crate::http::{http_json, http_remote_path, is_cf_http_url, parse_cf_http_url};
use crate::manifest::source_manifest_unchanged;
use crate::merge::{
    branch_operation_error, branch_state, ensure_sealed_source_state, with_branch_operation_context,
};
use crate::open_clone_idempotency::{
    clone_idempotency_payload, load_clone_idempotency, save_clone_idempotency,
};
use crate::remote::{
    copy_object_graph, load_remote_branch, parse_cf_url, remote_dirs, write_object_records,
};
use crate::repo_lock::RepoLock;
use crate::scan_verify::compute_scan_locked;
use crate::{
    absolute_path, branch_record_path, close_transaction, find_repo_root, load_branch_record,
    now_iso_utc, open_transaction, opened_registry_path, read_json, required_string,
    save_branch_record, scan_branch_state, scan_change_count, secure_worktree,
    validate_http_bundle_commit, CloneOptions, CloneResult, CloseOptions, CloseResult, OpenOptions,
    OpenResult,
};
use serde_json::{json, Value};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::{env, fs};

pub(crate) fn clone_branch(start: &Path, options: &CloneOptions) -> Result<CloneResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(&repo_root, &options.lock)?)
    };
    let idempotency_payload = clone_idempotency_payload(options, &repo_root);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_clone_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    if branch_record_path(&repo_root, &options.new_branch).exists() {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.new_branch),
            None,
            format!("branch already exists: {}", options.new_branch),
        ));
    }
    if is_cf_http_url(&options.source) {
        let remote = parse_cf_http_url(&options.source)?;
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
        let branch_record = bundle
            .get("branch")
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid branch bundle".to_string(),
                )
            })?
            .clone();
        let records = bundle
            .get("objects")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid branch bundle".to_string(),
                )
            })?;
        let source_head = required_string(&branch_record, &["head"])?;
        ensure_sealed_source_state("clone", &remote.branch, branch_state(&branch_record)).map_err(
            |error| with_branch_operation_context(error, &repo_root, Some(&remote.branch), None),
        )?;
        validate_http_bundle_commit(records, &source_head)?;
        let plan =
            clone_operation_plan(options, &repo_root, &source_head, remote.endpoint_scheme());
        if options.dry_run {
            return Ok(CloneResult { plan });
        }
        write_object_records(
            &repo_root.join(".codefire").join("objects"),
            records.iter().cloned(),
        )?;
        codefire_store::validate_sealed_commit(
            &repo_root.join(".codefire").join("objects"),
            &source_head,
        )?;
        let branch = json!({
            "type": "branch",
            "version": 1,
            "name": options.new_branch,
            "head": source_head,
            "state": "closed",
            "created_from": options.source,
            "created_at": now_iso_utc(),
        });
        save_branch_record(&repo_root, &branch)?;
        let result = CloneResult { plan };
        if let Some(key) = options.idempotency_key.as_deref() {
            save_clone_idempotency(&repo_root, key, &idempotency_payload, &result)?;
        }
        return Ok(result);
    }
    if options.source.starts_with("cf://") {
        let remote = parse_cf_url(&options.source)?;
        let source = load_remote_branch(&remote)?;
        let source_head = required_string(&source, &["head"])?;
        ensure_sealed_source_state("clone", &remote.branch, branch_state(&source)).map_err(
            |error| with_branch_operation_context(error, &repo_root, Some(&remote.branch), None),
        )?;
        let remote_objects = remote_dirs(&remote.project_root).objects;
        codefire_store::validate_sealed_commit(&remote_objects, &source_head)?;
        let plan = clone_operation_plan(options, &repo_root, &source_head, "cf");
        if options.dry_run {
            return Ok(CloneResult { plan });
        }
        copy_object_graph(
            &remote_objects,
            &repo_root.join(".codefire").join("objects"),
            &source_head,
        )?;
        let branch = json!({
            "type": "branch",
            "version": 1,
            "name": options.new_branch,
            "head": source_head,
            "state": "closed",
            "created_from": options.source,
            "created_at": now_iso_utc(),
        });
        save_branch_record(&repo_root, &branch)?;
        let result = CloneResult { plan };
        if let Some(key) = options.idempotency_key.as_deref() {
            save_clone_idempotency(&repo_root, key, &idempotency_payload, &result)?;
        }
        return Ok(result);
    }
    let source = load_branch_record(&repo_root, &options.source).map_err(|error| {
        with_branch_operation_context(error, &repo_root, Some(&options.source), None)
    })?;
    let source_head = required_string(&source, &["head"])?;
    ensure_sealed_source_state("clone", &options.source, branch_state(&source)).map_err(
        |error| with_branch_operation_context(error, &repo_root, Some(&options.source), None),
    )?;
    codefire_store::validate_sealed_commit(
        &repo_root.join(".codefire").join("objects"),
        &source_head,
    )?;
    let plan = clone_operation_plan(options, &repo_root, &source_head, "local");
    if options.dry_run {
        return Ok(CloneResult { plan });
    }
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": options.new_branch,
        "head": source_head,
        "state": "closed",
        "created_from": options.source,
        "created_at": now_iso_utc(),
    });
    save_branch_record(&repo_root, &branch)?;
    let result = CloneResult { plan };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_clone_idempotency(&repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

pub(crate) fn close_branch(start: &Path, options: &CloseOptions) -> Result<CloseResult, CliError> {
    close_transaction::close_branch_from(start, options)
}

pub(crate) fn validate_close_open_marker(
    repo_root: &Path,
    open_dir: &Path,
    branch: &str,
    registry: &Value,
    marker: &Value,
) -> Result<(), CliError> {
    let marker_branch = required_string(marker, &["branch", "name"])?;
    if marker_branch != branch {
        return Err(CliError::InvalidMarker(format!(
            "open marker branch mismatch: expected {branch}, found {marker_branch}"
        )));
    }
    let marker_repo_path = PathBuf::from(required_string(marker, &["repository", "path"])?);
    if marker_repo_path != repo_root.join(".codefire") {
        return Err(CliError::InvalidMarker(format!(
            "open marker repository mismatch for {}",
            open_dir.display()
        )));
    }
    let repository = read_json(&repo_root.join(".codefire").join("repo.json"))?;
    let expected_repository_id = required_string(&repository, &["repository_id"])?;
    let marker_repository_id = required_string(marker, &["repository", "repository_id"])?;
    if marker_repository_id != expected_repository_id {
        return Err(CliError::InvalidMarker(format!(
            "open marker repository identity mismatch for {}",
            open_dir.display()
        )));
    }
    let marker_open_path = PathBuf::from(required_string(marker, &["open", "opened_path"])?);
    if marker_open_path != open_dir {
        return Err(CliError::InvalidMarker(format!(
            "open marker path mismatch: expected {}, found {}",
            open_dir.display(),
            marker_open_path.display()
        )));
    }
    let marker_instance = required_string(marker, &["open", "open_instance_id"])?;
    let registry_instance = required_string(registry, &["open", "open_instance_id"])?;
    if marker_instance != registry_instance {
        return Err(CliError::InvalidMarker(format!(
            "open marker instance mismatch: expected {registry_instance}, found {marker_instance}"
        )));
    }
    Ok(())
}

pub(crate) fn close_operation_plan(
    options: &CloseOptions,
    repo_root: &Path,
    open_dir: &Path,
    registry_path: &Path,
    active_state_path: &Path,
    state: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": if options.discard { "discard" } else { "close" },
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "branch": &options.branch,
        "open_dir": open_dir,
        "discard": options.discard,
        "keep_dir": options.keep_dir,
        "initial_state": state,
        "operations": [
            {"kind": if options.keep_dir { "remove_open_marker" } else { "remove_open_directory" }, "path": open_dir},
            {"kind": "remove_active_state", "path": active_state_path},
            {"kind": "remove_open_registry", "path": registry_path},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "closed"},
        ],
        "next_actions": [
            {
                "kind": "branch_show",
                "command": format!("codefire branch show {} --json", command_arg(&options.branch)),
                "reason": "confirm the branch is closed",
                "target": {"branch": &options.branch, "path": repo_root},
            },
        ],
    })
}

pub(crate) fn open_branch(options: &OpenOptions) -> Result<OpenResult, CliError> {
    open_branch_from(&env::current_dir()?, options)
}

pub(crate) fn open_branch_from(
    start: &Path,
    options: &OpenOptions,
) -> Result<OpenResult, CliError> {
    open_transaction::open_branch_from(start, options)
}

pub(crate) fn open_operation_plan(
    options: &OpenOptions,
    repo_root: &Path,
    target: &Path,
    branch_head: &str,
    registry_path: &Path,
    active_path: &Path,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "open",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": &options.branch,
        "repo": repo_root,
        "target": target,
        "base_commit": branch_head,
        "operations": [
            {"kind": "materialize_commit", "commit": branch_head, "target": target, "publication": "sibling_stage"},
            {"kind": "write_open_marker", "path": target.join(".codefire-open"), "publication": "sibling_stage"},
            {"kind": "publish_open_target", "path": target, "visibility": "single_sibling_rename"},
            {"kind": "write_open_registry", "path": registry_path},
            {"kind": "create_active_state", "path": active_path},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "open-clean"},
        ],
        "next_actions": [
            {
                "kind": "status",
                "command": format!("codefire status --path {} --json", command_arg(&target.to_string_lossy())),
                "reason": "inspect the newly opened branch state",
                "target": {"path": target, "branch": &options.branch},
            },
        ],
    })
}

pub(crate) fn clone_operation_plan(
    options: &CloneOptions,
    repo_root: &Path,
    source_head: &str,
    source_kind: &str,
) -> Value {
    let suggested_open_path = suggested_branch_open_path(repo_root, &options.new_branch);
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "clone",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "source": {
            "value": &options.source,
            "kind": source_kind,
            "head": source_head,
        },
        "new_branch": &options.new_branch,
        "operations": [
            {"kind": "copy_object_graph_if_remote", "commit": source_head},
            {"kind": "write_branch_record", "branch": &options.new_branch, "head": source_head, "state": "closed"},
        ],
        "next_actions": [
            {
                "kind": "open",
                "command": format!(
                    "codefire open {} {}",
                    command_arg(&options.new_branch),
                    command_arg(&suggested_open_path.to_string_lossy())
                ),
                "reason": "open the newly cloned branch into a working directory",
                "target": {
                    "branch": &options.new_branch,
                    "repo": repo_root,
                    "recommended_path": suggested_open_path,
                    "path_policy": "use a sibling of the CodeFire repo root and never nest inside an existing open directory",
                },
            },
        ],
    })
}

pub(crate) fn suggested_branch_open_path(repo_root: &Path, branch: &str) -> PathBuf {
    let parent = repo_root.parent().unwrap_or(repo_root);
    let repo_name = repo_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("codefire-worktree");
    parent.join(format!(
        "{}-{}",
        repo_name,
        filesystem_safe_branch_suffix(branch)
    ))
}

pub(crate) fn filesystem_safe_branch_suffix(branch: &str) -> String {
    let suffix = branch
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if suffix.is_empty() {
        "branch".to_string()
    } else {
        suffix
    }
}

pub(crate) fn is_primary_branch(branch: &str) -> bool {
    branch == "main"
}

pub(crate) fn resolve_open_target(
    repo_root: &Path,
    requested: &Path,
    requested_branch: &str,
) -> Result<(PathBuf, secure_worktree::PinnedTargetResolution), CliError> {
    let boundary = fs::canonicalize(repo_root.parent().unwrap_or(repo_root))?;
    let requested_absolute = absolute_path(requested)?;
    let contains_parent = requested_absolute
        .components()
        .any(|component| component == Component::ParentDir);
    let mut lexical_target = PathBuf::new();
    for component in requested_absolute.components() {
        if component != Component::CurDir {
            lexical_target.push(component.as_os_str());
        }
    }
    if !contains_parent && !lexical_target.starts_with(&boundary) {
        return Err(open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&lexical_target),
            &boundary,
            "outside_workspace_boundary",
            format!(
                "open target must stay under resolved workspace boundary {}: requested {}, resolved {}",
                boundary.display(),
                requested.display(),
                lexical_target.display()
            ),
        ));
    }
    let target_resolution = secure_worktree::PinnedTargetResolution::resolve(&boundary, requested)
        .map_err(|error| {
            open_target_resolution_error(
                repo_root,
                requested_branch,
                requested,
                None,
                &boundary,
                "unsafe_path_component",
                error.to_string(),
            )
        })?;
    let target = target_resolution.target_path().to_path_buf();
    if !target.starts_with(&boundary) {
        return Err(open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&target),
            &boundary,
            "outside_workspace_boundary",
            format!(
                "open target must stay under resolved workspace boundary {}: requested {}, resolved {}",
                boundary.display(),
                requested.display(),
                target.display()
            ),
        ));
    }
    if target == repo_root || target.starts_with(repo_root) || repo_root.starts_with(&target) {
        return Err(open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&target),
            &boundary,
            "repository_overlap",
            format!(
                "open target must not overlap repository root {}: requested {}, resolved {}; choose a directory outside the repository root, for example {}",
                repo_root.display(),
                requested.display(),
                target.display(),
                suggested_open_dir(repo_root, &boundary, requested_branch).display()
            ),
        ));
    }
    let opened_dir = repo_root.join(".codefire").join("opened");
    if opened_dir.is_dir() {
        for entry in fs::read_dir(opened_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let registry = read_json(&entry.path())?;
                let registered_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
                let open_path = resolve_path_without_symlinks(&registered_path)?;
                let registered_branch = required_string(&registry, &["branch", "name"])?;
                if target == open_path && registered_branch == requested_branch {
                    continue;
                }
                if target.starts_with(&open_path) || open_path.starts_with(&target) {
                    return Err(open_target_resolution_error(
                        repo_root,
                        requested_branch,
                        requested,
                        Some(&target),
                        &boundary,
                        "open_directory_overlap",
                        format!(
                            "open target must not be nested inside existing open directory or overlap it {}: requested {}, resolved {}",
                            open_path.display(),
                            requested.display(),
                            target.display()
                        ),
                    ));
                }
            }
        }
    }
    target_resolution.ensure_current().map_err(|error| {
        open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&target),
            &boundary,
            "identity_changed_during_resolution",
            format!("open target ancestor changed during resolution: {error}"),
        )
    })?;
    Ok((target, target_resolution))
}

pub(crate) fn open_target_resolution_error(
    repo_root: &Path,
    branch: &str,
    requested_path: &Path,
    resolved_path: Option<&Path>,
    workspace_boundary: &Path,
    reason: &'static str,
    message: String,
) -> CliError {
    CliError::OpenTargetResolution(Box::new(OpenTargetResolutionError {
        message,
        reason,
        repo_root: repo_root.to_path_buf(),
        branch: branch.to_string(),
        requested_path: requested_path.to_path_buf(),
        resolved_path: resolved_path.map(Path::to_path_buf),
        workspace_boundary: workspace_boundary.to_path_buf(),
    }))
}

/// After `clone`, the branch exists but has no working directory yet.
pub(crate) fn open_next_step(branch: &str) -> Option<String> {
    let repo_root = find_repo_root(&env::current_dir().ok()?).ok()?;
    let boundary = repo_root.parent()?;
    Some(format!(
        "Next: codefire open {branch} {}    (any directory beside the repository root)",
        suggested_open_dir(&repo_root, boundary, branch).display()
    ))
}

/// A directory beside the repository root to suggest after a rejected target.
pub(crate) fn suggested_open_dir(repo_root: &Path, boundary: &Path, branch: &str) -> PathBuf {
    let leaf = branch.replace('/', "-");
    let candidate = boundary.join(&leaf);
    if candidate.starts_with(repo_root) || repo_root.starts_with(&candidate) {
        boundary.join(format!("{leaf}-open"))
    } else {
        candidate
    }
}

pub(crate) fn resolve_path_without_symlinks(path: &Path) -> Result<PathBuf, CliError> {
    let absolute = absolute_path(path)?;
    if absolute
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(CliError::Usage(format!(
            "path traversal with '..' is not allowed for open targets: {}; use an absolute path without '..' (open directories live beside the repository root, not inside it)",
            path.display()
        )));
    }

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => unreachable!("parent components were rejected above"),
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    let mut existing_ancestor = normalized.as_path();
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(existing_ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(CliError::Usage(format!(
                        "symlink components are not allowed for open targets: {}",
                        existing_ancestor.display()
                    )));
                }
                if !suffix.is_empty() && !metadata.is_dir() {
                    return Err(CliError::Usage(format!(
                        "open target ancestor is not a directory: {}",
                        existing_ancestor.display()
                    )));
                }
                break;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let name = existing_ancestor.file_name().ok_or_else(|| {
                    CliError::Usage(format!(
                        "open target has no existing filesystem ancestor: {}",
                        path.display()
                    ))
                })?;
                suffix.push(name.to_os_string());
                existing_ancestor = existing_ancestor.parent().ok_or_else(|| {
                    CliError::Usage(format!(
                        "open target has no existing filesystem ancestor: {}",
                        path.display()
                    ))
                })?;
            }
            Err(error) => return Err(CliError::Io(error)),
        }
    }

    let mut prefix = PathBuf::new();
    for component in existing_ancestor.components() {
        prefix.push(component.as_os_str());
        if fs::symlink_metadata(&prefix)?.file_type().is_symlink() {
            return Err(CliError::Usage(format!(
                "symlink components are not allowed for open targets: {}",
                prefix.display()
            )));
        }
    }

    let mut resolved = fs::canonicalize(existing_ancestor)?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

pub(crate) fn refresh_open_clean_for_branch_operation(
    repo_root: &Path,
    branch: &str,
    open_dir: &Path,
    lock: &RepoLock,
) -> Result<(), CliError> {
    let registry_path = opened_registry_path(repo_root, branch);
    let registry = read_json(&registry_path)?;
    let base_commit = required_string(&registry, &["open", "current_base_commit"])?;
    let objects = repo_root.join(".codefire").join("objects");
    let base_path = codefire_store::object_record_path(&objects, &base_commit)
        .unwrap_or_else(|| objects.clone());
    codefire_store::validate_sealed_commit(&objects, &base_commit).map_err(|error| {
        CliError::StoreContext {
            error: Box::new(error),
            repo_root: repo_root.to_path_buf(),
            path: base_path,
        }
    })?;
    if source_manifest_unchanged(&objects, &base_commit, open_dir)? {
        let state = required_string(&registry, &["state", "last_known"])?;
        if state == "open-clean" {
            return Ok(());
        }
        return Err(branch_operation_error(
            repo_root,
            Some(branch),
            Some(open_dir),
            format!("branch {branch} is {state}; commit, verify, or discard before continuing"),
        ));
    }
    let scan = compute_scan_locked(open_dir, lock).map_err(|error| {
        with_branch_operation_context(error, repo_root, Some(branch), Some(open_dir))
    })?;
    let state = scan_branch_state(scan_change_count(&scan.scan), scan.scan.open_fires.len());
    if state == "open-clean" {
        Ok(())
    } else {
        Err(branch_operation_error(
            repo_root,
            Some(branch),
            Some(open_dir),
            format!(
                "branch {branch} has unsealed open work after refresh ({state}); commit, verify, or discard before continuing"
            ),
        ))
    }
}
