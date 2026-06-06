use super::http_tls;
use super::remote::*;
use super::*;
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const MAX_HTTP_BODY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct CfHttpUrl {
    pub(crate) endpoint: String,
    pub(crate) org: String,
    pub(crate) app: String,
    pub(crate) branch: String,
}

impl CfHttpUrl {
    pub(crate) fn endpoint_scheme(&self) -> &'static str {
        endpoint_scheme(&self.endpoint)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CfHttpProjectUrl {
    pub(crate) endpoint: String,
    pub(crate) org: String,
    pub(crate) app: String,
}

impl CfHttpProjectUrl {
    pub(crate) fn endpoint_scheme(&self) -> &'static str {
        endpoint_scheme(&self.endpoint)
    }
}

fn endpoint_scheme(endpoint: &str) -> &'static str {
    if endpoint.starts_with("https://") {
        "https"
    } else {
        "http"
    }
}

pub(crate) fn parse_cf_http_url(url: &str) -> Result<CfHttpUrl, CliError> {
    let (endpoint, parts) = parse_cf_http_parts(url, 3)?;
    Ok(CfHttpUrl {
        endpoint,
        org: parts[0].clone(),
        app: parts[1].clone(),
        branch: parts[2].clone(),
    })
}

pub(crate) fn parse_cf_http_project_url(url: &str) -> Result<CfHttpProjectUrl, CliError> {
    let (endpoint, parts) = parse_cf_http_parts(url, 2)?;
    Ok(CfHttpProjectUrl {
        endpoint,
        org: parts[0].clone(),
        app: parts[1].clone(),
    })
}

fn parse_cf_http_parts(url: &str, exact_parts: usize) -> Result<(String, Vec<String>), CliError> {
    let (scheme, raw) = if let Some(raw) = url.strip_prefix("cf+http://") {
        ("http", raw)
    } else if let Some(raw) = url.strip_prefix("cf+https://") {
        ("https", raw)
    } else {
        return Err(CliError::Usage(format!(
            "not a CodeFire HTTP remote URL: {url}"
        )));
    };
    let (netloc, path) = raw
        .split_once('/')
        .ok_or_else(|| CliError::Usage(format!("not a CodeFire HTTP remote URL: {url}")))?;
    if netloc.is_empty() {
        return Err(CliError::Usage(format!(
            "not a CodeFire HTTP remote URL: {url}"
        )));
    }
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(percent_decode)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != exact_parts {
        return Err(CliError::Usage(if exact_parts == 3 {
            "HTTP remote URL must be cf+http://<host>/<org>/<app>/<branch> or cf+https://<host>/<org>/<app>/<branch>".to_string()
        } else {
            "HTTP remote project URL must be cf+http://<host>/<org>/<app> or cf+https://<host>/<org>/<app>".to_string()
        }));
    }
    Ok((format!("{scheme}://{netloc}"), parts))
}

pub(crate) fn is_cf_http_url(url: &str) -> bool {
    url.starts_with("cf+http://") || url.starts_with("cf+https://")
}

pub(crate) fn http_remote_path(endpoint: &str, org: &str, app: &str, parts: &[&str]) -> String {
    let encoded = [org, app]
        .into_iter()
        .chain(parts.iter().copied())
        .map(ref_file_name)
        .collect::<Vec<_>>()
        .join("/");
    format!("{endpoint}/v1/projects/{encoded}")
}

pub(crate) fn http_json(
    method: &str,
    url: &str,
    payload: Option<Value>,
) -> Result<Value, CliError> {
    let endpoint = parse_http_url(url)?;
    let mut stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))?;
    if endpoint.scheme == "https" {
        let mut tls_stream = http_tls::connect_client(stream, &endpoint.host)?;
        return http_json_stream(method, &endpoint, payload, &mut tls_stream);
    }
    http_json_stream(method, &endpoint, payload, &mut stream)
}

fn http_json_stream<S: Read + Write>(
    method: &str,
    endpoint: &HttpEndpoint,
    payload: Option<Value>,
    stream: &mut S,
) -> Result<Value, CliError> {
    let request_body = payload
        .map(|value| serde_json::to_vec(&value))
        .transpose()?
        .unwrap_or_default();
    let request = format!(
        "{method} {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        endpoint.path,
        endpoint.host,
        endpoint.port,
        request_body.len()
    );
    stream.write_all(request.as_bytes())?;
    stream.write_all(&request_body)?;
    stream.flush()?;
    let (head, response_body) = read_http_response(stream)?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| CliError::InvalidRepository("invalid HTTP status line".to_string()))?;
    let value = if response_body.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&response_body)?
    };
    if status >= 400 {
        let detail = value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("HTTP remote error");
        if value.get("exit_code").and_then(Value::as_i64)
            == Some(ExitCode::IdempotencyConflict.code() as i64)
        {
            return Err(CliError::IdempotencyConflict(format!(
                "HTTP remote error: {detail}"
            )));
        }
        if value.get("exit_code").and_then(Value::as_i64)
            == Some(ExitCode::AuthenticationOrSignatureFailure.code() as i64)
        {
            return Err(CliError::AuthenticationOrSignature(format!(
                "HTTP remote error: {detail}"
            )));
        }
        return Err(CliError::Usage(format!("HTTP remote error: {detail}")));
    }
    Ok(value)
}

fn read_http_response<S: Read + Write>(stream: &mut S) -> Result<(String, Vec<u8>), CliError> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut temp)?;
        if read == 0 {
            return Err(CliError::InvalidRepository(
                "incomplete HTTP response".to_string(),
            ));
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(position) = find_header_end(&buffer) {
            break position;
        }
        if buffer.len() > 64 * 1024 {
            return Err(CliError::Usage(
                "HTTP response headers too large".to_string(),
            ));
        }
    };
    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let content_length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let read = stream.read(&mut temp)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
    }
    if buffer.len() < body_start + content_length {
        return Err(CliError::InvalidRepository(
            "incomplete HTTP response body".to_string(),
        ));
    }
    Ok((
        head,
        buffer[body_start..body_start + content_length].to_vec(),
    ))
}

#[derive(Debug)]
struct HttpEndpoint {
    scheme: String,
    host: String,
    port: u16,
    path: String,
}

fn parse_http_url(url: &str) -> Result<HttpEndpoint, CliError> {
    let (scheme, raw) = if let Some(raw) = url.strip_prefix("http://") {
        ("http", raw)
    } else if let Some(raw) = url.strip_prefix("https://") {
        ("https", raw)
    } else {
        return Err(CliError::Usage(
            "only cf+http and cf+https transports are supported".to_string(),
        ));
    };
    let (authority, path) = raw.split_once('/').unwrap_or((raw, ""));
    let (host, port) = if let Some((host, port)) = authority.rsplit_once(':') {
        (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| CliError::Usage(format!("invalid HTTP port: {port}")))?,
        )
    } else {
        let port = if scheme == "http" { 80 } else { 443 };
        (authority.to_string(), port)
    };
    Ok(HttpEndpoint {
        scheme: scheme.to_string(),
        host,
        port,
        path: format!("/{path}"),
    })
}

pub(crate) fn serve_http(options: &ServeOptions) -> Result<(), CliError> {
    let storage_root = absolute_path(&options.storage_root)?;
    fs::create_dir_all(&storage_root)?;
    let listener = TcpListener::bind((options.host.as_str(), options.port))?;
    let address = listener.local_addr()?;
    let tls_config = match (&options.tls_cert, &options.tls_key) {
        (Some(cert), Some(key)) => Some(http_tls::server_config(cert, key)?),
        _ => None,
    };
    let scheme = if tls_config.is_some() {
        "cf+https"
    } else {
        "cf+http"
    };
    println!(
        "serving CodeFire HTTP remote: {scheme}://{}:{}",
        address.ip(),
        address.port()
    );
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Some(config) = &tls_config {
                    match http_tls::accept_server(stream, config.clone()) {
                        Ok(mut tls_stream) => {
                            if let Err(error) = handle_http_stream(&storage_root, &mut tls_stream) {
                                eprintln!("HTTP request failed: {error}");
                            }
                        }
                        Err(error) => {
                            eprintln!("TLS handshake failed: {error}");
                        }
                    }
                } else if let Err(error) = handle_http_stream(&storage_root, &mut stream) {
                    eprintln!("HTTP request failed: {error}");
                }
            }
            Err(error) => return Err(CliError::Io(error)),
        }
    }
    Ok(())
}

fn handle_http_stream<S: Read + Write>(
    storage_root: &Path,
    stream: &mut S,
) -> Result<(), CliError> {
    let request = read_http_request(stream)?;
    let (status, payload) = match dispatch_http(storage_root, &request) {
        Ok(response) => response,
        Err(error) => (
            if matches!(error, CliError::IdempotencyConflict(_)) {
                409
            } else if matches!(error, CliError::HttpPayloadTooLarge(_)) {
                413
            } else {
                400
            },
            json!({"error": error.to_string(), "exit_code": error.exit_code()}),
        ),
    };
    write_http_json(stream, status, &payload)
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    body: Value,
}

fn read_http_request<S: Read + Write>(stream: &mut S) -> Result<HttpRequest, CliError> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut temp)?;
        if read == 0 {
            return Err(CliError::InvalidRepository(
                "incomplete HTTP request".to_string(),
            ));
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(position) = find_header_end(&buffer) {
            break position;
        }
        if buffer.len() > 64 * 1024 {
            return Err(CliError::Usage(
                "HTTP request headers too large".to_string(),
            ));
        }
    };
    let header = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = header.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| CliError::InvalidRepository("missing HTTP request line".to_string()))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| CliError::InvalidRepository("missing HTTP method".to_string()))?
        .to_string();
    let path = request_parts
        .next()
        .ok_or_else(|| CliError::InvalidRepository("missing HTTP path".to_string()))?
        .to_string();
    let content_length = lines
        .filter_map(|line| line.split_once(':'))
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(CliError::HttpPayloadTooLarge(format!(
            "HTTP request body too large: {content_length} bytes exceeds {MAX_HTTP_BODY_BYTES} bytes"
        )));
    }
    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let read = stream.read(&mut temp)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
    }
    if buffer.len() < body_start + content_length {
        return Err(CliError::InvalidRepository(
            "incomplete HTTP request body".to_string(),
        ));
    }
    let body_bytes = &buffer[body_start..body_start + content_length];
    let body = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(body_bytes)?
    };
    Ok(HttpRequest { method, path, body })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn write_http_json<S: Read + Write>(
    stream: &mut S,
    status: u16,
    payload: &Value,
) -> Result<(), CliError> {
    let body = serde_json::to_vec(payload)?;
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Payload Too Large",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}

fn dispatch_http(storage_root: &Path, request: &HttpRequest) -> Result<(u16, Value), CliError> {
    let (org, app, rest) = http_project_parts(&request.path)?;
    let rest = rest.iter().map(String::as_str).collect::<Vec<_>>();
    match (request.method.as_str(), rest.as_slice()) {
        ("GET", ["branches"]) => handle_http_list(storage_root, &org, &app),
        ("GET", ["branches", branch, "bundle"]) => {
            handle_http_bundle(storage_root, &org, &app, branch)
        }
        ("GET", ["merge-requests"]) => handle_http_request_list(storage_root, &org, &app),
        ("POST", ["branches", branch, "upload"]) => {
            handle_http_upload(storage_root, &org, &app, branch, &request.body)
        }
        ("POST", ["merge-requests"]) => {
            handle_http_request_merge(storage_root, &org, &app, &request.body)
        }
        ("POST", ["merge-requests", mr_id, "review"]) => {
            handle_http_request_review(storage_root, &org, &app, mr_id, &request.body)
        }
        ("POST", ["merge-requests", mr_id, "apply"]) => {
            handle_http_request_apply(storage_root, &org, &app, mr_id, &request.body)
        }
        _ => Ok((404, json!({"error": "unknown endpoint"}))),
    }
}

fn http_project_parts(path: &str) -> Result<(String, String, Vec<String>), CliError> {
    let path = path.split_once('?').map_or(path, |(path, _)| path);
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(percent_decode)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() < 4 || parts[0] != "v1" || parts[1] != "projects" {
        return Err(CliError::Usage("unknown endpoint".to_string()));
    }
    Ok((parts[2].clone(), parts[3].clone(), parts[4..].to_vec()))
}

fn http_project_root(storage_root: &Path, org: &str, app: &str) -> PathBuf {
    storage_root
        .join(".codefire-server")
        .join("projects")
        .join(org)
        .join(app)
}

fn http_lock_options(request: &Value) -> Result<LockOptions, CliError> {
    let Some(lock) = request.get("lock") else {
        return Ok(LockOptions::default());
    };
    let wait = lock.get("wait").and_then(Value::as_bool).unwrap_or(false);
    let timeout_ms = match lock.get("timeout_ms") {
        None | Some(Value::Null) => None,
        Some(value) => Some(value.as_u64().ok_or_else(|| {
            CliError::Usage("lock.timeout_ms must be an unsigned integer".to_string())
        })?),
    };
    Ok(LockOptions {
        wait: wait || timeout_ms.is_some(),
        timeout_ms,
    })
}

fn handle_http_list(storage_root: &Path, org: &str, app: &str) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    let dirs = remote_dirs(&project_root);
    if !dirs.branches.exists() {
        return Ok((404, json!({"error": "remote project not found"})));
    }
    let branches = list_remote_branches(&format!("cf://{}/{org}/{app}", storage_root.display()))?
        .into_iter()
        .map(|branch| json!({"name": branch.name, "head": branch.head}))
        .collect::<Vec<_>>();
    Ok((200, json!({"branches": branches})))
}

fn handle_http_bundle(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    let branch_path = remote_branch_path(&project_root, branch_name);
    let Some(branch) = read_optional_json(&branch_path)? else {
        return Ok((404, json!({"error": "remote branch not found"})));
    };
    let head = required_string(&branch, &["head"])?;
    let objects = remote_dirs(&project_root).objects;
    codefire_store::validate_sealed_commit(&objects, &head)?;
    let records = collect_object_records(&objects, &head)?;
    Ok((200, json!({"branch": branch, "objects": records})))
}

fn handle_http_upload(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
    request: &Value,
) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    ensure_remote_layout(&project_root)?;
    let head = required_string(request, &["head"])?;
    let local_branch = request
        .get("branch")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let records = request
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage("upload request must include objects".to_string()))?;
    if !head.starts_with("CF-COMMIT-") || local_branch.is_empty() {
        return Ok((
            400,
            json!({"error": "upload request must include branch, head, and objects"}),
        ));
    }
    let actor = request
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let target =
        super::signatures::remote_request_target(org, app, "upload", Some(branch_name), None);
    super::signatures::verify_remote_request_signature(
        &project_root,
        "upload",
        actor,
        &target,
        request.get("request_signature"),
    )?;
    let records_value = Value::Array(records.to_vec());
    let records_hash = sha256_hex(&codefire_store::canonical_json(&records_value)?);
    let idempotency_payload = remote_upload_payload(
        &project_root,
        branch_name,
        &local_branch,
        &head,
        Some(&records_hash),
    );
    if let Some(key) = remote_idempotency_key(request) {
        if let Some(result) = load_remote_idempotency_result(
            &project_root,
            "remote_upload",
            &key,
            &idempotency_payload,
        )? {
            return Ok((200, result));
        }
    }
    let dirs = remote_dirs(&project_root);
    let lock_options = http_lock_options(request)?;
    let _lock = FileLock::acquire_with_options(
        dirs.locks
            .join(format!("branch-{}.lock", ref_file_name(branch_name))),
        &lock_options,
    )?;
    let tmp_objects = project_root
        .join("tmp")
        .join(format!(
            "upload-{}-{}",
            std::process::id(),
            stable_hash_48(head.as_bytes())
        ))
        .join("objects");
    if tmp_objects.exists() {
        fs::remove_dir_all(&tmp_objects)?;
    }
    ensure_object_dirs(&tmp_objects)?;
    copy_dir_recursive(&dirs.objects, &tmp_objects)?;
    write_object_records(&tmp_objects, records.iter().cloned())?;
    codefire_store::validate_sealed_commit(&tmp_objects, &head)?;
    super::signatures::enforce_remote_commit_signature_policy(&project_root, &tmp_objects, &head)?;
    if let Some(current) = read_optional_json(&remote_branch_path(&project_root, branch_name))? {
        let current_head = required_string(&current, &["head"])?;
        if !is_ancestor_in_objects(&tmp_objects, &current_head, &head)? {
            let _ = fs::remove_dir_all(tmp_objects.parent().unwrap_or(&tmp_objects));
            return Ok((
                409,
                json!({"error": "upload rejected: remote branch is not an ancestor of local branch head"}),
            ));
        }
    }
    let generation = next_remote_generation(&project_root)?;
    write_object_records(&dirs.objects, records.iter().cloned())?;
    write_json_atomic(
        &remote_branch_path(&project_root, branch_name),
        &json!({
            "version": 1,
            "name": branch_name,
            "head": head,
            "generation": generation,
            "uploaded_from": local_branch,
            "uploaded_by": request.get("actor").and_then(Value::as_str).unwrap_or("local"),
            "transport": "http",
            "updated_at": now_iso_utc(),
        }),
    )?;
    let _ = fs::remove_dir_all(tmp_objects.parent().unwrap_or(&tmp_objects));
    let response = json!({"branch": branch_name, "head": head});
    if let Some(key) = remote_idempotency_key(request) {
        save_remote_idempotency_result(
            &project_root,
            "remote_upload",
            &key,
            &idempotency_payload,
            &response,
        )?;
    }
    Ok((200, response))
}

fn handle_http_request_merge(
    storage_root: &Path,
    org: &str,
    app: &str,
    request: &Value,
) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    ensure_remote_layout(&project_root)?;
    let actor = request
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let request_target =
        super::signatures::remote_request_target(org, app, "request_merge", None, None);
    super::signatures::verify_remote_request_signature(
        &project_root,
        "request_merge",
        actor,
        &request_target,
        request.get("request_signature"),
    )?;
    let source_url = required_string(request, &["source_url"])?;
    let target_url = required_string(request, &["target_url"])?;
    let (_, source_root, source_branch) = http_remote_branch(storage_root, &source_url)?;
    let (target_info, target_root, target_branch) = http_remote_branch(storage_root, &target_url)?;
    if target_root != project_root || target_info.org != org || target_info.app != app {
        return Ok((
            400,
            json!({"error": "target_url must belong to this project"}),
        ));
    }
    let source_head = required_string(&source_branch, &["head"])?;
    let target_head = required_string(&target_branch, &["head"])?;
    codefire_store::validate_sealed_commit(&remote_dirs(&source_root).objects, &source_head)?;
    codefire_store::validate_sealed_commit(&remote_dirs(&target_root).objects, &target_head)?;
    let idempotency_payload = remote_merge_payload(
        &project_root,
        &source_url,
        &source_head,
        &target_url,
        &target_head,
    );
    if let Some(key) = remote_idempotency_key(request) {
        if let Some(result) = load_remote_idempotency_result(
            &project_root,
            "remote_request_merge",
            &key,
            &idempotency_payload,
        )? {
            return Ok((200, result));
        }
    }
    let lock_options = http_lock_options(request)?;
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&project_root).locks.join("merge-requests.lock"),
        &lock_options,
    )?;
    let mut mr_payload = json!({
        "version": 1,
        "source_url": source_url,
        "source_head": source_head,
        "target_url": target_url,
        "target_head_at_request": target_head,
        "status": "open",
        "created_by": request.get("actor").and_then(Value::as_str).unwrap_or("local"),
        "created_at": now_iso_utc(),
    });
    let mr_id = format!(
        "MR-{}",
        &sha256_hex(&codefire_store::canonical_json(&mr_payload)?)[..12]
    );
    mr_payload["id"] = Value::String(mr_id.clone());
    write_json_atomic(
        &remote_dirs(&project_root)
            .merge_requests
            .join(format!("{mr_id}.json")),
        &mr_payload,
    )?;
    let response = json!({
        "id": mr_id,
        "source": required_string(&mr_payload, &["source_head"])?,
        "target": required_string(&mr_payload, &["target_head_at_request"])?,
    });
    if let Some(key) = remote_idempotency_key(request) {
        save_remote_idempotency_result(
            &project_root,
            "remote_request_merge",
            &key,
            &idempotency_payload,
            &response,
        )?;
    }
    Ok((200, response))
}

fn handle_http_request_list(
    storage_root: &Path,
    org: &str,
    app: &str,
) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    let dirs = remote_dirs(&project_root);
    if !dirs.merge_requests.exists() {
        return Ok((404, json!({"error": "remote project not found"})));
    }
    let mut requests = Vec::new();
    let mut paths = fs::read_dir(&dirs.merge_requests)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let mr = read_json(&path)?;
        validate_http_merge_request_record(storage_root, &mr)?;
        let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
        let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
        let current_target =
            read_optional_json(&remote_branch_path(&target_root, &target_info.branch))?;
        let mut status = mr
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("open")
            .to_string();
        if let Some(current_target) = current_target {
            if required_string(&current_target, &["head"])?
                != required_string(&mr, &["target_head_at_request"])?
            {
                status = "stale".to_string();
            }
        }
        requests.push(json!({
            "id": required_string(&mr, &["id"])?,
            "status": status,
            "source_url": required_string(&mr, &["source_url"])?,
            "target_url": required_string(&mr, &["target_url"])?,
        }));
    }
    Ok((200, json!({"merge_requests": requests})))
}

fn handle_http_request_review(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    request: &Value,
) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    let mr_path = remote_dirs(&project_root)
        .merge_requests
        .join(format!("{mr_id}.json"));
    let Some(mut mr) = read_optional_json(&mr_path)? else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    validate_http_merge_request_record(storage_root, &mr)?;
    let actor = request
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let request_target =
        super::signatures::remote_request_target(org, app, "review", None, Some(mr_id));
    super::signatures::verify_remote_request_signature(
        &project_root,
        "review",
        actor,
        &request_target,
        request.get("request_signature"),
    )?;
    let decision = required_string(request, &["decision"])?;
    if decision != "approve" && decision != "reject" {
        return Ok((
            400,
            json!({"error": "review decision must be approve or reject"}),
        ));
    }
    let reviewer = required_string(request, &["reviewer"])?;
    let comment = request.get("comment").and_then(Value::as_str).unwrap_or("");
    let idempotency_payload =
        remote_review_payload(&project_root, &mr, mr_id, &reviewer, &decision, comment)?;
    if let Some(key) = remote_idempotency_key(request) {
        if let Some(result) = load_remote_idempotency_result(
            &project_root,
            "remote_request_review",
            &key,
            &idempotency_payload,
        )? {
            return Ok((200, result));
        }
    }
    let lock_options = http_lock_options(request)?;
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&project_root)
            .locks
            .join(format!("merge-request-{}.lock", ref_file_name(mr_id))),
        &lock_options,
    )?;
    if http_merge_request_is_stale(storage_root, &mr)? {
        mr["status"] = Value::String("stale".to_string());
        write_json_atomic(&mr_path, &mr)?;
        return Ok((
            409,
            json!({"error": format!("merge request is stale: {mr_id}")}),
        ));
    }
    let mut reviews = mr
        .get("reviews")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    reviews.push(json!({
        "reviewer": reviewer,
        "decision": decision,
        "comment": comment,
        "reviewed_at": now_iso_utc(),
    }));
    mr["reviews"] = Value::Array(reviews);
    mr["status"] = Value::String(if decision == "approve" {
        "approved".to_string()
    } else {
        "rejected".to_string()
    });
    write_json_atomic(&mr_path, &mr)?;
    let response = json!({"id": mr_id, "decision": decision, "reviewer": reviewer});
    if let Some(key) = remote_idempotency_key(request) {
        save_remote_idempotency_result(
            &project_root,
            "remote_request_review",
            &key,
            &idempotency_payload,
            &response,
        )?;
    }
    Ok((200, response))
}

fn handle_http_request_apply(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    request: &Value,
) -> Result<(u16, Value), CliError> {
    let project_root = http_project_root(storage_root, org, app);
    let mr_path = remote_dirs(&project_root)
        .merge_requests
        .join(format!("{mr_id}.json"));
    let Some(mut mr) = read_optional_json(&mr_path)? else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    validate_http_merge_request_record(storage_root, &mr)?;
    let actor = request
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let request_target =
        super::signatures::remote_request_target(org, app, "apply", None, Some(mr_id));
    super::signatures::verify_remote_request_signature(
        &project_root,
        "apply",
        actor,
        &request_target,
        request.get("request_signature"),
    )?;
    let idempotency_payload = remote_apply_payload(&project_root, &mr, mr_id)?;
    if let Some(key) = remote_idempotency_key(request) {
        if let Some(result) = load_remote_idempotency_result(
            &project_root,
            "remote_request_apply",
            &key,
            &idempotency_payload,
        )? {
            return Ok((200, result));
        }
    }
    if mr.get("status").and_then(Value::as_str) != Some("approved") {
        return Ok((
            409,
            json!({"error": format!("merge request must be approved before apply: {mr_id}")}),
        ));
    }
    if http_merge_request_is_stale(storage_root, &mr)? {
        mr["status"] = Value::String("stale".to_string());
        write_json_atomic(&mr_path, &mr)?;
        return Ok((
            409,
            json!({"error": format!("merge request is stale: {mr_id}")}),
        ));
    }
    let (_, source_root, source_branch) =
        http_remote_branch(storage_root, &required_string(&mr, &["source_url"])?)?;
    let (target_info, target_root, target_branch) =
        http_remote_branch(storage_root, &required_string(&mr, &["target_url"])?)?;
    if target_root != project_root || target_info.org != org || target_info.app != app {
        return Ok((
            400,
            json!({"error": "target_url must belong to this project"}),
        ));
    }
    let source_head = required_string(&source_branch, &["head"])?;
    let target_head = required_string(&target_branch, &["head"])?;
    if !is_ancestor_in_objects(
        &remote_dirs(&source_root).objects,
        &target_head,
        &source_head,
    )? {
        return Ok((
            409,
            json!({"error": "request apply rejected: source is not a fast-forward of target"}),
        ));
    }
    super::signatures::enforce_remote_commit_signature_policy(
        &target_root,
        &remote_dirs(&source_root).objects,
        &source_head,
    )?;
    let lock_options = http_lock_options(request)?;
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&target_root).locks.join(format!(
            "branch-{}.lock",
            ref_file_name(&target_info.branch)
        )),
        &lock_options,
    )?;
    let generation = next_remote_generation(&target_root)?;
    copy_object_graph(
        &remote_dirs(&source_root).objects,
        &remote_dirs(&target_root).objects,
        &source_head,
    )?;
    write_json_atomic(
        &remote_branch_path(&target_root, &target_info.branch),
        &json!({
            "version": 1,
            "name": target_info.branch,
            "head": source_head,
            "generation": generation,
            "applied_from": required_string(&mr, &["source_url"])?,
            "applied_by": request.get("actor").and_then(Value::as_str).unwrap_or("local"),
            "transport": "http",
            "updated_at": now_iso_utc(),
        }),
    )?;
    mr["status"] = Value::String("applied".to_string());
    mr["applied_at"] = Value::String(now_iso_utc());
    mr["applied_by"] = Value::String(
        request
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("local")
            .to_string(),
    );
    mr["applied_head"] = Value::String(source_head.clone());
    write_json_atomic(&mr_path, &mr)?;
    let response = json!({"id": mr_id, "target_branch": target_info.branch, "head": source_head});
    if let Some(key) = remote_idempotency_key(request) {
        save_remote_idempotency_result(
            &project_root,
            "remote_request_apply",
            &key,
            &idempotency_payload,
            &response,
        )?;
    }
    Ok((200, response))
}

fn http_remote_branch(
    storage_root: &Path,
    url: &str,
) -> Result<(CfHttpUrl, PathBuf, Value), CliError> {
    let info = parse_cf_http_url(url)?;
    let project_root = http_project_root(storage_root, &info.org, &info.app);
    let branch = read_optional_json(&remote_branch_path(&project_root, &info.branch))?
        .ok_or_else(|| CliError::Usage(format!("remote branch not found: {url}")))?;
    let head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&remote_dirs(&project_root).objects, &head)?;
    Ok((info, project_root, branch))
}

fn validate_http_merge_request_record(storage_root: &Path, mr: &Value) -> Result<(), CliError> {
    let (_, source_root, _) =
        http_remote_branch(storage_root, &required_string(mr, &["source_url"])?)?;
    let (_, target_root, _) =
        http_remote_branch(storage_root, &required_string(mr, &["target_url"])?)?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&source_root).objects,
        &required_string(mr, &["source_head"])?,
    )?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&target_root).objects,
        &required_string(mr, &["target_head_at_request"])?,
    )?;
    if let Some(applied_head) = mr.get("applied_head").and_then(Value::as_str) {
        codefire_store::validate_sealed_commit(&remote_dirs(&target_root).objects, applied_head)?;
    }
    Ok(())
}

fn http_merge_request_is_stale(storage_root: &Path, mr: &Value) -> Result<bool, CliError> {
    let target = parse_cf_http_url(&required_string(mr, &["target_url"])?)?;
    let target_root = http_project_root(storage_root, &target.org, &target.app);
    let current_target = read_optional_json(&remote_branch_path(&target_root, &target.branch))?;
    Ok(current_target
        .and_then(|branch| required_string(&branch, &["head"]).ok())
        .is_some_and(|head| {
            head != required_string(mr, &["target_head_at_request"]).unwrap_or_default()
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn http_request_rejects_truncated_body() {
        let request =
            b"POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: 7\r\n\r\n{}";
        let error = read_http_request(&mut Cursor::new(request.to_vec())).unwrap_err();
        assert!(error.to_string().contains("incomplete HTTP request body"));
    }

    #[test]
    fn http_request_rejects_oversized_body_before_reading_body() {
        let request = format!(
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_HTTP_BODY_BYTES + 1
        );
        let error = read_http_request(&mut Cursor::new(request.into_bytes())).unwrap_err();
        assert!(matches!(error, CliError::HttpPayloadTooLarge(_)));
    }
}
