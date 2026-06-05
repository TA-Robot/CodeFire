mod batch;

pub(crate) use batch::{evidence_batch_data_json, run_evidence_batch};

use super::{find_repo_root, now_iso_utc, CliError};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const DEFAULT_MAX_OUTPUT_BYTES: usize = 64 * 1024;

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
    pub(crate) command_cwd: Option<PathBuf>,
    pub(crate) max_output_bytes: usize,
}

#[derive(Debug)]
pub(crate) struct EvidenceAddResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) evidence_id: String,
    pub(crate) artifact_ref_id: Option<String>,
    pub(crate) command_exit_code: Option<i32>,
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
    let mut command_cwd = None;
    let mut max_output_bytes = DEFAULT_MAX_OUTPUT_BYTES;
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
            "--cwd" => {
                index += 1;
                command_cwd = Some(PathBuf::from(required_arg(args, index, "--cwd")?));
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
            value if value.starts_with("--cwd=") => {
                command_cwd = Some(PathBuf::from(value.trim_start_matches("--cwd=")));
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
            || command_cwd.is_some())
    {
        return Err(CliError::Usage(
            "evidence add --batch cannot be combined with single-item capture options".to_string(),
        ));
    }
    if batch_path.is_none() && dry_run {
        return Err(CliError::Usage(
            "evidence add --dry-run requires --batch".to_string(),
        ));
    }
    if batch_path.is_none() && artifact_path.is_none() && command.is_none() {
        return Err(CliError::Usage(
            "evidence add requires --artifact or --from-command".to_string(),
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
        command_cwd,
        max_output_bytes,
    })
}

pub(crate) fn run_evidence_add(
    options: &EvidenceAddOptions,
) -> Result<EvidenceAddResult, CliError> {
    let repo_root = find_repo_root(&options.start)?;
    validate_evidence_add_options(options)?;
    let objects = repo_root.join(".codefire").join("objects");
    let artifact_ref_id = match options.artifact_path.as_deref() {
        Some(path) => Some(store_artifact_ref(&objects, options, path)?),
        None => None,
    };
    let command_capture = match options.command.as_deref() {
        Some(command) => Some(capture_command(command, options)?),
        None => None,
    };
    let command_exit_code = command_capture
        .as_ref()
        .and_then(|capture| capture.exit_code);
    let evidence_id = codefire_store::store_object(
        &objects,
        "evidence",
        json!({
            "type": "evidence",
            "version": 1,
            "label": &options.label,
            "artifact_ref": &artifact_ref_id,
            "command": command_capture.as_ref().map(CommandCapture::to_json),
            "created_at": now_iso_utc(),
        }),
    )?;
    Ok(EvidenceAddResult {
        repo_root,
        evidence_id,
        artifact_ref_id,
        command_exit_code,
    })
}

pub(super) fn validate_evidence_add_options(options: &EvidenceAddOptions) -> Result<(), CliError> {
    if options.artifact_path.is_none() && options.command.is_none() {
        return Err(CliError::Usage(
            "evidence add requires --artifact or --from-command".to_string(),
        ));
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
    json!({
        "type": "codefire_evidence_add_result",
        "version": 1,
        "evidence_id": &result.evidence_id,
        "artifact_ref_id": &result.artifact_ref_id,
        "command_exit_code": result.command_exit_code,
    })
}

pub(crate) fn print_evidence_add_result(result: &EvidenceAddResult) {
    println!("recorded evidence: {}", result.evidence_id);
    if let Some(artifact_ref_id) = &result.artifact_ref_id {
        println!("artifact ref: {artifact_ref_id}");
    }
    if let Some(exit_code) = result.command_exit_code {
        println!("command exit code: {exit_code}");
    }
}

fn store_artifact_ref(
    objects: &Path,
    options: &EvidenceAddOptions,
    path: &Path,
) -> Result<String, CliError> {
    let absolute = absolute_existing_path(path)?;
    let metadata = fs::metadata(&absolute)?;
    if !metadata.is_file() {
        return Err(CliError::Usage(format!(
            "artifact path must be a file: {}",
            absolute.display()
        )));
    }
    let content_hash = sha256_file(&absolute)?;
    let uri = options
        .artifact_uri
        .clone()
        .unwrap_or_else(|| absolute.to_string_lossy().into_owned());
    codefire_store::store_object(
        objects,
        "artifact_ref",
        json!({
            "type": "artifact_ref",
            "version": 1,
            "label": &options.label,
            "uri": uri,
            "path": absolute,
            "hash_algorithm": "sha256",
            "content_hash": format!("sha256:{content_hash}"),
            "size_bytes": metadata.len(),
            "captured_at": now_iso_utc(),
        }),
    )
    .map_err(CliError::from)
}

#[derive(Debug)]
struct CommandCapture {
    command: String,
    cwd: PathBuf,
    exit_code: Option<i32>,
    success: bool,
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
            "cwd": &self.cwd,
            "exit_code": self.exit_code,
            "success": self.success,
            "duration_ms": self.duration_ms,
            "stdout": &self.stdout,
            "stdout_truncated": self.stdout_truncated,
            "stderr": &self.stderr,
            "stderr_truncated": self.stderr_truncated,
        })
    }
}

fn capture_command(
    command: &str,
    options: &EvidenceAddOptions,
) -> Result<CommandCapture, CliError> {
    let cwd = options
        .command_cwd
        .clone()
        .unwrap_or(std::env::current_dir()?);
    let start = Instant::now();
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&cwd)
        .output()?;
    let duration_ms = start.elapsed().as_millis();
    let (stdout, stdout_truncated) = lossy_truncated(&output.stdout, options.max_output_bytes);
    let (stderr, stderr_truncated) = lossy_truncated(&output.stderr, options.max_output_bytes);
    Ok(CommandCapture {
        command: command.to_string(),
        cwd,
        exit_code: output.status.code(),
        success: output.status.success(),
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
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_lower(&hasher.finalize()))
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

fn required_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
