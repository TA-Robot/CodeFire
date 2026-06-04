use serde_json::Value;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    match args.first().map(String::as_str) {
        Some("status") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let status = read_status(&start)?;
            print_status(&status);
            Ok(())
        }
        Some("--version") | Some("version") => {
            println!("codefire-rs foundation {}", codefire_core::VERSION);
            Ok(())
        }
        Some(command) => Err(CliError::Usage(format!("unsupported command: {command}"))),
        None => {
            println!("codefire-rs foundation {}", codefire_core::VERSION);
            Ok(())
        }
    }
}

#[derive(Debug)]
enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Store(codefire_store::StoreError),
    Usage(String),
    NotOpen(PathBuf),
    InvalidMarker(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(error) => write!(f, "{error}"),
            CliError::Json(error) => write!(f, "{error}"),
            CliError::Store(error) => write!(f, "{error}"),
            CliError::Usage(message) => write!(f, "{message}"),
            CliError::NotOpen(path) => write!(
                f,
                "not inside an open CodeFire branch directory: {}",
                path.display()
            ),
            CliError::InvalidMarker(message) => {
                write!(f, "invalid open directory marker: {message}")
            }
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::Json(error)
    }
}

impl From<codefire_store::StoreError> for CliError {
    fn from(error: codefire_store::StoreError) -> Self {
        CliError::Store(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Status {
    branch: String,
    state: String,
    base: String,
    open_fires: usize,
}

fn print_status(status: &Status) {
    println!("Branch: {}", status.branch);
    println!("State: {}", status.state);
    println!("Base: {}", status.base);
    println!("Open fires: {}", status.open_fires);
}

fn read_status(start: &Path) -> Result<Status, CliError> {
    let marker_path =
        find_open_marker(start)?.ok_or_else(|| CliError::NotOpen(start.to_path_buf()))?;
    let marker: Value = read_json(&marker_path)?;
    let open_dir = marker_path
        .parent()
        .ok_or_else(|| CliError::InvalidMarker("marker has no parent directory".to_string()))?;
    let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repo_root = repo_dir
        .parent()
        .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()))?;
    let branch = required_string(&marker, &["branch", "name"])?;
    let registry_path = repo_root
        .join(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(&branch)));
    let registry: Value = read_json(&registry_path)?;
    let registry_open_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
    if registry_open_path != open_dir {
        return Err(CliError::InvalidMarker(
            "opened_path does not match registry".to_string(),
        ));
    }
    let open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    let registry_open_instance_id = required_string(&registry, &["open", "open_instance_id"])?;
    if open_instance_id != registry_open_instance_id {
        return Err(CliError::InvalidMarker(
            "open_instance_id does not match registry".to_string(),
        ));
    }

    let base = required_string(&registry, &["open", "current_base_commit"])?;
    let objects_root = repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects_root, &base)?;
    let state = required_string(&registry, &["state", "last_known"])?;
    let active_state_path =
        PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let fires_path = active_state_path.join("fires.json");
    let open_fires = if fires_path.exists() {
        let fires: Value = read_json(&fires_path)?;
        fires
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.get("status").and_then(Value::as_str) == Some("open"))
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    Ok(Status {
        branch,
        state,
        base,
        open_fires,
    })
}

fn find_open_marker(start: &Path) -> Result<Option<PathBuf>, CliError> {
    let mut current = start.canonicalize()?;
    loop {
        let marker = current.join(".codefire-open");
        if marker.exists() {
            return Ok(Some(marker));
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}

fn read_json(path: &Path) -> Result<Value, CliError> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn required_string(value: &Value, path: &[&str]) -> Result<String, CliError> {
    let mut current = value;
    for key in path {
        current = current
            .get(*key)
            .ok_or_else(|| CliError::InvalidMarker(format!("missing {}", path.join("."))))?;
    }
    current
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| CliError::InvalidMarker(format!("{} must be a string", path.join("."))))
}

fn ref_file_name(name: &str) -> String {
    let mut encoded = String::with_capacity(name.len());
    for byte in name.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};
    use tempfile::tempdir;

    #[test]
    fn status_reads_python_compatible_open_directory() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path();
        let open_dir = repo_root.join("main");
        let cf = repo_root.join(".codefire");
        fs::create_dir_all(&open_dir).unwrap();
        fs::create_dir_all(cf.join("opened")).unwrap();
        fs::create_dir_all(cf.join("active").join("open_123")).unwrap();

        let commit_id = write_valid_commit(&cf.join("objects"));
        fs::write(
            open_dir.join(".codefire-open"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "repository": {"path": cf, "repository_id": "repo_123"},
                "branch": {"name": "main", "opened_from_commit": commit_id},
                "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "opened_path": open_dir}
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            repo_root.join(".codefire").join("opened").join("main.json"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "branch": {"name": "main"},
                "open": {
                    "path": open_dir,
                    "open_instance_id": "open_123",
                    "opened_from_commit": commit_id,
                    "current_base_commit": commit_id,
                    "active_state_path": repo_root.join(".codefire").join("active").join("open_123")
                },
                "state": {"last_known": "open-consistent"}
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            repo_root
                .join(".codefire")
                .join("active")
                .join("open_123")
                .join("fires.json"),
            serde_json::to_string_pretty(&json!([
                {"display_id": "FIRE-001", "status": "open"},
                {"display_id": "FIRE-002", "status": "extinguished"}
            ]))
            .unwrap(),
        )
        .unwrap();

        let status = read_status(&open_dir).unwrap();

        assert_eq!(
            status,
            Status {
                branch: "main".to_string(),
                state: "open-consistent".to_string(),
                base: commit_id,
                open_fires: 1
            }
        );
    }

    #[test]
    fn ref_file_name_matches_python_quote_safe_empty() {
        assert_eq!(
            ref_file_name("feature/a b%雪"),
            "feature%2Fa%20b%25%E9%9B%AA"
        );
    }

    fn write_valid_commit(objects: &Path) -> String {
        let roots = write_required_roots(objects);
        let certificate = consistent_certificate();
        let commit = codefire_store::commit_payload(vec![], roots, certificate);
        codefire_store::store_object(objects, "commit", commit).unwrap()
    }

    fn write_required_roots(objects: &Path) -> Map<String, Value> {
        let content_manifest = codefire_store::store_object(
            objects,
            "content_manifest",
            json!({"type": "content_manifest", "version": 1, "entries": []}),
        )
        .unwrap();
        let atom_index = codefire_store::store_object(
            objects,
            "atom_index",
            json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
        )
        .unwrap();
        let trace_graph = codefire_store::store_object(
            objects,
            "trace_graph",
            json!({"type": "trace_graph", "version": 1, "links": []}),
        )
        .unwrap();
        let fire_delta = codefire_store::store_object(
            objects,
            "fire_ledger",
            json!({"type": "fire_ledger", "version": 1, "fires": []}),
        )
        .unwrap();
        let verification = codefire_store::store_object(
            objects,
            "verification",
            json!({"type": "verification", "version": 1, "checks": []}),
        )
        .unwrap();
        let policy = codefire_store::store_object(
            objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {}}),
        )
        .unwrap();

        let mut roots = Map::new();
        roots.insert(
            "content_manifest".to_string(),
            Value::String(content_manifest),
        );
        roots.insert("atom_index".to_string(), Value::String(atom_index));
        roots.insert("trace_graph".to_string(), Value::String(trace_graph));
        roots.insert("fire_delta".to_string(), Value::String(fire_delta));
        roots.insert("verification".to_string(), Value::String(verification));
        roots.insert("policy".to_string(), Value::String(policy));
        roots
    }

    fn consistent_certificate() -> Map<String, Value> {
        let mut certificate = Map::new();
        certificate.insert(
            "result".to_string(),
            Value::String("consistent".to_string()),
        );
        certificate.insert("open_required_fires".to_string(), Value::Number(0.into()));
        certificate.insert("failed_checks".to_string(), Value::Number(0.into()));
        certificate.insert(
            "missing_required_links".to_string(),
            Value::Number(0.into()),
        );
        certificate.insert("stale_resolutions".to_string(), Value::Number(0.into()));
        certificate.insert("duplicate_atom_ids".to_string(), Value::Number(0.into()));
        certificate
    }
}
