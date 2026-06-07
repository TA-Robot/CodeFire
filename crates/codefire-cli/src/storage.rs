use super::remote::{parse_cf_project_url, remote_dirs};
use super::{find_repo_root, read_json, CliError};
use crate::automation::cli_command;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_LARGE_THRESHOLD_BYTES: u64 = 1_048_576;

#[derive(Debug)]
pub(crate) struct StorageReportOptions {
    pub(crate) start: PathBuf,
    pub(crate) json_output: bool,
    pub(crate) large_threshold_bytes: u64,
    pub(crate) remotes: Vec<String>,
    pub(crate) quick: bool,
}

#[derive(Debug)]
pub(crate) struct StorageReport {
    pub(crate) repo_root: PathBuf,
    pub(crate) objects: AreaStats,
    pub(crate) active_state: AreaStats,
    pub(crate) idempotency: AreaStats,
    pub(crate) object_types: Vec<ObjectTypeStats>,
    pub(crate) largest_objects: Vec<ObjectFileStats>,
    pub(crate) external_artifacts: ExternalArtifactStats,
    pub(crate) remotes: Vec<RemoteStorageStats>,
    pub(crate) warnings: Vec<StorageWarning>,
    pub(crate) quick: bool,
    pub(crate) skipped_checks: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AreaStats {
    pub(crate) files: u64,
    pub(crate) bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectTypeStats {
    pub(crate) type_tag: String,
    pub(crate) files: u64,
    pub(crate) bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectFileStats {
    pub(crate) object_id: String,
    pub(crate) type_tag: String,
    pub(crate) path: PathBuf,
    pub(crate) bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExternalArtifactStats {
    pub(crate) refs: u64,
    pub(crate) referenced_bytes: u64,
    pub(crate) payload_bytes_stored: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemoteStorageStats {
    pub(crate) url: String,
    pub(crate) project_root: PathBuf,
    pub(crate) objects: AreaStats,
    pub(crate) branches: AreaStats,
    pub(crate) merge_requests: AreaStats,
    pub(crate) idempotency: AreaStats,
    pub(crate) idempotency_retention: RemoteIdempotencyRetentionStats,
    pub(crate) retention: RemoteRetentionStats,
    pub(crate) objects_by_generation: Vec<RemoteGenerationStats>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemoteIdempotencyRetentionStats {
    pub(crate) retention_seconds: u64,
    pub(crate) oldest_created_at: Option<String>,
    pub(crate) expired_files: u64,
    pub(crate) expired_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemoteRetentionStats {
    pub(crate) retention_seconds: u64,
    pub(crate) retention_generations: u64,
    pub(crate) idempotency_retention_seconds: u64,
    pub(crate) current_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemoteGenerationStats {
    pub(crate) generation: Option<u64>,
    pub(crate) files: u64,
    pub(crate) bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorageWarning {
    pub(crate) kind: String,
    pub(crate) message: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) bytes: Option<u64>,
}

pub(crate) fn parse_storage_report_args(args: &[String]) -> Result<StorageReportOptions, CliError> {
    let mut start = None;
    let mut json_output = false;
    let mut large_threshold_bytes = DEFAULT_LARGE_THRESHOLD_BYTES;
    let mut remotes = Vec::new();
    let mut quick = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--quick" => quick = true,
            "--full" => quick = false,
            "--remote" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--remote requires a value".to_string()))?;
                remotes.push(value.to_string());
            }
            "--large-threshold" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CliError::Usage("--large-threshold requires a value".to_string())
                })?;
                large_threshold_bytes = parse_byte_size(value)?;
            }
            value if value.starts_with("--large-threshold=") => {
                large_threshold_bytes =
                    parse_byte_size(value.trim_start_matches("--large-threshold="))?;
            }
            value if value.starts_with("--remote=") => {
                remotes.push(value.trim_start_matches("--remote=").to_string());
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported storage report option: {option}"
                )));
            }
            value if start.is_none() => start = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected storage report argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(StorageReportOptions {
        start: start.unwrap_or(std::env::current_dir()?),
        json_output,
        large_threshold_bytes,
        remotes,
        quick,
    })
}

pub(crate) fn run_storage_report(
    options: &StorageReportOptions,
) -> Result<StorageReport, CliError> {
    let repo_root = find_repo_root(&options.start)?;
    let cf = repo_root.join(".codefire");
    let object_scan = scan_objects(
        &cf.join("objects"),
        options.large_threshold_bytes,
        options.quick,
    )?;
    let remote_scan = scan_remote_storage(&options.remotes)?;
    let mut warnings = object_scan.warnings;
    warnings.extend(remote_scan.warnings);
    Ok(StorageReport {
        repo_root,
        active_state: scan_area(&cf.join("active"))?,
        idempotency: scan_area(&cf.join("idempotency"))?,
        objects: object_scan.area,
        object_types: object_scan.object_types,
        largest_objects: object_scan.largest_objects,
        external_artifacts: object_scan.external_artifacts,
        remotes: remote_scan.remotes,
        warnings,
        quick: options.quick,
        skipped_checks: object_scan.skipped_checks,
    })
}

pub(crate) fn storage_report_data_json(report: &StorageReport) -> Value {
    json!({
        "type": "codefire_storage_report",
        "version": 1,
        "repo_root": &report.repo_root,
        "mode": if report.quick { "quick" } else { "full" },
        "skipped_checks": &report.skipped_checks,
        "objects": {
            "files": report.objects.files,
            "bytes": report.objects.bytes,
            "by_type": report.object_types.iter().map(object_type_json).collect::<Vec<_>>(),
            "largest": report.largest_objects.iter().map(object_file_json).collect::<Vec<_>>(),
        },
        "active_state": {
            "files": report.active_state.files,
            "bytes": report.active_state.bytes,
        },
        "idempotency": {
            "files": report.idempotency.files,
            "bytes": report.idempotency.bytes,
        },
        "external_artifacts": {
            "refs": report.external_artifacts.refs,
            "referenced_bytes": report.external_artifacts.referenced_bytes,
            "payload_bytes_stored": report.external_artifacts.payload_bytes_stored,
        },
        "remotes": report.remotes.iter().map(remote_storage_json).collect::<Vec<_>>(),
        "warnings": report.warnings.iter().map(storage_warning_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn storage_report_diagnostics_json(report: &StorageReport) -> Vec<Value> {
    report
        .warnings
        .iter()
        .map(storage_warning_json)
        .collect::<Vec<_>>()
}

pub(crate) fn storage_report_next_actions(report: &StorageReport) -> Vec<Value> {
    if report.warnings.is_empty() {
        return Vec::new();
    }
    vec![json!({
        "id": "inspect_storage_warnings",
        "command": cli_command("storage report --json"),
        "description": "inspect storage warnings and largest objects",
        "context": {"warnings": report.warnings.len()},
    })]
}

pub(crate) fn print_storage_report(report: &StorageReport) {
    println!("CodeFire storage report");
    println!("repo: {}", report.repo_root.display());
    println!("mode: {}", if report.quick { "quick" } else { "full" });
    if !report.skipped_checks.is_empty() {
        println!("skipped checks: {}", report.skipped_checks.join(", "));
    }
    println!(
        "objects: {} files, {}",
        report.objects.files,
        format_bytes(report.objects.bytes)
    );
    println!(
        "active state: {} files, {}",
        report.active_state.files,
        format_bytes(report.active_state.bytes)
    );
    println!(
        "idempotency: {} files, {}",
        report.idempotency.files,
        format_bytes(report.idempotency.bytes)
    );
    println!(
        "external artifacts: {} refs, {} referenced, {} payload bytes stored",
        report.external_artifacts.refs,
        format_bytes(report.external_artifacts.referenced_bytes),
        format_bytes(report.external_artifacts.payload_bytes_stored)
    );
    println!("remotes:");
    if report.remotes.is_empty() {
        println!("  none");
    } else {
        for remote in &report.remotes {
            println!(
                "  {}: objects {} files, {}; branches {}; merge requests {}",
                remote.url,
                remote.objects.files,
                format_bytes(remote.objects.bytes),
                remote.branches.files,
                remote.merge_requests.files
            );
            println!(
                "    retention: {}s, {} generation(s), current generation {}; idempotency {}s, expired {}",
                remote.retention.retention_seconds,
                remote.retention.retention_generations,
                remote.retention.current_generation,
                remote.idempotency_retention.retention_seconds,
                remote.idempotency_retention.expired_files
            );
        }
    }
    println!("largest object types:");
    if report.object_types.is_empty() {
        println!("  none");
    } else {
        for stats in &report.object_types {
            println!(
                "  {}: {} files, {}",
                stats.type_tag,
                stats.files,
                format_bytes(stats.bytes)
            );
        }
    }
    println!("warnings:");
    if report.warnings.is_empty() {
        println!("  none");
    } else {
        for warning in &report.warnings {
            match (&warning.path, warning.bytes) {
                (Some(path), Some(bytes)) => {
                    println!(
                        "  {}: {} ({}, {})",
                        warning.kind,
                        warning.message,
                        path.display(),
                        format_bytes(bytes)
                    );
                }
                (Some(path), None) => {
                    println!(
                        "  {}: {} ({})",
                        warning.kind,
                        warning.message,
                        path.display()
                    );
                }
                _ => println!("  {}: {}", warning.kind, warning.message),
            }
        }
    }
}

struct ObjectScan {
    area: AreaStats,
    object_types: Vec<ObjectTypeStats>,
    largest_objects: Vec<ObjectFileStats>,
    external_artifacts: ExternalArtifactStats,
    warnings: Vec<StorageWarning>,
    skipped_checks: Vec<String>,
}

fn scan_objects(
    objects_root: &Path,
    large_threshold_bytes: u64,
    quick: bool,
) -> Result<ObjectScan, CliError> {
    let mut area = AreaStats::default();
    let mut by_type = BTreeMap::<String, AreaStats>::new();
    let mut largest_objects = Vec::new();
    let mut external_artifacts = ExternalArtifactStats::default();
    let mut warnings = Vec::new();
    let skipped_checks = if quick {
        vec![
            "object_json_validation".to_string(),
            "object_type_breakdown".to_string(),
            "external_artifact_refs".to_string(),
        ]
    } else {
        Vec::new()
    };
    for path in collect_files(objects_root)? {
        let bytes = fs::metadata(&path)?.len();
        area.files += 1;
        area.bytes += bytes;
        if quick {
            let object_id = path
                .file_stem()
                .and_then(|name| name.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| "(unknown)".to_string());
            if bytes >= large_threshold_bytes {
                warnings.push(StorageWarning {
                    kind: "large_object".to_string(),
                    message: "object file exceeds large threshold".to_string(),
                    path: Some(path.clone()),
                    bytes: Some(bytes),
                });
            }
            largest_objects.push(ObjectFileStats {
                object_id,
                type_tag: "unscanned".to_string(),
                path,
                bytes,
            });
            continue;
        }
        let value = match read_json(&path) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(StorageWarning {
                    kind: "invalid_object_json".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                    bytes: Some(bytes),
                });
                continue;
            }
        };
        let record = match serde_json::from_value::<codefire_store::ObjectRecord>(value) {
            Ok(record) => record,
            Err(error) => {
                warnings.push(StorageWarning {
                    kind: "invalid_object_record".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                    bytes: Some(bytes),
                });
                continue;
            }
        };
        let type_tag = record.type_tag.clone();
        let object_id = record.object_id.clone();
        let stats = by_type.entry(type_tag.clone()).or_default();
        stats.files += 1;
        stats.bytes += bytes;
        if let Some(payload_type) = record.payload.get("type").and_then(Value::as_str) {
            if payload_type != type_tag {
                warnings.push(StorageWarning {
                    kind: "payload_type_mismatch".to_string(),
                    message: format!(
                        "object record type '{type_tag}' differs from payload type '{payload_type}'"
                    ),
                    path: Some(path.clone()),
                    bytes: Some(bytes),
                });
            }
        }
        if type_tag == "artifact_ref" {
            external_artifacts.refs += 1;
            external_artifacts.referenced_bytes += record
                .payload
                .get("size_bytes")
                .and_then(Value::as_u64)
                .unwrap_or(0);
        }
        if bytes >= large_threshold_bytes {
            warnings.push(StorageWarning {
                kind: "large_object".to_string(),
                message: format!("{type_tag} object exceeds large threshold"),
                path: Some(path.clone()),
                bytes: Some(bytes),
            });
        }
        largest_objects.push(ObjectFileStats {
            object_id,
            type_tag,
            path,
            bytes,
        });
    }
    let mut object_types = by_type
        .into_iter()
        .map(|(type_tag, stats)| ObjectTypeStats {
            type_tag,
            files: stats.files,
            bytes: stats.bytes,
        })
        .collect::<Vec<_>>();
    object_types.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.type_tag.cmp(&right.type_tag))
    });
    largest_objects.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.object_id.cmp(&right.object_id))
    });
    largest_objects.truncate(10);
    Ok(ObjectScan {
        area,
        object_types,
        largest_objects,
        external_artifacts,
        warnings,
        skipped_checks,
    })
}

fn scan_area(path: &Path) -> Result<AreaStats, CliError> {
    let mut stats = AreaStats::default();
    for file in collect_files(path)? {
        stats.files += 1;
        stats.bytes += fs::metadata(file)?.len();
    }
    Ok(stats)
}

struct RemoteStorageScan {
    remotes: Vec<RemoteStorageStats>,
    warnings: Vec<StorageWarning>,
}

fn scan_remote_storage(remote_urls: &[String]) -> Result<RemoteStorageScan, CliError> {
    let mut remotes = Vec::new();
    let mut warnings = Vec::new();
    for url in remote_urls {
        let project = parse_cf_project_url(url)?;
        let project_root = project.project_root;
        let dirs = remote_dirs(&project_root);
        let retention = scan_remote_retention(&project_root)?;
        let idempotency = scan_area(&dirs.idempotency)?;
        let idempotency_retention = scan_remote_idempotency_retention(
            &dirs.idempotency,
            retention.idempotency_retention_seconds,
        )?;
        if idempotency_retention.expired_files > 0 {
            warnings.push(StorageWarning {
                kind: "remote_idempotency_retention".to_string(),
                message: format!(
                    "{} remote idempotency record(s) exceed retention {}s",
                    idempotency_retention.expired_files, idempotency_retention.retention_seconds
                ),
                path: Some(dirs.idempotency.clone()),
                bytes: Some(idempotency_retention.expired_bytes),
            });
        }
        remotes.push(RemoteStorageStats {
            url: url.clone(),
            project_root: project_root.clone(),
            objects: scan_area(&dirs.objects)?,
            branches: scan_area(&dirs.branches)?,
            merge_requests: scan_area(&dirs.merge_requests)?,
            idempotency,
            idempotency_retention,
            retention,
            objects_by_generation: scan_remote_object_generations(&dirs.objects)?,
        });
    }
    Ok(RemoteStorageScan { remotes, warnings })
}

fn scan_remote_retention(project_root: &Path) -> Result<RemoteRetentionStats, CliError> {
    let policy = read_optional_json_value(&project_root.join("server_policy.json"))?;
    let state = read_optional_json_value(&project_root.join("gc_state.json"))?;
    Ok(RemoteRetentionStats {
        retention_seconds: policy
            .as_ref()
            .and_then(|policy| policy.get("gc"))
            .and_then(|gc| gc.get("retention_seconds"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        retention_generations: policy
            .as_ref()
            .and_then(|policy| policy.get("gc"))
            .and_then(|gc| gc.get("retention_generations"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        idempotency_retention_seconds: policy
            .as_ref()
            .and_then(|policy| policy.get("gc"))
            .and_then(|gc| gc.get("idempotency_retention_seconds"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        current_generation: state
            .as_ref()
            .and_then(|state| state.get("current_generation"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
    })
}

fn scan_remote_idempotency_retention(
    idempotency_root: &Path,
    retention_seconds: u64,
) -> Result<RemoteIdempotencyRetentionStats, CliError> {
    let now = unix_now_seconds();
    let mut stats = RemoteIdempotencyRetentionStats {
        retention_seconds,
        ..RemoteIdempotencyRetentionStats::default()
    };
    for path in collect_files(idempotency_root)? {
        let bytes = fs::metadata(&path)?.len();
        let Ok(record) = read_json(&path) else {
            continue;
        };
        let Some(created_at) = record.get("created_at").and_then(Value::as_str) else {
            continue;
        };
        let is_oldest = match stats.oldest_created_at.as_deref() {
            Some(oldest) => created_at < oldest,
            None => true,
        };
        if is_oldest {
            stats.oldest_created_at = Some(created_at.to_string());
        }
        if retention_seconds > 0 {
            if let Some(created_at_seconds) = parse_iso_utc_seconds(created_at) {
                if now.saturating_sub(created_at_seconds) > retention_seconds as i64 {
                    stats.expired_files += 1;
                    stats.expired_bytes += bytes;
                }
            }
        }
    }
    Ok(stats)
}

fn scan_remote_object_generations(
    objects_root: &Path,
) -> Result<Vec<RemoteGenerationStats>, CliError> {
    let mut by_generation = BTreeMap::<Option<u64>, AreaStats>::new();
    for path in collect_files(objects_root)? {
        let bytes = fs::metadata(&path)?.len();
        let generation = read_json(&path).ok().and_then(|value| {
            value
                .get("remote")
                .and_then(|remote| remote.get("last_seen_generation"))
                .and_then(Value::as_u64)
        });
        let stats = by_generation.entry(generation).or_default();
        stats.files += 1;
        stats.bytes += bytes;
    }
    Ok(by_generation
        .into_iter()
        .map(|(generation, stats)| RemoteGenerationStats {
            generation,
            files: stats.files,
            bytes: stats.bytes,
        })
        .collect())
}

fn unix_now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn parse_iso_utc_seconds(value: &str) -> Option<i64> {
    if value.len() != 20
        || !value.ends_with('Z')
        || &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
    {
        return None;
    }
    let year = value[0..4].parse::<i32>().ok()?;
    let month = value[5..7].parse::<u32>().ok()?;
    let day = value[8..10].parse::<u32>().ok()?;
    let hour = value[11..13].parse::<u32>().ok()?;
    let minute = value[14..16].parse::<u32>().ok()?;
    let second = value[17..19].parse::<u32>().ok()?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    Some(
        days_from_civil(year, month, day) * 86_400
            + i64::from(hour) * 3_600
            + i64::from(minute) * 60
            + i64::from(second),
    )
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 }.div_euclid(400);
    let yoe = year - era * 400;
    let month = i64::from(month);
    let doy =
        (153 * (month + if month > 2 { -3 } else { 9 }) + 2).div_euclid(5) + i64::from(day) - 1;
    let doe = yoe * 365 + yoe.div_euclid(4) - yoe.div_euclid(100) + doy;
    era * 146_097 + doe - 719_468
}

fn read_optional_json_value(path: &Path) -> Result<Option<Value>, CliError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CliError::Io(error)),
    }
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn object_type_json(stats: &ObjectTypeStats) -> Value {
    json!({
        "type": &stats.type_tag,
        "files": stats.files,
        "bytes": stats.bytes,
    })
}

fn object_file_json(stats: &ObjectFileStats) -> Value {
    json!({
        "object_id": &stats.object_id,
        "type": &stats.type_tag,
        "path": &stats.path,
        "bytes": stats.bytes,
    })
}

fn storage_warning_json(warning: &StorageWarning) -> Value {
    json!({
        "kind": &warning.kind,
        "message": &warning.message,
        "path": &warning.path,
        "bytes": warning.bytes,
    })
}

fn remote_storage_json(stats: &RemoteStorageStats) -> Value {
    json!({
        "url": &stats.url,
        "project_root": &stats.project_root,
        "objects": {"files": stats.objects.files, "bytes": stats.objects.bytes},
        "branches": {"files": stats.branches.files, "bytes": stats.branches.bytes},
        "merge_requests": {"files": stats.merge_requests.files, "bytes": stats.merge_requests.bytes},
        "idempotency": {
            "files": stats.idempotency.files,
            "bytes": stats.idempotency.bytes,
            "retention_seconds": stats.idempotency_retention.retention_seconds,
            "oldest_created_at": stats.idempotency_retention.oldest_created_at,
            "expired_files": stats.idempotency_retention.expired_files,
            "expired_bytes": stats.idempotency_retention.expired_bytes,
        },
        "retention": {
            "retention_seconds": stats.retention.retention_seconds,
            "retention_generations": stats.retention.retention_generations,
            "idempotency_retention_seconds": stats.retention.idempotency_retention_seconds,
            "current_generation": stats.retention.current_generation,
        },
        "objects_by_generation": stats.objects_by_generation.iter().map(remote_generation_json).collect::<Vec<_>>(),
    })
}

fn remote_generation_json(stats: &RemoteGenerationStats) -> Value {
    json!({
        "generation": stats.generation,
        "files": stats.files,
        "bytes": stats.bytes,
    })
}

fn parse_byte_size(value: &str) -> Result<u64, CliError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CliError::Usage(
            "--large-threshold must not be empty".to_string(),
        ));
    }
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(split_at);
    if number.is_empty() {
        return Err(CliError::Usage(format!(
            "invalid byte size for --large-threshold: {value}"
        )));
    }
    let base = number.parse::<u64>().map_err(|_| {
        CliError::Usage(format!("invalid byte size for --large-threshold: {value}"))
    })?;
    let multiplier = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024 * 1024,
        "g" | "gb" | "gib" => 1024 * 1024 * 1024,
        _ => {
            return Err(CliError::Usage(format!(
                "unsupported byte size unit for --large-threshold: {value}"
            )))
        }
    };
    base.checked_mul(multiplier).ok_or_else(|| {
        CliError::Usage(format!(
            "byte size is too large for --large-threshold: {value}"
        ))
    })
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}
