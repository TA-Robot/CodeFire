mod batch;

pub(crate) use batch::{evidence_batch_data_json, run_evidence_batch};

use super::{find_repo_root, now_iso_utc, CliError};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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
    pub(crate) command_cwd: Option<PathBuf>,
    pub(crate) command_cwd_source: Option<&'static str>,
    pub(crate) repo_relative_command_cwd: Option<String>,
    pub(crate) diagnostics: Vec<Value>,
    pub(crate) plan: Option<Value>,
}

#[derive(Debug, Clone)]
struct CommandCwdResolution {
    requested: Option<PathBuf>,
    resolved: PathBuf,
    source: &'static str,
    repo_relative: Option<String>,
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
            command_cwd: command_cwd.as_ref().map(|cwd| cwd.resolved.clone()),
            command_cwd_source: command_cwd.as_ref().map(|cwd| cwd.source),
            repo_relative_command_cwd: command_cwd
                .as_ref()
                .and_then(|cwd| cwd.repo_relative.clone()),
            diagnostics: Vec::new(),
            plan: Some(evidence_add_operation_plan(options, command_cwd.as_ref())),
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
        )?)
    } else if let Some(argv) = options.command_argv.as_deref() {
        Some(capture_argv_command(
            argv,
            options,
            command_cwd.as_ref().expect("command cwd resolved"),
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
    let command_cwd_result = command_cwd.as_ref().map(|cwd| cwd.resolved.clone());
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
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "evidence-add",
        "dry_run": true,
        "would_apply": false,
        "label": &options.label,
        "has_artifact": options.artifact_path.is_some(),
        "artifact": &options.artifact_path,
        "artifact_uri": &options.artifact_uri,
        "has_command": options.command.is_some(),
        "from_command": &options.command,
        "has_command_argv": options.command_argv.is_some(),
        "command_argv": &options.command_argv,
        "requested_cwd": command_cwd.and_then(|cwd| cwd.requested.as_ref()),
        "cwd": command_cwd.map(|cwd| &cwd.resolved),
        "resolved_cwd": command_cwd.map(|cwd| &cwd.resolved),
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

struct StoredArtifactRef {
    id: String,
    diagnostics: Vec<Value>,
}

struct ArtifactLocation {
    uri: String,
    path: String,
    path_kind: &'static str,
    local_path_redacted: bool,
}

fn artifact_location(
    absolute: &Path,
    repo_root: &Path,
    explicit_uri: Option<&str>,
    content_hash: &str,
) -> ArtifactLocation {
    if let Ok(relative) = absolute.strip_prefix(repo_root) {
        let relative = path_slash_string(relative);
        return ArtifactLocation {
            uri: explicit_uri
                .map(str::to_string)
                .unwrap_or_else(|| format!("repo://{relative}")),
            path: relative,
            path_kind: "repo_relative",
            local_path_redacted: false,
        };
    }
    ArtifactLocation {
        uri: explicit_uri
            .map(str::to_string)
            .unwrap_or_else(|| format!("artifact://redacted/{}", &content_hash[..16])),
        path: "<redacted>".to_string(),
        path_kind: "redacted",
        local_path_redacted: true,
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
    let (requested, resolved, source) = if let Some(cwd) = options.command_cwd.as_deref() {
        (
            Some(cwd.to_path_buf()),
            absolute_existing_path(cwd)?,
            "explicit_cwd",
        )
    } else {
        (
            None,
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
    Ok(CommandCwdResolution {
        requested,
        resolved,
        source,
        repo_relative,
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
    cwd: PathBuf,
    mode: &'static str,
    shell: bool,
    exit_code: Option<i32>,
    success: bool,
    timed_out: bool,
    timeout_ms: u64,
    duration_ms: u128,
    stdout: String,
    stdout_truncated: bool,
    stderr: String,
    stderr_truncated: bool,
}

impl CommandCapture {
    fn to_json(&self) -> Value {
        json!({
            "command": &self.command,
            "argv": &self.argv,
            "cwd": &self.cwd,
            "mode": self.mode,
            "shell": self.shell,
            "exit_code": self.exit_code,
            "success": self.success,
            "timed_out": self.timed_out,
            "timeout_ms": self.timeout_ms,
            "duration_ms": self.duration_ms,
            "stdout": &self.stdout,
            "stdout_truncated": self.stdout_truncated,
            "stderr": &self.stderr,
            "stderr_truncated": self.stderr_truncated,
        })
    }
}

fn capture_shell_command(
    command: &str,
    options: &EvidenceAddOptions,
    cwd: &CommandCwdResolution,
) -> Result<CommandCapture, CliError> {
    let mut command_process = Command::new("sh");
    command_process
        .arg("-c")
        .arg(command)
        .current_dir(&cwd.resolved)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command_process.spawn()?;
    finish_command_capture(
        command.to_string(),
        None,
        "shell",
        true,
        cwd.resolved.clone(),
        child,
        options,
    )
}

fn capture_argv_command(
    argv: &[String],
    options: &EvidenceAddOptions,
    cwd: &CommandCwdResolution,
) -> Result<CommandCapture, CliError> {
    let Some((program, args)) = argv.split_first() else {
        return Err(CliError::Usage(
            "evidence add --from-argv requires a program".to_string(),
        ));
    };
    let mut command_process = Command::new(program);
    command_process
        .args(args)
        .current_dir(&cwd.resolved)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command_process.spawn()?;
    finish_command_capture(
        argv.join(" "),
        Some(argv.to_vec()),
        "argv",
        false,
        cwd.resolved.clone(),
        child,
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
            capture.timeout_ms,
            capture.cwd.display()
        )
    } else {
        format!(
            "evidence command failed with exit code {} in {}",
            capture
                .exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            capture.cwd.display()
        )
    }
}

fn finish_command_capture(
    command: String,
    argv: Option<Vec<String>>,
    mode: &'static str,
    shell: bool,
    cwd: PathBuf,
    mut child: std::process::Child,
    options: &EvidenceAddOptions,
) -> Result<CommandCapture, CliError> {
    let start = Instant::now();
    let timeout = Duration::from_millis(options.command_timeout_ms);
    let mut timed_out = false;
    while child.try_wait()?.is_none() {
        if start.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output()?;
    let duration_ms = start.elapsed().as_millis();
    let (stdout, stdout_truncated) = lossy_truncated(&output.stdout, options.max_output_bytes);
    let (stderr, stderr_truncated) = lossy_truncated(&output.stderr, options.max_output_bytes);
    Ok(CommandCapture {
        command,
        argv,
        cwd,
        mode,
        shell,
        exit_code: output.status.code(),
        success: output.status.success() && !timed_out,
        timed_out,
        timeout_ms: options.command_timeout_ms,
        duration_ms,
        stdout,
        stdout_truncated,
        stderr,
        stderr_truncated,
    })
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

fn lossy_truncated(bytes: &[u8], limit: usize) -> (String, bool) {
    if bytes.len() <= limit {
        return (String::from_utf8_lossy(bytes).into_owned(), false);
    }
    let mut end = limit;
    while end > 0 && std::str::from_utf8(&bytes[..end]).is_err() {
        end -= 1;
    }
    (String::from_utf8_lossy(&bytes[..end]).into_owned(), true)
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
