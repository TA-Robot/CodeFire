mod batch;

pub(crate) use batch::{evidence_batch_data_json, run_evidence_batch};

use super::{
    command_execution::{minimal_command_environment, run_bounded_command, CommandExecutionSpec},
    find_repo_root, now_iso_utc, read_json, required_string,
    secret_redaction::{redact_argv, redact_literals, redact_text, Redacted},
    CliError,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) const DEFAULT_MAX_OUTPUT_BYTES: usize = 64 * 1024;
pub(super) const DEFAULT_COMMAND_TIMEOUT_MS: u64 = 300_000;
pub(super) const LARGE_ARTIFACT_WARNING_BYTES: u64 = 1_048_576;
const ARTIFACT_HASH_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub(crate) struct EvidenceAddOptions {
    pub(crate) start: PathBuf,
    pub(crate) json_output: bool,
    pub(crate) dry_run: bool,
    pub(crate) batch_path: Option<PathBuf>,
    pub(crate) label: Option<String>,
    pub(crate) artifact_path: Option<PathBuf>,
    pub(crate) artifact_uri: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) command_argv: Option<Vec<String>>,
    pub(crate) command_cwd: Option<PathBuf>,
    pub(crate) command_timeout_ms: u64,
    pub(crate) max_output_bytes: usize,
    pub(crate) allow_failed_command: bool,
}

#[derive(Debug)]
pub(crate) struct EvidenceAddResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) dry_run: bool,
    pub(crate) evidence_id: String,
    pub(crate) artifact_ref_id: Option<String>,
    pub(crate) object_path: Option<PathBuf>,
    pub(crate) command_exit_code: Option<i32>,
    pub(crate) command_timed_out: bool,
    pub(crate) command_stdout_summary: Option<String>,
    pub(crate) command_stdout_truncated: bool,
    pub(crate) command_stderr_summary: Option<String>,
    pub(crate) command_stderr_truncated: bool,
    pub(crate) command_cwd: Option<String>,
    pub(crate) command_cwd_source: Option<&'static str>,
    pub(crate) repo_relative_command_cwd: Option<String>,
    pub(crate) diagnostics: Vec<Value>,
    pub(crate) plan: Option<Value>,
}

#[derive(Debug, Clone)]
struct CommandCwdResolution {
    resolved: PathBuf,
    source: &'static str,
    repo_relative: Option<String>,
    portable: String,
    redacted: bool,
}

pub(crate) fn parse_evidence_add_args(args: &[String]) -> Result<EvidenceAddOptions, CliError> {
    let mut start = None;
    let mut json_output = false;
    let mut dry_run = false;
    let mut batch_path = None;
    let mut label = None;
    let mut artifact_path = None;
    let mut artifact_uri = None;
    let mut command = None;
    let mut command_argv = None::<Vec<String>>;
    let mut command_cwd = None;
    let mut command_timeout_ms = DEFAULT_COMMAND_TIMEOUT_MS;
    let mut max_output_bytes = DEFAULT_MAX_OUTPUT_BYTES;
    let mut allow_failed_command = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--dry-run" => dry_run = true,
            "--batch" => {
                index += 1;
                batch_path = Some(PathBuf::from(required_arg(args, index, "--batch")?));
            }
            "--path" => {
                index += 1;
                start = Some(PathBuf::from(required_arg(args, index, "--path")?));
            }
            "--label" => {
                index += 1;
                label = Some(required_arg(args, index, "--label")?.to_string());
            }
            "--artifact" => {
                index += 1;
                artifact_path = Some(PathBuf::from(required_arg(args, index, "--artifact")?));
            }
            "--artifact-uri" => {
                index += 1;
                artifact_uri = Some(required_arg(args, index, "--artifact-uri")?.to_string());
            }
            "--from-command" => {
                index += 1;
                command = Some(required_arg(args, index, "--from-command")?.to_string());
            }
            "--from-argv" => {
                index += 1;
                command_argv = Some(vec![required_arg(args, index, "--from-argv")?.to_string()]);
            }
            "--argv" => {
                index += 1;
                let value = required_arg(args, index, "--argv")?.to_string();
                let argv = command_argv.as_mut().ok_or_else(|| {
                    CliError::Usage("--argv requires --from-argv first".to_string())
                })?;
                argv.push(value);
            }
            "--cwd" => {
                index += 1;
                command_cwd = Some(PathBuf::from(required_arg(args, index, "--cwd")?));
            }
            "--allow-failed-command" => allow_failed_command = true,
            "--timeout" | "--timeout-ms" => {
                index += 1;
                command_timeout_ms = parse_duration_ms(required_arg(args, index, "--timeout")?)?;
            }
            "--max-output-bytes" => {
                index += 1;
                max_output_bytes = parse_usize(required_arg(args, index, "--max-output-bytes")?)?;
            }
            value if value.starts_with("--path=") => {
                start = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--batch=") => {
                batch_path = Some(PathBuf::from(value.trim_start_matches("--batch=")));
            }
            value if value.starts_with("--label=") => {
                label = Some(value.trim_start_matches("--label=").to_string());
            }
            value if value.starts_with("--artifact=") => {
                artifact_path = Some(PathBuf::from(value.trim_start_matches("--artifact=")));
            }
            value if value.starts_with("--artifact-uri=") => {
                artifact_uri = Some(value.trim_start_matches("--artifact-uri=").to_string());
            }
            value if value.starts_with("--from-command=") => {
                command = Some(value.trim_start_matches("--from-command=").to_string());
            }
            value if value.starts_with("--from-argv=") => {
                command_argv = Some(vec![value.trim_start_matches("--from-argv=").to_string()]);
            }
            value if value.starts_with("--argv=") => {
                let argv = command_argv.as_mut().ok_or_else(|| {
                    CliError::Usage("--argv requires --from-argv first".to_string())
                })?;
                argv.push(value.trim_start_matches("--argv=").to_string());
            }
            value if value.starts_with("--cwd=") => {
                command_cwd = Some(PathBuf::from(value.trim_start_matches("--cwd=")));
            }
            "--allow-failed-command=true" => {
                allow_failed_command = true;
            }
            "--allow-failed-command=false" => {
                allow_failed_command = false;
            }
            value if value.starts_with("--timeout=") => {
                command_timeout_ms = parse_duration_ms(value.trim_start_matches("--timeout="))?;
            }
            value if value.starts_with("--timeout-ms=") => {
                command_timeout_ms = parse_duration_ms(value.trim_start_matches("--timeout-ms="))?;
            }
            value if value.starts_with("--max-output-bytes=") => {
                max_output_bytes = parse_usize(value.trim_start_matches("--max-output-bytes="))?;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported evidence add option: {option}"
                )));
            }
            value if start.is_none() => start = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected evidence add argument: {value}"
                )));
            }
        }
        index += 1;
    }
    if batch_path.is_some()
        && (artifact_path.is_some()
            || artifact_uri.is_some()
            || command.is_some()
            || command_argv.is_some()
            || command_cwd.is_some())
    {
        return Err(CliError::Usage(
            "evidence add --batch cannot be combined with single-item capture options".to_string(),
        ));
    }
    if command.is_some() && command_argv.is_some() {
        return Err(CliError::Usage(
            "evidence add cannot combine --from-command and --from-argv".to_string(),
        ));
    }
    if batch_path.is_none()
        && artifact_path.is_none()
        && command.is_none()
        && command_argv.is_none()
    {
        return Err(CliError::Usage(
            "evidence add requires --artifact, --from-command, or --from-argv".to_string(),
        ));
    }
    Ok(EvidenceAddOptions {
        start: start.unwrap_or(std::env::current_dir()?),
        json_output,
        dry_run,
        batch_path,
        label,
        artifact_path,
        artifact_uri,
        command,
        command_argv,
        command_cwd,
        command_timeout_ms,
        max_output_bytes,
        allow_failed_command,
    })
}

pub(crate) fn run_evidence_add(
    options: &EvidenceAddOptions,
) -> Result<EvidenceAddResult, CliError> {
    let repo_root = find_repo_root(&options.start)?;
    validate_evidence_add_options(options)?;
    let command_cwd = if options.command.is_some() || options.command_argv.is_some() {
        Some(resolve_command_cwd(options, &repo_root)?)
    } else {
        None
    };
    let artifact_plan_path = options
        .artifact_path
        .as_deref()
        .map(|path| portable_plan_path(path, &repo_root))
        .transpose()?;
    if options.dry_run {
        return Ok(EvidenceAddResult {
            repo_root,
            dry_run: true,
            evidence_id: String::new(),
            artifact_ref_id: None,
            object_path: None,
            command_exit_code: None,
            command_timed_out: false,
            command_stdout_summary: None,
            command_stdout_truncated: false,
            command_stderr_summary: None,
            command_stderr_truncated: false,
            command_cwd: command_cwd.as_ref().map(|cwd| cwd.portable.clone()),
            command_cwd_source: command_cwd.as_ref().map(|cwd| cwd.source),
            repo_relative_command_cwd: command_cwd
                .as_ref()
                .and_then(|cwd| cwd.repo_relative.clone()),
            diagnostics: Vec::new(),
            plan: Some(evidence_add_operation_plan(
                options,
                command_cwd.as_ref(),
                artifact_plan_path.as_deref(),
            )),
        });
    }
    let objects = repo_root.join(".codefire").join("objects");
    let mut diagnostics = Vec::new();
    let artifact_ref_id = match options.artifact_path.as_deref() {
        Some(path) => {
            let artifact = store_artifact_ref(&objects, options, path, &repo_root)?;
            diagnostics.extend(artifact.diagnostics);
            Some(artifact.id)
        }
        None => None,
    };
    let command_capture = if let Some(command) = options.command.as_deref() {
        Some(capture_shell_command(
            command,
            options,
            command_cwd.as_ref().expect("command cwd resolved"),
            &repo_root,
        )?)
    } else if let Some(argv) = options.command_argv.as_deref() {
        Some(capture_argv_command(
            argv,
            options,
            command_cwd.as_ref().expect("command cwd resolved"),
            &repo_root,
        )?)
    } else {
        None
    };
    if let Some(capture) = command_capture.as_ref() {
        validate_command_capture_success(capture, options)?;
    }
    let command_exit_code = command_capture
        .as_ref()
        .and_then(|capture| capture.exit_code);
    let command_stdout_summary = command_capture
        .as_ref()
        .map(|capture| capture.stdout.clone());
    let command_stdout_truncated = command_capture
        .as_ref()
        .map(|capture| capture.stdout_truncated)
        .unwrap_or(false);
    let command_stderr_summary = command_capture
        .as_ref()
        .map(|capture| capture.stderr.clone());
    let command_stderr_truncated = command_capture
        .as_ref()
        .map(|capture| capture.stderr_truncated)
        .unwrap_or(false);
    let command_cwd_result = command_cwd.as_ref().map(|cwd| cwd.portable.clone());
    let command_cwd_source = command_cwd.as_ref().map(|cwd| cwd.source);
    let repo_relative_command_cwd = command_cwd
        .as_ref()
        .and_then(|cwd| cwd.repo_relative.clone());
    let proof_status = command_capture
        .as_ref()
        .map(|capture| {
            if capture.success {
                "command-passed"
            } else {
                "failed-command-captured"
            }
        })
        .unwrap_or("captured");
    if let Some(capture) = command_capture.as_ref() {
        if !capture.success {
            diagnostics.push(command_failure_diagnostic(capture, "warning"));
        }
        if capture.redaction_count > 0 {
            diagnostics.push(json!({
                "kind": "evidence_secret_redacted",
                "severity": "warning",
                "blocking": false,
                "redacted_value_count": capture.redaction_count,
                "message": "secret-like command metadata or output was redacted before evidence storage",
            }));
        }
    }
    let evidence_id = codefire_store::store_object(
        &objects,
        "evidence",
        json!({
            "type": "evidence",
            "version": 1,
            "label": &options.label,
            "artifact_ref": &artifact_ref_id,
            "command": command_capture.as_ref().map(CommandCapture::to_json),
            "proof_status": proof_status,
            "command_cwd": &command_cwd_result,
            "command_cwd_source": command_cwd_source,
            "repo_relative_command_cwd": &repo_relative_command_cwd,
            "created_at": now_iso_utc(),
        }),
    )?;
    let object_path = codefire_store::object_record_path(&objects, &evidence_id);
    Ok(EvidenceAddResult {
        repo_root,
        dry_run: false,
        evidence_id,
        artifact_ref_id,
        object_path,
        command_exit_code,
        command_timed_out: command_capture
            .as_ref()
            .map(|capture| capture.timed_out)
            .unwrap_or(false),
        command_stdout_summary,
        command_stdout_truncated,
        command_stderr_summary,
        command_stderr_truncated,
        command_cwd: command_cwd_result,
        command_cwd_source,
        repo_relative_command_cwd,
        diagnostics,
        plan: None,
    })
}

pub(super) fn validate_evidence_add_options(options: &EvidenceAddOptions) -> Result<(), CliError> {
    if options.command.is_some() && options.command_argv.is_some() {
        return Err(CliError::Usage(
            "evidence add cannot combine --from-command and --from-argv".to_string(),
        ));
    }
    if options.artifact_path.is_none()
        && options.command.is_none()
        && options.command_argv.is_none()
    {
        return Err(CliError::Usage(
            "evidence add requires --artifact, --from-command, or --from-argv".to_string(),
        ));
    }
    if let Some(argv) = options.command_argv.as_deref() {
        if argv.is_empty() || argv[0].trim().is_empty() {
            return Err(CliError::Usage(
                "evidence add --from-argv requires a program".to_string(),
            ));
        }
    }
    if let Some(path) = options.artifact_path.as_deref() {
        let absolute = absolute_existing_path(path).map_err(|error| {
            CliError::Usage(format!(
                "artifact path must exist: {} ({error})",
                path.display()
            ))
        })?;
        let metadata = fs::metadata(&absolute)?;
        if !metadata.is_file() {
            return Err(CliError::Usage(format!(
                "artifact path must be a file: {}",
                absolute.display()
            )));
        }
    }
    if let Some(cwd) = options.command_cwd.as_deref() {
        let absolute = absolute_existing_path(cwd).map_err(|error| {
            CliError::Usage(format!(
                "command cwd must exist: {} ({error})",
                cwd.display()
            ))
        })?;
        if !absolute.is_dir() {
            return Err(CliError::Usage(format!(
                "command cwd must be a directory: {}",
                absolute.display()
            )));
        }
    }
    Ok(())
}

pub(crate) fn evidence_add_data_json(result: &EvidenceAddResult) -> Value {
    let evidence_id = if result.evidence_id.is_empty() {
        Value::Null
    } else {
        Value::String(result.evidence_id.clone())
    };
    json!({
        "type": "codefire_evidence_add_result",
        "version": 1,
        "dry_run": result.dry_run,
        "created": !result.dry_run,
        "evidence_id": evidence_id,
        "artifact_ref_id": &result.artifact_ref_id,
        "object_path": &result.object_path,
        "command_exit_code": result.command_exit_code,
        "command_timed_out": result.command_timed_out,
        "command_stdout_summary": &result.command_stdout_summary,
        "command_stdout_truncated": result.command_stdout_truncated,
        "command_stderr_summary": &result.command_stderr_summary,
        "command_stderr_truncated": result.command_stderr_truncated,
        "command_cwd": &result.command_cwd,
        "command_cwd_source": result.command_cwd_source,
        "repo_relative_command_cwd": &result.repo_relative_command_cwd,
        "diagnostics": &result.diagnostics,
        "plan": &result.plan,
    })
}

pub(crate) fn print_evidence_add_result(result: &EvidenceAddResult) {
    if result.dry_run {
        println!("evidence dry-run: capture plan validated");
        return;
    }
    println!("recorded evidence: {}", result.evidence_id);
    if let Some(object_path) = &result.object_path {
        println!("object path: {}", object_path.display());
    }
    if let Some(artifact_ref_id) = &result.artifact_ref_id {
        println!("artifact ref: {artifact_ref_id}");
    }
    if let Some(exit_code) = result.command_exit_code {
        println!("command exit code: {exit_code}");
    }
    if result.command_timed_out {
        println!("command timed out: true");
    }
    for diagnostic in &result.diagnostics {
        if let Some(message) = diagnostic.get("message").and_then(Value::as_str) {
            println!("warning: {message}");
        }
    }
}

fn evidence_add_operation_plan(
    options: &EvidenceAddOptions,
    command_cwd: Option<&CommandCwdResolution>,
    artifact_path: Option<&str>,
) -> Value {
    let sensitive_literals = command_cwd.map(sensitive_path_literals).unwrap_or_default();
    let from_command = options
        .command
        .as_deref()
        .map(|value| redact_capture_text(value, &sensitive_literals))
        .map(|value| value.value);
    let command_argv = options
        .command_argv
        .as_deref()
        .map(|value| redact_capture_argv(value, &sensitive_literals))
        .map(|value| value.value);
    let artifact_uri = options
        .artifact_uri
        .as_deref()
        .map(redact_text)
        .map(|value| value.value);
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "evidence-add",
        "dry_run": true,
        "would_apply": false,
        "label": &options.label,
        "has_artifact": options.artifact_path.is_some(),
        "artifact": artifact_path,
        "artifact_uri": artifact_uri,
        "has_command": options.command.is_some(),
        "from_command": from_command,
        "has_command_argv": options.command_argv.is_some(),
        "command_argv": command_argv,
        "requested_cwd": command_cwd.map(|cwd| &cwd.portable),
        "cwd": command_cwd.map(|cwd| &cwd.portable),
        "resolved_cwd": command_cwd.map(|cwd| &cwd.portable),
        "cwd_redacted": command_cwd.is_some_and(|cwd| cwd.redacted),
        "cwd_source": command_cwd.map(|cwd| cwd.source),
        "repo_relative_cwd": command_cwd.and_then(|cwd| cwd.repo_relative.as_ref()),
        "timeout_ms": options.command_timeout_ms,
        "max_output_bytes": options.max_output_bytes,
        "allow_failed_command": options.allow_failed_command,
        "operations": [
            {"kind": "validate_capture_inputs"},
            {"kind": "store_artifact_ref", "enabled": options.artifact_path.is_some()},
            {"kind": "run_command_capture", "enabled": options.command.is_some() || options.command_argv.is_some()},
            {"kind": "store_evidence_object"}
        ],
        "next_actions": [
            {"kind": "evidence_add", "command": "codefire evidence add ... --json"},
            {"kind": "storage_report", "command": "codefire storage report --json"}
        ],
    })
}

fn portable_plan_path(path: &Path, repo_root: &Path) -> Result<String, CliError> {
    let absolute = absolute_existing_path(path)?;
    Ok(absolute
        .strip_prefix(repo_root)
        .ok()
        .map(path_slash_string)
        .unwrap_or_else(|| "<redacted>".to_string()))
}

struct StoredArtifactRef {
    id: String,
    diagnostics: Vec<Value>,
}

struct ArtifactLocation {
    uri: String,
    path: String,
    path_kind: &'static str,
    local_path_redacted: bool,
    uri_redacted_count: usize,
}

fn artifact_location(
    absolute: &Path,
    repo_root: &Path,
    explicit_uri: Option<&str>,
    content_hash: &str,
) -> ArtifactLocation {
    let explicit_uri = explicit_uri.map(redact_text);
    let uri_redacted_count = explicit_uri.as_ref().map_or(0, |value| value.count);
    let explicit_uri = explicit_uri.map(|value| value.value);
    if let Ok(relative) = absolute.strip_prefix(repo_root) {
        let relative = path_slash_string(relative);
        return ArtifactLocation {
            uri: explicit_uri.unwrap_or_else(|| format!("repo://{relative}")),
            path: relative,
            path_kind: "repo_relative",
            local_path_redacted: false,
            uri_redacted_count,
        };
    }
    ArtifactLocation {
        uri: explicit_uri.unwrap_or_else(|| format!("artifact://redacted/{}", &content_hash[..16])),
        path: "<redacted>".to_string(),
        path_kind: "redacted",
        local_path_redacted: true,
        uri_redacted_count,
    }
}

fn path_slash_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn resolve_command_cwd(
    options: &EvidenceAddOptions,
    repo_root: &Path,
) -> Result<CommandCwdResolution, CliError> {
    let (resolved, source) = if let Some(cwd) = options.command_cwd.as_deref() {
        (absolute_existing_path(cwd)?, "explicit_cwd")
    } else {
        (
            absolute_existing_path(&std::env::current_dir()?)?,
            "caller_cwd",
        )
    };
    if !resolved.is_dir() {
        return Err(CliError::Usage(format!(
            "command cwd must be a directory: {}",
            resolved.display()
        )));
    }
    let repo_relative = resolved.strip_prefix(repo_root).ok().map(path_slash_string);
    let redacted = repo_relative.is_none();
    let portable = repo_relative
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(if redacted { "<redacted>" } else { "." })
        .to_string();
    Ok(CommandCwdResolution {
        resolved,
        source,
        repo_relative,
        portable,
        redacted,
    })
}

fn store_artifact_ref(
    objects: &Path,
    options: &EvidenceAddOptions,
    path: &Path,
    repo_root: &Path,
) -> Result<StoredArtifactRef, CliError> {
    let absolute = absolute_existing_path(path)?;
    let metadata = fs::metadata(&absolute)?;
    if !metadata.is_file() {
        return Err(CliError::Usage(format!(
            "artifact path must be a file: {}",
            absolute.display()
        )));
    }
    let content_hash = sha256_file(&absolute)?;
    let location = artifact_location(
        &absolute,
        repo_root,
        options.artifact_uri.as_deref(),
        &content_hash,
    );
    let large_artifact = metadata.len() >= LARGE_ARTIFACT_WARNING_BYTES;
    let mut diagnostics = Vec::new();
    if location.local_path_redacted {
        diagnostics.push(json!({
            "kind": "artifact_path_redacted",
            "severity": "warning",
            "message": "artifact path was outside the repository and was redacted before storing evidence",
        }));
    }
    if location.uri_redacted_count > 0 {
        diagnostics.push(json!({
            "kind": "artifact_uri_secret_redacted",
            "severity": "warning",
            "message": "secret-like artifact URI metadata was redacted before evidence storage",
            "redacted_value_count": location.uri_redacted_count,
        }));
    }
    if large_artifact {
        diagnostics.push(json!({
            "kind": "large_artifact",
            "severity": "warning",
            "message": format!("artifact is {} bytes; hash was computed with streaming SHA-256", metadata.len()),
            "bytes": metadata.len(),
            "threshold_bytes": LARGE_ARTIFACT_WARNING_BYTES,
        }));
    }
    let id = codefire_store::store_object(
        objects,
        "artifact_ref",
        json!({
            "type": "artifact_ref",
            "version": 1,
            "label": &options.label,
            "uri": location.uri,
            "path": location.path,
            "path_kind": location.path_kind,
            "local_path_redacted": location.local_path_redacted,
            "uri_secret_redacted": location.uri_redacted_count > 0,
            "uri_redacted_value_count": location.uri_redacted_count,
            "hash_algorithm": "sha256",
            "content_hash": format!("sha256:{content_hash}"),
            "hash_streaming": true,
            "hash_chunk_bytes": ARTIFACT_HASH_BUFFER_BYTES,
            "large_artifact": large_artifact,
            "large_artifact_threshold_bytes": LARGE_ARTIFACT_WARNING_BYTES,
            "size_bytes": metadata.len(),
            "captured_at": now_iso_utc(),
        }),
    )
    .map_err(CliError::from)?;
    Ok(StoredArtifactRef { id, diagnostics })
}

#[derive(Debug)]
struct CommandCapture {
    command: String,
    argv: Option<Vec<String>>,
    cwd: String,
    cwd_kind: &'static str,
    mode: &'static str,
    shell: bool,
    profile: &'static str,
    phase: &'static str,
    trust_kind: &'static str,
    portability: &'static str,
    environment_policy: &'static str,
    environment_keys: Vec<String>,
    environment_digest: String,
    input_digest: String,
    subject: Value,
    exit_code: Option<i32>,
    success: bool,
    timed_out: bool,
    timeout_ms: u64,
    duration_ms: u128,
    stdout: String,
    stdout_bytes_seen: u64,
    stdout_bytes_stored: usize,
    stdout_truncated: bool,
    stderr: String,
    stderr_bytes_seen: u64,
    stderr_bytes_stored: usize,
    stderr_truncated: bool,
    termination_scope: &'static str,
    redaction_count: usize,
}

struct CommandInvocation {
    command: String,
    argv: Option<Vec<String>>,
    mode: &'static str,
    shell: bool,
    program: String,
    args: Vec<String>,
}

impl CommandCapture {
    fn to_json(&self) -> Value {
        json!({
            "command": &self.command,
            "argv": &self.argv,
            "cwd": &self.cwd,
            "cwd_kind": self.cwd_kind,
            "mode": self.mode,
            "shell": self.shell,
            "profile": self.profile,
            "phase": self.phase,
            "trust_kind": self.trust_kind,
            "portability": self.portability,
            "environment_policy": self.environment_policy,
            "environment_keys": &self.environment_keys,
            "environment_digest": &self.environment_digest,
            "input_digest": &self.input_digest,
            "subject": &self.subject,
            "exit_code": self.exit_code,
            "success": self.success,
            "timed_out": self.timed_out,
            "timeout_ms": self.timeout_ms,
            "duration_ms": self.duration_ms,
            "stdout": &self.stdout,
            "stdout_bytes_seen": self.stdout_bytes_seen,
            "stdout_bytes_stored": self.stdout_bytes_stored,
            "stdout_truncated": self.stdout_truncated,
            "stderr": &self.stderr,
            "stderr_bytes_seen": self.stderr_bytes_seen,
            "stderr_bytes_stored": self.stderr_bytes_stored,
            "stderr_truncated": self.stderr_truncated,
            "termination_scope": self.termination_scope,
            "redacted_value_count": self.redaction_count,
        })
    }
}

fn capture_shell_command(
    command: &str,
    options: &EvidenceAddOptions,
    cwd: &CommandCwdResolution,
    repo_root: &Path,
) -> Result<CommandCapture, CliError> {
    finish_command_capture(
        CommandInvocation {
            command: command.to_string(),
            argv: None,
            mode: "shell",
            shell: true,
            program: "sh".to_string(),
            args: vec!["-c".to_string(), command.to_string()],
        },
        cwd,
        repo_root,
        options,
    )
}

fn capture_argv_command(
    argv: &[String],
    options: &EvidenceAddOptions,
    cwd: &CommandCwdResolution,
    repo_root: &Path,
) -> Result<CommandCapture, CliError> {
    let Some((program, args)) = argv.split_first() else {
        return Err(CliError::Usage(
            "evidence add --from-argv requires a program".to_string(),
        ));
    };
    finish_command_capture(
        CommandInvocation {
            command: argv.join(" "),
            argv: Some(argv.to_vec()),
            mode: "argv",
            shell: false,
            program: program.clone(),
            args: args.to_vec(),
        },
        cwd,
        repo_root,
        options,
    )
}

fn validate_command_capture_success(
    capture: &CommandCapture,
    options: &EvidenceAddOptions,
) -> Result<(), CliError> {
    if capture.success || options.allow_failed_command {
        return Ok(());
    }
    Err(CliError::EvidenceCommandFailed {
        exit_code: capture.exit_code,
        timed_out: capture.timed_out,
        cwd: capture.cwd.clone(),
        message: command_failure_message(capture),
    })
}

fn command_failure_diagnostic(capture: &CommandCapture, severity: &str) -> Value {
    json!({
        "kind": "evidence_command_failed",
        "severity": severity,
        "blocking": severity == "blocking",
        "exit_code": capture.exit_code,
        "timed_out": capture.timed_out,
        "cwd": &capture.cwd,
        "message": command_failure_message(capture),
    })
}

fn command_failure_message(capture: &CommandCapture) -> String {
    if capture.timed_out {
        format!(
            "evidence command timed out after {}ms in {}",
            capture.timeout_ms, capture.cwd
        )
    } else {
        format!(
            "evidence command failed with exit code {} in {}",
            capture
                .exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            capture.cwd
        )
    }
}

fn finish_command_capture(
    invocation: CommandInvocation,
    cwd: &CommandCwdResolution,
    repo_root: &Path,
    options: &EvidenceAddOptions,
) -> Result<CommandCapture, CliError> {
    let environment = minimal_command_environment();
    let environment_keys = environment.keys().cloned().collect::<Vec<_>>();
    let environment_digest = super::command_execution::canonical_digest(&json!(&environment))?;
    let subject =
        command_execution_subject(repo_root, cwd, &invocation, &environment_digest, options)?;
    let CommandInvocation {
        command,
        argv,
        mode,
        shell,
        program,
        args,
    } = invocation;
    let output = run_bounded_command(&CommandExecutionSpec {
        program,
        args,
        cwd: cwd.resolved.clone(),
        clear_env: true,
        env: environment,
        timeout_ms: options.command_timeout_ms,
        stdout_limit: options.max_output_bytes,
        stderr_limit: options.max_output_bytes,
    })?;
    let sensitive_literals = sensitive_path_literals(cwd);
    let redacted_command = redact_capture_text(&command, &sensitive_literals);
    let redacted_argv = argv
        .as_deref()
        .map(|values| redact_capture_argv(values, &sensitive_literals));
    let stdout = redact_capture_text(&output.stdout.text, &sensitive_literals);
    let stderr = redact_capture_text(&output.stderr.text, &sensitive_literals);
    let redaction_count = redacted_command.count
        + redacted_argv.as_ref().map_or(0, |value| value.count)
        + stdout.count
        + stderr.count;
    let input_digest = required_string(&subject, &["input_digest"])?;
    Ok(CommandCapture {
        command: redacted_command.value,
        argv: redacted_argv.map(|value| value.value),
        cwd: cwd.portable.clone(),
        cwd_kind: if cwd.redacted {
            "external_redacted"
        } else {
            "repo_relative"
        },
        mode,
        shell,
        profile: "local_evidence_command_v1",
        phase: "evidence_capture",
        trust_kind: "local_unisolated",
        portability: if cwd.redacted {
            "local_only_redacted"
        } else {
            "portable_redacted"
        },
        environment_policy: "minimal_allowlist",
        environment_keys,
        environment_digest,
        input_digest,
        subject,
        exit_code: output.exit_code,
        success: output.success,
        timed_out: output.timed_out,
        timeout_ms: options.command_timeout_ms,
        duration_ms: output.duration_ms,
        stdout: stdout.value,
        stdout_bytes_seen: output.stdout.bytes_seen,
        stdout_bytes_stored: output.stdout.bytes_stored,
        stdout_truncated: output.stdout.truncated,
        stderr: stderr.value,
        stderr_bytes_seen: output.stderr.bytes_seen,
        stderr_bytes_stored: output.stderr.bytes_stored,
        stderr_truncated: output.stderr.truncated,
        termination_scope: output.termination_scope,
        redaction_count,
    })
}

fn command_execution_subject(
    repo_root: &Path,
    cwd: &CommandCwdResolution,
    invocation: &CommandInvocation,
    environment_digest: &str,
    options: &EvidenceAddOptions,
) -> Result<Value, CliError> {
    let repo = read_json(&repo_root.join(".codefire").join("repo.json"))?;
    let repository_id = required_string(&repo, &["repository_id"])?;
    let input = json!({
        "mode": invocation.mode,
        "command": &invocation.command,
        "argv": &invocation.argv,
        "cwd_kind": if cwd.redacted { "external_redacted" } else { "repo_relative" },
        "cwd": &cwd.portable,
        "environment_policy": "minimal_allowlist",
        "environment_digest": environment_digest,
        "timeout_ms": options.command_timeout_ms,
        "max_output_bytes": options.max_output_bytes,
    });
    let input_digest = format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&input)?)
    );
    let identity = json!({
        "type": "codefire_execution_subject",
        "version": 1,
        "profile": "local_evidence_command_v1",
        "phase": "evidence_capture",
        "trust_kind": "local_unisolated",
        "portability": if cwd.redacted { "local_only_redacted" } else { "portable_redacted" },
        "repository_id": repository_id,
        "cwd_kind": if cwd.redacted { "external_redacted" } else { "repo_relative" },
        "cwd": &cwd.portable,
        "input_digest": input_digest,
        "environment_digest": environment_digest,
    });
    let subject_digest = format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&identity)?)
    );
    let mut subject = identity;
    subject["subject_digest"] = Value::String(subject_digest);
    Ok(subject)
}

fn sensitive_path_literals(cwd: &CommandCwdResolution) -> Vec<String> {
    let mut literals = Vec::new();
    if cwd.redacted {
        literals.push(cwd.resolved.to_string_lossy().into_owned());
    }
    for key in ["HOME", "USERPROFILE"] {
        if let Ok(value) = std::env::var(key) {
            literals.push(value);
        }
    }
    literals
}

fn redact_capture_text(input: &str, literals: &[String]) -> Redacted<String> {
    let secret = redact_text(input);
    let path = redact_literals(&secret.value, literals);
    Redacted {
        value: path.value,
        count: secret.count + path.count,
    }
}

fn redact_capture_argv(argv: &[String], literals: &[String]) -> Redacted<Vec<String>> {
    let secret = redact_argv(argv);
    let mut count = secret.count;
    let value = secret
        .value
        .into_iter()
        .map(|argument| {
            let path = redact_literals(&argument, literals);
            count += path.count;
            path.value
        })
        .collect();
    Redacted { value, count }
}

fn sha256_file(path: &Path) -> Result<String, CliError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; ARTIFACT_HASH_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(codefire_util::hex_lower(&hasher.finalize()))
}

fn absolute_existing_path(path: &Path) -> Result<PathBuf, CliError> {
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }
    .canonicalize()?)
}

fn parse_usize(value: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|_| CliError::Usage(format!("invalid usize value: {value}")))
}

pub(super) fn parse_duration_ms(value: &str) -> Result<u64, CliError> {
    let parsed = if let Some(milliseconds) = value.strip_suffix("ms") {
        milliseconds.parse::<u64>().ok()
    } else if let Some(seconds) = value.strip_suffix('s') {
        seconds
            .parse::<u64>()
            .ok()
            .and_then(|seconds| seconds.checked_mul(1000))
    } else {
        value.parse::<u64>().ok()
    };
    match parsed {
        Some(0) | None => Err(CliError::Usage(format!(
            "invalid timeout duration: {value}"
        ))),
        Some(milliseconds) => Ok(milliseconds),
    }
}

fn required_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}
