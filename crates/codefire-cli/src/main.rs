use serde_json::{json, Value};
use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    match args.first().map(String::as_str) {
        Some("atom-index") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let index = codefire_core::build_atom_index(&start)?;
            println!("{}", serde_json::to_string_pretty(&index)?);
            Ok(())
        }
        Some("trace-graph") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let trace_graph = codefire_core::current_trace_graph(&start)?;
            println!("{}", serde_json::to_string_pretty(&trace_graph)?);
            Ok(())
        }
        Some("missing-links") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let missing = codefire_core::current_required_link_missing(&start)?;
            println!("{}", serde_json::to_string_pretty(&missing)?);
            Ok(())
        }
        Some("init") => {
            let options = parse_init_args(&args[1..])?;
            let result = init_repo(&options.path, options.force)?;
            println!(
                "initialized CodeFire repository: {}",
                result.repo_root.display()
            );
            println!("main: {}", result.main_commit);
            Ok(())
        }
        Some("open") => {
            let options = parse_open_args(&args[1..])?;
            let result = open_branch(&options)?;
            println!("opened {}: {}", result.branch, result.open_dir.display());
            Ok(())
        }
        Some("branch") => match args.get(1).map(String::as_str) {
            Some("list") => {
                let start = args
                    .get(2)
                    .map(PathBuf::from)
                    .unwrap_or(env::current_dir()?);
                let branches = list_branches(&start)?;
                print_branches(&branches);
                Ok(())
            }
            Some(command) => Err(CliError::Usage(format!(
                "unsupported branch command: {command}"
            ))),
            None => Err(CliError::Usage(
                "missing branch command; expected: list".to_string(),
            )),
        },
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
    Core(codefire_core::CoreError),
    Store(codefire_store::StoreError),
    Usage(String),
    NotOpen(PathBuf),
    InvalidMarker(String),
    InvalidRepository(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(error) => write!(f, "{error}"),
            CliError::Json(error) => write!(f, "{error}"),
            CliError::Core(error) => write!(f, "{error}"),
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
            CliError::InvalidRepository(message) => {
                write!(f, "invalid CodeFire repository: {message}")
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

impl From<codefire_core::CoreError> for CliError {
    fn from(error: codefire_core::CoreError) -> Self {
        CliError::Core(error)
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

#[derive(Debug, PartialEq, Eq)]
struct Branch {
    name: String,
    head: String,
    state: String,
}

#[derive(Debug)]
struct InitOptions {
    path: PathBuf,
    force: bool,
}

#[derive(Debug)]
struct InitResult {
    repo_root: PathBuf,
    main_commit: String,
}

#[derive(Debug)]
struct OpenOptions {
    branch: String,
    path: PathBuf,
}

#[derive(Debug)]
struct OpenResult {
    branch: String,
    open_dir: PathBuf,
}

fn print_status(status: &Status) {
    println!("Branch: {}", status.branch);
    println!("State: {}", status.state);
    println!("Base: {}", status.base);
    println!("Open fires: {}", status.open_fires);
}

fn print_branches(branches: &[Branch]) {
    for branch in branches {
        println!("{}\t{}\t{}", branch.name, branch.head, branch.state);
    }
}

fn parse_init_args(args: &[String]) -> Result<InitOptions, CliError> {
    let mut path = None;
    let mut force = false;
    for arg in args {
        if arg == "--force" {
            force = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err(CliError::Usage(format!("unexpected init argument: {arg}")));
        }
    }
    Ok(InitOptions {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        force,
    })
}

fn parse_open_args(args: &[String]) -> Result<OpenOptions, CliError> {
    match args {
        [branch, path] => Ok(OpenOptions {
            branch: branch.to_string(),
            path: PathBuf::from(path),
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs open <branch> <path>".to_string(),
        )),
    }
}

fn init_repo(path: &Path, force: bool) -> Result<InitResult, CliError> {
    let repo_root = absolute_path(path)?;
    let cf = repo_root.join(".codefire");
    if cf.exists() && !force {
        return Err(CliError::Usage(".codefire already exists".to_string()));
    }

    ensure_repo_layout(&repo_root)?;
    let now = now_iso_utc();
    write_json_atomic(
        &cf.join("repo.json"),
        &json!({
            "version": 1,
            "repository_id": stable_repo_id(&repo_root),
            "created_at": now,
        }),
    )?;

    let objects = cf.join("objects");
    let roots = initial_roots(&objects, &now)?;
    let commit_payload = json!({
        "type": "commit",
        "version": 1,
        "parents": [],
        "message": "Initial empty CodeFire repository",
        "roots": roots,
        "certificate": {
            "result": "consistent",
            "open_required_fires": 0,
            "failed_checks": 0,
            "missing_required_links": 0,
            "stale_resolutions": 0,
            "duplicate_atom_ids": 0,
        },
        "changed_atoms": [],
        "extinguished_fires": [],
        "created_at": now,
    });
    let commit_id = codefire_store::store_object(&objects, "commit", commit_payload)?;
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": "main",
        "head": commit_id,
        "state": "closed",
        "created_at": now,
    });
    write_json_atomic(
        &cf.join("branches")
            .join(format!("{}.json", ref_file_name("main"))),
        &branch,
    )?;
    codefire_store::store_object(&objects, "branch", branch)?;

    Ok(InitResult {
        repo_root,
        main_commit: commit_id,
    })
}

fn open_branch(options: &OpenOptions) -> Result<OpenResult, CliError> {
    open_branch_from(&env::current_dir()?, options)
}

fn open_branch_from(start: &Path, options: &OpenOptions) -> Result<OpenResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = RepoLock::acquire(&repo_root)?;
    let cf = repo_root.join(".codefire");
    let objects = cf.join("objects");
    let mut branch = load_branch_record(&repo_root, &options.branch)?;
    let branch_head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &branch_head)?;

    let target = absolute_path(&options.path)?;
    if target.exists() && !is_empty_dir(&target)? {
        return Err(CliError::Usage(format!(
            "target path must not exist or must be empty: {}",
            target.display()
        )));
    }
    let registry_path = opened_registry_path(&repo_root, &options.branch);
    if registry_path.exists() {
        return Err(CliError::Usage(format!(
            "branch is already open: {}",
            options.branch
        )));
    }

    fs::create_dir_all(&target)?;
    materialize_commit(&objects, &branch_head, &target)?;
    let opened_at = now_iso_utc();
    let open_instance_id = format!(
        "open_{:012x}",
        stable_hash_48(format!("{}:{}:{opened_at}", options.branch, target.display()).as_bytes())
    );
    let repo_json = read_json(&cf.join("repo.json"))?;
    let repository_id = required_string(&repo_json, &["repository_id"])?;
    write_json_atomic(
        &target.join(".codefire-open"),
        &json!({
            "version": 1,
            "repository": {"path": cf, "repository_id": repository_id},
            "branch": {"name": options.branch, "opened_from_commit": branch_head},
            "open": {"open_instance_id": open_instance_id, "opened_path": target, "opened_at": opened_at},
        }),
    )?;
    let active_path = cf.join("active").join(&open_instance_id);
    write_json_atomic(
        &registry_path,
        &json!({
            "version": 1,
            "branch": {"name": options.branch},
            "open": {
                "open_instance_id": open_instance_id,
                "path": target,
                "opened_from_commit": branch_head,
                "current_base_commit": branch_head,
                "active_state_path": active_path,
            },
            "state": {"last_known": "open-clean"},
        }),
    )?;
    fs::create_dir_all(&active_path)?;
    write_json_atomic(
        &active_path.join("state.json"),
        &json!({"state": "open-clean"}),
    )?;
    branch["state"] = Value::String("open-clean".to_string());
    save_branch_record(&repo_root, &branch)?;

    Ok(OpenResult {
        branch: options.branch.clone(),
        open_dir: target,
    })
}

fn initial_roots(objects: &Path, now: &str) -> Result<Value, CliError> {
    let content_manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": []}),
    )?;
    let atom_index = codefire_store::store_object(
        objects,
        "atom_index",
        json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
    )?;
    let trace_graph = codefire_store::store_object(
        objects,
        "trace_graph",
        json!({"type": "trace_graph", "version": 1, "links": []}),
    )?;
    let fire_delta = codefire_store::store_object(
        objects,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": []}),
    )?;
    let verification = codefire_store::store_object(
        objects,
        "verification",
        json!({
            "type": "verification",
            "version": 1,
            "result": "passed",
            "open_required_fires": 0,
            "failed_checks": [],
            "missing_required_links": [],
            "stale_resolutions": [],
            "duplicate_atom_ids": [],
            "verified_at": now,
        }),
    )?;
    let policy = codefire_store::store_object(
        objects,
        "policy",
        json!({"type": "policy", "version": 1, "policy": {}}),
    )?;
    Ok(json!({
        "content_manifest": content_manifest,
        "atom_index": atom_index,
        "trace_graph": trace_graph,
        "fire_delta": fire_delta,
        "verification": verification,
        "policy": policy,
    }))
}

fn ensure_repo_layout(repo_root: &Path) -> Result<(), CliError> {
    let cf = repo_root.join(".codefire");
    for path in [
        cf.clone(),
        cf.join("objects"),
        cf.join("branches"),
        cf.join("opened"),
        cf.join("active"),
        cf.join("cache"),
        cf.join("locks"),
        cf.join("remotes"),
    ] {
        fs::create_dir_all(path)?;
    }
    for subdir in [
        "blobs",
        "content_manifests",
        "atom_indexes",
        "trace_graphs",
        "fire_ledgers",
        "resolution_ledgers",
        "verifications",
        "policies",
        "commits",
        "branches",
    ] {
        fs::create_dir_all(cf.join("objects").join(subdir))?;
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, CliError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn stable_repo_id(repo_root: &Path) -> String {
    format!(
        "repo_{:012x}",
        stable_hash_48(repo_root.to_string_lossy().as_bytes())
    )
}

fn stable_hash_48(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash & 0x0000_ffff_ffff_ffff
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), CliError> {
    let parent = path
        .parent()
        .ok_or_else(|| CliError::Usage(format!("path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    let temp_path = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ));
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
        file.write_all(b"\n")?;
    }
    fs::rename(temp_path, path)?;
    Ok(())
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
    let registry_path = opened_registry_path(repo_root, &branch);
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

fn list_branches(start: &Path) -> Result<Vec<Branch>, CliError> {
    let repo_root = find_repo_root(start)?;
    let branches_root = repo_root.join(".codefire").join("branches");
    let objects_root = repo_root.join(".codefire").join("objects");
    let mut branch_paths = Vec::new();
    for entry in fs::read_dir(branches_root)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            branch_paths.push(path);
        }
    }
    branch_paths.sort();

    let mut branches = Vec::with_capacity(branch_paths.len());
    for path in branch_paths {
        let record = read_json(&path)?;
        let name = required_string(&record, &["name"])?;
        let head = required_string(&record, &["head"])?;
        codefire_store::validate_sealed_commit(&objects_root, &head)?;
        let state = record
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("closed")
            .to_string();
        branches.push(Branch { name, head, state });
    }
    Ok(branches)
}

fn load_branch_record(repo_root: &Path, branch: &str) -> Result<Value, CliError> {
    let path = branch_record_path(repo_root, branch);
    if !path.exists() {
        return Err(CliError::Usage(format!("unknown branch: {branch}")));
    }
    read_json(&path)
}

fn save_branch_record(repo_root: &Path, branch: &Value) -> Result<(), CliError> {
    let name = required_string(branch, &["name"])?;
    write_json_atomic(&branch_record_path(repo_root, &name), branch)?;
    codefire_store::store_object(
        &repo_root.join(".codefire").join("objects"),
        "branch",
        branch.clone(),
    )?;
    Ok(())
}

fn branch_record_path(repo_root: &Path, branch: &str) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("branches")
        .join(format!("{}.json", ref_file_name(branch)))
}

fn opened_registry_path(repo_root: &Path, branch: &str) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(branch)))
}

fn materialize_commit(objects: &Path, commit_id: &str, target: &Path) -> Result<(), CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let manifest_id = required_string(&commit, &["roots", "content_manifest"])?;
    let manifest = codefire_store::read_object(objects, &manifest_id)?;
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
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
        let output_path = safe_manifest_output_path(target, &rel_path)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, decode_base64(&content)?)?;
    }
    Ok(())
}

fn safe_manifest_output_path(target: &Path, rel_path: &str) -> Result<PathBuf, CliError> {
    let path = Path::new(rel_path);
    if path.is_absolute() {
        return Err(CliError::InvalidRepository(format!(
            "manifest path must be relative: {rel_path}"
        )));
    }
    let mut output = target.to_path_buf();
    for component in path.components() {
        match component {
            Component::Normal(part) => output.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CliError::InvalidRepository(format!(
                    "manifest path escapes target: {rel_path}"
                )));
            }
        }
    }
    Ok(output)
}

fn decode_base64(input: &str) -> Result<Vec<u8>, CliError> {
    let bytes: Vec<u8> = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if bytes.len() % 4 != 0 {
        return Err(CliError::InvalidRepository(
            "base64 content length is invalid".to_string(),
        ));
    }
    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for (chunk_index, chunk) in bytes.chunks(4).enumerate() {
        let last = chunk_index + 1 == bytes.len() / 4;
        let a = base64_value(chunk[0])?;
        let b = base64_value(chunk[1])?;
        let c_padding = chunk[2] == b'=';
        let d_padding = chunk[3] == b'=';
        if c_padding && !d_padding {
            return Err(CliError::InvalidRepository(
                "base64 padding is invalid".to_string(),
            ));
        }
        if (c_padding || d_padding) && !last {
            return Err(CliError::InvalidRepository(
                "base64 padding before final chunk".to_string(),
            ));
        }
        let c = if c_padding {
            0
        } else {
            base64_value(chunk[2])?
        };
        let d = if d_padding {
            0
        } else {
            base64_value(chunk[3])?
        };
        output.push((a << 2) | (b >> 4));
        if !c_padding {
            output.push(((b & 0x0f) << 4) | (c >> 2));
        }
        if !d_padding {
            output.push(((c & 0x03) << 6) | d);
        }
    }
    Ok(output)
}

fn base64_value(byte: u8) -> Result<u8, CliError> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(CliError::InvalidRepository(
            "base64 content contains an invalid character".to_string(),
        )),
    }
}

fn is_empty_dir(path: &Path) -> Result<bool, CliError> {
    Ok(path.is_dir() && fs::read_dir(path)?.next().is_none())
}

struct RepoLock {
    path: PathBuf,
}

impl RepoLock {
    fn acquire(repo_root: &Path) -> Result<Self, CliError> {
        let path = repo_root.join(".codefire").join("locks").join("repo.lock");
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => Ok(Self { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(CliError::Usage("CodeFire repository is locked".to_string()))
            }
            Err(error) => Err(CliError::Io(error)),
        }
    }
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn find_repo_root(start: &Path) -> Result<PathBuf, CliError> {
    let mut current = start.canonicalize()?;
    if current.is_file() {
        current.pop();
    }
    let mut candidate = Some(current.as_path());
    while let Some(path) = candidate {
        if path.join(".codefire").is_dir() {
            return Ok(path.to_path_buf());
        }
        candidate = path.parent();
    }
    if let Some(marker_path) = find_open_marker(&current)? {
        let marker = read_json(&marker_path)?;
        let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
        return repo_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()));
    }
    Err(CliError::Usage(
        "not inside a CodeFire repository; run 'codefire init' first".to_string(),
    ))
}

fn find_open_marker(start: &Path) -> Result<Option<PathBuf>, CliError> {
    let mut current = start.canonicalize()?;
    if current.is_file() {
        current.pop();
    }
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

fn now_iso_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_seconds_utc(seconds)
}

fn format_unix_seconds_utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = second_of_day / 3_600;
    let minute = second_of_day % 3_600 / 60;
    let second = second_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let days = days_since_unix_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 }.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_part + 2).div_euclid(5) + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    let adjusted_year = year + if month <= 2 { 1 } else { 0 };
    (adjusted_year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};
    use tempfile::tempdir;

    #[test]
    fn init_creates_python_compatible_repo_layout() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");

        let result = init_repo(&repo_root, false).unwrap();
        let branches = list_branches(&repo_root).unwrap();

        assert_eq!(result.repo_root, repo_root);
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "main");
        assert_eq!(branches[0].head, result.main_commit);
        assert_eq!(branches[0].state, "closed");
        codefire_store::validate_sealed_commit(
            &repo_root.join(".codefire").join("objects"),
            &result.main_commit,
        )
        .unwrap();
        assert!(init_repo(&repo_root, false).is_err());
        init_repo(&repo_root, true).unwrap();
    }

    #[test]
    fn open_materializes_manifest_and_updates_registry() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let open_dir = temp.path().join("main-open");
        init_repo(&repo_root, false).unwrap();
        let objects = repo_root.join(".codefire").join("objects");
        let commit_id = write_file_commit(&objects, "docs/readme.md", "hello\n");
        let branch = json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": commit_id,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        });
        save_branch_record(&repo_root, &branch).unwrap();

        let result = open_branch_from(
            &repo_root,
            &OpenOptions {
                branch: "main".to_string(),
                path: open_dir.clone(),
            },
        )
        .unwrap();
        let status = read_status(&open_dir).unwrap();
        let branches = list_branches(&open_dir).unwrap();

        assert_eq!(result.open_dir, open_dir);
        assert_eq!(
            fs::read_to_string(result.open_dir.join("docs").join("readme.md")).unwrap(),
            "hello\n"
        );
        assert_eq!(
            status,
            Status {
                branch: "main".to_string(),
                state: "open-clean".to_string(),
                base: commit_id.clone(),
                open_fires: 0,
            }
        );
        assert_eq!(branches[0].state, "open-clean");
    }

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
    fn branch_list_reads_repo_from_open_marker_and_validates_heads() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let open_dir = temp.path().join("worktree");
        let cf = repo_root.join(".codefire");
        fs::create_dir_all(cf.join("branches")).unwrap();
        fs::create_dir_all(&open_dir).unwrap();

        let commit_id = write_valid_commit(&cf.join("objects"));
        let branch_name = "feature/a b%雪";
        fs::write(
            cf.join("branches")
                .join(format!("{}.json", ref_file_name(branch_name))),
            serde_json::to_string_pretty(&json!({
                "type": "branch",
                "version": 1,
                "name": branch_name,
                "head": commit_id,
                "state": "open-clean",
                "created_at": "2026-06-04T00:00:00Z"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            open_dir.join(".codefire-open"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "repository": {"path": cf, "repository_id": "repo_123"},
                "branch": {"name": branch_name, "opened_from_commit": commit_id},
                "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "opened_path": open_dir}
            }))
            .unwrap(),
        )
        .unwrap();

        let branches = list_branches(&open_dir).unwrap();

        assert_eq!(
            branches,
            vec![Branch {
                name: branch_name.to_string(),
                head: commit_id,
                state: "open-clean".to_string()
            }]
        );
    }

    #[test]
    fn ref_file_name_matches_python_quote_safe_empty() {
        assert_eq!(
            ref_file_name("feature/a b%雪"),
            "feature%2Fa%20b%25%E9%9B%AA"
        );
    }

    #[test]
    fn unix_timestamp_format_uses_utc_iso_seconds() {
        assert_eq!(format_unix_seconds_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix_seconds_utc(951_782_400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn base64_decoder_handles_padding_and_rejects_invalid_input() {
        assert_eq!(decode_base64("aGVsbG8K").unwrap(), b"hello\n");
        assert_eq!(decode_base64("Zg==").unwrap(), b"f");
        assert!(decode_base64("Z===").is_err());
    }

    fn write_valid_commit(objects: &Path) -> String {
        let roots = write_required_roots(objects);
        let certificate = consistent_certificate();
        let commit = codefire_store::commit_payload(vec![], roots, certificate);
        codefire_store::store_object(objects, "commit", commit).unwrap()
    }

    fn write_file_commit(objects: &Path, path: &str, contents: &str) -> String {
        let blob = codefire_store::store_object(
            objects,
            "blob",
            json!({
                "type": "blob",
                "version": 1,
                "encoding": "base64",
                "content": encode_base64(contents.as_bytes()),
            }),
        )
        .unwrap();
        let manifest = codefire_store::store_object(
            objects,
            "content_manifest",
            json!({
                "type": "content_manifest",
                "version": 1,
                "entries": [{"path": path, "kind": "file", "mode": "100644", "blob": blob}],
            }),
        )
        .unwrap();
        let mut roots = write_required_roots(objects);
        roots.insert("content_manifest".to_string(), Value::String(manifest));
        codefire_store::store_object(
            objects,
            "commit",
            codefire_store::commit_payload(vec![], roots, consistent_certificate()),
        )
        .unwrap()
    }

    fn encode_base64(bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let a = chunk[0];
            let b = *chunk.get(1).unwrap_or(&0);
            let c = *chunk.get(2).unwrap_or(&0);
            encoded.push(ALPHABET[(a >> 2) as usize] as char);
            encoded.push(ALPHABET[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
            if chunk.len() >= 2 {
                encoded.push(ALPHABET[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
            } else {
                encoded.push('=');
            }
            if chunk.len() == 3 {
                encoded.push(ALPHABET[(c & 0x3f) as usize] as char);
            } else {
                encoded.push('=');
            }
        }
        encoded
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
