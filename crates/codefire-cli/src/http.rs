use super::http_tls;
use super::remote::*;
use super::*;
use crate::compatibility::validate_record_version;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_HTTP_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_HTTP_HEADER_BYTES: usize = 64 * 1024;
const MAX_HTTP_ERROR_RESPONSE_BYTES: usize = 8 * 1024;
const DEFAULT_HTTP_TIMEOUT_MS: u64 = 30_000;
const MAX_HTTP_SERVER_CONNECTIONS: usize = 32;

#[derive(Debug, Clone)]
struct HttpRequestDeadline {
    started_at: Instant,
    limit: Duration,
    limit_ms: u64,
}

impl HttpRequestDeadline {
    fn from_profile(profile: &codefire_core::DecisionWorkProfile) -> Self {
        let limit_ms = profile
            .request_deadline_ms()
            .expect("HTTP v1 profile must advertise a request deadline");
        Self {
            started_at: Instant::now(),
            limit: Duration::from_millis(limit_ms),
            limit_ms,
        }
    }

    fn check(&self, phase: &'static str) -> Result<(), CliError> {
        if self.started_at.elapsed() >= self.limit {
            return Err(self.error(phase));
        }
        Ok(())
    }

    fn remaining_ms(&self, phase: &'static str) -> Result<u64, CliError> {
        self.check(phase)?;
        let remaining = self.limit.saturating_sub(self.started_at.elapsed());
        u64::try_from(remaining.as_millis())
            .map(|millis| millis.max(1))
            .map_err(|_| CliError::InvalidRepository("HTTP deadline overflow".to_string()))
    }

    fn error(&self, phase: &'static str) -> CliError {
        CliError::RequestDeadlineExceeded {
            phase,
            limit_ms: self.limit_ms,
        }
    }

    fn is_expired(&self) -> bool {
        self.started_at.elapsed() >= self.limit
    }
}

#[cfg(test)]
static HTTP_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
thread_local! {
    static HTTP_MUTATION_STORE_BINDING_EXPECTED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static HTTP_MUTATION_STORE_BINDING_CURRENT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static HTTP_MUTATION_STORE_BINDING_STRUCTURAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn record_http_mutation_store_binding(expected: u64, current: u64, structural: bool) {
    HTTP_MUTATION_STORE_BINDING_EXPECTED.with(|value| value.set(expected));
    HTTP_MUTATION_STORE_BINDING_CURRENT.with(|value| value.set(current));
    HTTP_MUTATION_STORE_BINDING_STRUCTURAL.with(|value| value.set(structural));
}

#[cfg(test)]
fn take_http_mutation_store_binding() -> (u64, u64, bool) {
    let expected = HTTP_MUTATION_STORE_BINDING_EXPECTED.with(|value| value.replace(0));
    let current = HTTP_MUTATION_STORE_BINDING_CURRENT.with(|value| value.replace(0));
    let structural = HTTP_MUTATION_STORE_BINDING_STRUCTURAL.with(|value| value.replace(false));
    (expected, current, structural)
}

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
        .map(decode_http_path_component)
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
    let timeout = http_timeout()?;
    let address = (endpoint.host.as_str(), endpoint.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| {
            CliError::Usage(format!("unable to resolve HTTP host: {}", endpoint.host))
        })?;
    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
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
        "{method} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        endpoint.path,
        endpoint.host_header(),
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
        return Err(http_remote_error(status, &value));
    }
    Ok(value)
}

fn http_remote_error(status: u16, value: &Value) -> CliError {
    let detail = value
        .get("error")
        .and_then(Value::as_str)
        .or_else(|| value.get("message").and_then(Value::as_str))
        .unwrap_or("HTTP remote error");
    let exit_code = value
        .get("exit_code")
        .and_then(Value::as_i64)
        .and_then(exit_code_from_i64)
        .unwrap_or(ExitCode::RemoteRejected);
    let kind = value
        .get("diagnostics")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("kind"))
        .and_then(Value::as_str)
        .or_else(|| value.get("kind").and_then(Value::as_str))
        .map(str::to_string)
        .unwrap_or_else(|| match exit_code {
            ExitCode::AuthenticationOrSignatureFailure => {
                "remote_authentication_or_signature_error".to_string()
            }
            ExitCode::IdempotencyConflict => "remote_idempotency_conflict".to_string(),
            _ => "remote_rejected".to_string(),
        });
    let resource_budget = value
        .get("diagnostics")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("kind").and_then(Value::as_str) == Some("resource_budget_exceeded")
            })
        })
        .or_else(|| {
            (value.get("kind").and_then(Value::as_str) == Some("resource_budget_exceeded"))
                .then_some(value)
        })
        .and_then(|item| {
            serde_json::from_value::<codefire_core::DecisionWorkBudgetExceeded>(item.clone()).ok()
        })
        .map(Box::new);
    CliError::RemoteDiagnostic {
        status,
        exit_code,
        kind,
        message: format!("HTTP remote error: {detail}"),
        resource_budget,
    }
}

fn exit_code_from_i64(value: i64) -> Option<ExitCode> {
    match value {
        value if value == ExitCode::InvalidUsageOrConfig.code() as i64 => {
            Some(ExitCode::InvalidUsageOrConfig)
        }
        value if value == ExitCode::RepositoryCorruption.code() as i64 => {
            Some(ExitCode::RepositoryCorruption)
        }
        value if value == ExitCode::ObjectReferenceInvalid.code() as i64 => {
            Some(ExitCode::ObjectReferenceInvalid)
        }
        value if value == ExitCode::SealedCommitInvalid.code() as i64 => {
            Some(ExitCode::SealedCommitInvalid)
        }
        value if value == ExitCode::RemoteRejected.code() as i64 => Some(ExitCode::RemoteRejected),
        value if value == ExitCode::AuthenticationOrSignatureFailure.code() as i64 => {
            Some(ExitCode::AuthenticationOrSignatureFailure)
        }
        value if value == ExitCode::IdempotencyConflict.code() as i64 => {
            Some(ExitCode::IdempotencyConflict)
        }
        value if value == ExitCode::ResourceBudgetExceeded.code() as i64 => {
            Some(ExitCode::ResourceBudgetExceeded)
        }
        _ => None,
    }
}

fn http_budget_exceeded(
    dimension: codefire_core::DecisionWorkDimension,
    phase: codefire_core::DecisionWorkPhase,
    command: &str,
) -> CliError {
    let profile = codefire_core::DecisionWorkProfile::http_v1();
    let limit = profile
        .limit(dimension)
        .expect("HTTP v1 profile must define every HTTP parser dimension")
        .limit;
    let mut ledger = codefire_core::DecisionWorkLedger::new(
        profile,
        codefire_core::DecisionWorkCommandKind::HttpRequest,
        command,
    )
    .expect("HTTP v1 profile must apply to HTTP requests");
    let accounting_error = ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(dimension, limit.saturating_add(1)),
        )
        .expect_err("limit plus one must exhaust the HTTP budget");
    CliError::ResourceBudget(Box::new(
        accounting_error
            .budget_exceeded()
            .expect("limit plus one must be a typed budget exhaustion")
            .clone(),
    ))
}

fn charge_http_limit_failure(
    ledger: &mut codefire_core::DecisionWorkLedger,
    dimension: codefire_core::DecisionWorkDimension,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> CliError {
    let limit = ledger
        .profile()
        .limit(dimension)
        .expect("HTTP request ledger must define parser dimensions")
        .limit;
    let error = ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(dimension, limit.saturating_add(1)),
        )
        .expect_err("HTTP limit plus one must exhaust the request ledger");
    upload_work_error(error, operation)
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
            let completed_header_bytes = position.checked_add(4).ok_or_else(|| {
                CliError::InvalidRepository("HTTP response header size overflow".to_string())
            })?;
            if completed_header_bytes > MAX_HTTP_HEADER_BYTES {
                return Err(http_budget_exceeded(
                    codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
                    codefire_core::DecisionWorkPhase::ResponseHeaders,
                    "http response",
                ));
            }
            break position;
        }
        if buffer.len() > MAX_HTTP_HEADER_BYTES {
            return Err(http_budget_exceeded(
                codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
                codefire_core::DecisionWorkPhase::ResponseHeaders,
                "http response",
            ));
        }
    };
    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let content_length = parse_content_length(
        head.lines().skip(1),
        codefire_core::DecisionWorkDimension::ResponseBytes,
        codefire_core::DecisionWorkPhase::ResponseBody,
        "http response",
        None,
    )?;
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(http_budget_exceeded(
            codefire_core::DecisionWorkDimension::ResponseBytes,
            codefire_core::DecisionWorkPhase::ResponseBody,
            "http response",
        ));
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

impl HttpEndpoint {
    fn host_header(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
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
    let (host, port) = parse_http_authority(scheme, authority)?;
    Ok(HttpEndpoint {
        scheme: scheme.to_string(),
        host,
        port,
        path: format!("/{path}"),
    })
}

fn parse_http_authority(scheme: &str, authority: &str) -> Result<(String, u16), CliError> {
    if authority.is_empty() {
        return Err(CliError::Usage("HTTP URL authority is empty".to_string()));
    }
    let default_port = if scheme == "http" { 80 } else { 443 };
    if let Some(rest) = authority.strip_prefix('[') {
        let Some((host, after_host)) = rest.split_once(']') else {
            return Err(CliError::Usage(format!(
                "invalid bracketed IPv6 authority: {authority}"
            )));
        };
        if host.is_empty() {
            return Err(CliError::Usage("HTTP IPv6 host is empty".to_string()));
        }
        let port = if after_host.is_empty() {
            default_port
        } else {
            let raw_port = after_host.strip_prefix(':').ok_or_else(|| {
                CliError::Usage(format!("invalid HTTP authority suffix: {authority}"))
            })?;
            parse_http_port(raw_port)?
        };
        return Ok((host.to_string(), port));
    }
    if authority.matches(':').count() > 1 {
        return Err(CliError::Usage(
            "IPv6 HTTP authority must use [addr]:port syntax".to_string(),
        ));
    }
    if let Some((host, raw_port)) = authority.rsplit_once(':') {
        if host.is_empty() {
            return Err(CliError::Usage("HTTP host is empty".to_string()));
        }
        Ok((host.to_string(), parse_http_port(raw_port)?))
    } else {
        Ok((authority.to_string(), default_port))
    }
}

fn parse_http_port(raw_port: &str) -> Result<u16, CliError> {
    if raw_port.is_empty() {
        return Err(CliError::Usage("HTTP port is empty".to_string()));
    }
    raw_port
        .parse::<u16>()
        .map_err(|_| CliError::Usage(format!("invalid HTTP port: {raw_port}")))
}

fn http_timeout() -> Result<Duration, CliError> {
    let Ok(value) = std::env::var("CODEFIRE_HTTP_TIMEOUT_MS") else {
        return Ok(Duration::from_millis(DEFAULT_HTTP_TIMEOUT_MS));
    };
    let parsed = value.parse::<u64>().map_err(|_| {
        CliError::Usage(format!(
            "invalid CODEFIRE_HTTP_TIMEOUT_MS: expected positive integer milliseconds, got {value}"
        ))
    })?;
    if parsed == 0 {
        return Err(CliError::Usage(format!(
            "invalid CODEFIRE_HTTP_TIMEOUT_MS: expected positive integer milliseconds, got {value}"
        )));
    }
    Ok(Duration::from_millis(parsed))
}

pub(crate) fn serve_http(options: &ServeOptions) -> Result<(), CliError> {
    let storage_root = Arc::new(absolute_path(&options.storage_root)?);
    fs::create_dir_all(storage_root.as_path())?;
    let listener = TcpListener::bind((options.host.as_str(), options.port))?;
    let address = listener.local_addr()?;
    let tls_config = match (&options.tls_cert, &options.tls_key) {
        (Some(cert), Some(key)) => Some(http_tls::server_config(
            cert,
            key,
            options.tls_client_ca.as_deref(),
        )?),
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
    let connection_limiter = Arc::new(ServerConnectionLimiter::new(MAX_HTTP_SERVER_CONNECTIONS));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Some(permit) = connection_limiter.try_acquire() {
                    spawn_http_connection_handler(
                        Arc::clone(&storage_root),
                        tls_config.clone(),
                        stream,
                        permit,
                    )?;
                } else {
                    reject_overloaded_http_connection(stream, tls_config.is_some());
                }
            }
            Err(error) => return Err(CliError::Io(error)),
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ServerConnectionLimiter {
    active: AtomicUsize,
    max: usize,
}

impl ServerConnectionLimiter {
    fn new(max: usize) -> Self {
        Self {
            active: AtomicUsize::new(0),
            max: max.max(1),
        }
    }

    fn try_acquire(self: &Arc<Self>) -> Option<ServerConnectionPermit> {
        loop {
            let active = self.active.load(Ordering::Acquire);
            if active >= self.max {
                return None;
            }
            if self
                .active
                .compare_exchange(active, active + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some(ServerConnectionPermit {
                    limiter: Arc::clone(self),
                });
            }
        }
    }

    #[cfg(test)]
    fn active(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
struct ServerConnectionPermit {
    limiter: Arc<ServerConnectionLimiter>,
}

impl Drop for ServerConnectionPermit {
    fn drop(&mut self) {
        self.limiter.active.fetch_sub(1, Ordering::AcqRel);
    }
}

fn spawn_http_connection_handler(
    storage_root: Arc<PathBuf>,
    tls_config: Option<Arc<rustls::ServerConfig>>,
    stream: TcpStream,
    permit: ServerConnectionPermit,
) -> Result<(), CliError> {
    std::thread::Builder::new()
        .name("codefire-http-connection".to_string())
        .spawn(move || {
            let _permit = permit;
            if let Err(error) = handle_tcp_http_connection(&storage_root, tls_config, stream) {
                eprintln!("HTTP request failed: {error}");
            }
        })?;
    Ok(())
}

fn handle_tcp_http_connection(
    storage_root: &Path,
    tls_config: Option<Arc<rustls::ServerConfig>>,
    stream: TcpStream,
) -> Result<(), CliError> {
    configure_http_server_stream(&stream)?;
    if let Some(config) = tls_config {
        let mut tls_stream = http_tls::accept_server(stream, config)?;
        handle_http_stream(storage_root, &mut tls_stream)
    } else {
        let mut stream = stream;
        handle_http_stream(storage_root, &mut stream)
    }
}

fn configure_http_server_stream(stream: &TcpStream) -> Result<(), CliError> {
    let timeout = http_timeout()?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    Ok(())
}

fn reject_overloaded_http_connection(mut stream: TcpStream, tls_enabled: bool) {
    let _ = configure_http_server_stream(&stream);
    if !tls_enabled {
        let _ = write_http_json(
            &mut stream,
            503,
            &json!({"error": "CodeFire HTTP server connection limit reached"}),
        );
    }
}

fn handle_http_stream<S: Read + Write>(
    storage_root: &Path,
    stream: &mut S,
) -> Result<(), CliError> {
    let profile = codefire_core::DecisionWorkProfile::http_v1();
    let deadline = HttpRequestDeadline::from_profile(&profile);
    let mut ledger = Some(
        codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http request",
        )
        .map_err(|error| upload_work_error(error, "initialize HTTP request"))?,
    );
    let mut request = match read_http_request_with_ledger(
        stream,
        ledger.as_mut().expect("request ledger must be present"),
        &deadline,
    ) {
        Ok(request) => request,
        Err(error) => {
            let reason = http_request_terminal_reason(&error);
            terminate_http_request(
                ledger.as_mut().expect("request ledger must be present"),
                reason,
            )?;
            let (status, payload) = http_error_response(&error);
            let body = encode_http_terminal_error(&payload);
            return write_http_json_body(stream, status, &body);
        }
    };
    let mut prepared = None;
    let response = dispatch_http_with_ledger(
        storage_root,
        &mut request,
        &mut ledger,
        &mut prepared,
        &deadline,
    );
    request.release_body(
        ledger
            .as_mut()
            .expect("request ledger must return from dispatch"),
    )?;
    request.release_routing(
        ledger
            .as_mut()
            .expect("request ledger must return from dispatch"),
    )?;

    write_http_dispatch_outcome(
        stream,
        response,
        prepared,
        ledger
            .as_mut()
            .expect("request ledger must return from dispatch"),
        &deadline,
    )
}

fn write_http_dispatch_outcome<S: Read + Write>(
    stream: &mut S,
    response: Result<HttpDispatchResponse, CliError>,
    mut prepared: Option<PreparedHttpResponse>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<(), CliError> {
    let HttpDispatchResponse {
        status,
        payload,
        mut payload_owner,
    } = match response {
        Ok(response) => response,
        Err(error) => {
            if let Some(prepared) = prepared.take() {
                release_prepared_http_response(
                    ledger,
                    prepared,
                    "release failed HTTP route response",
                )?;
            }
            let reason = http_request_terminal_reason(&error);
            terminate_http_request(ledger, reason)?;
            let (status, payload) = http_error_response(&error);
            return write_http_json_body(stream, status, &encode_http_terminal_error(&payload));
        }
    };
    if let Some(prepared) = prepared {
        // Mutating routes prepare their response and perform the final deadline
        // check before entering the durable reducer. Once dispatch succeeds,
        // do not turn a committed mutation into a no-effect deadline response.
        let write_result = write_http_json_body(stream, prepared.status, &prepared.body);
        release_prepared_http_response(ledger, prepared, "release HTTP response body")?;
        drop(payload);
        release_object_record_payload_owner(&mut payload_owner, ledger)?;
        release_all_http_current_memory(ledger, "release completed HTTP route memory")?;
        if ledger.snapshot().terminal_reason.is_none() {
            ledger
                .finish(codefire_core::DecisionWorkTerminalReason::Complete)
                .map_err(|error| upload_work_error(error, "finish HTTP request"))?;
        }
        return write_result;
    }
    write_http_json_with_ledger(
        stream,
        status,
        payload,
        &mut payload_owner,
        ledger,
        deadline,
    )
}

struct HttpDispatchResponse {
    status: u16,
    payload: Value,
    payload_owner: Option<AccountedObjectRecordPayloadOwnership>,
}

fn release_object_record_payload_owner(
    owner: &mut Option<AccountedObjectRecordPayloadOwnership>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    if let Some(owner) = owner.as_mut() {
        owner.release_after_payload_drop(ledger)?;
    }
    *owner = None;
    Ok(())
}

fn http_request_terminal_reason(error: &CliError) -> codefire_core::DecisionWorkTerminalReason {
    match error {
        CliError::RequestDeadlineExceeded { .. } => {
            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded
        }
        CliError::ResourceBudget(_) => codefire_core::DecisionWorkTerminalReason::BudgetExceeded,
        CliError::Usage(_)
        | CliError::AuthenticationOrSignature(_)
        | CliError::IdempotencyConflict(_)
        | CliError::LockContention(_)
        | CliError::RepositoryLockContention { .. } => {
            codefire_core::DecisionWorkTerminalReason::Complete
        }
        CliError::BranchReferenceSubjectChanged { .. }
        | CliError::OpenedRegistrySubjectChanged { .. } => {
            codefire_core::DecisionWorkTerminalReason::SubjectChanged
        }
        CliError::Store(error)
            if matches!(
                error.as_ref(),
                codefire_store::StoreError::SealedValidationSubjectChanged { .. }
            ) =>
        {
            codefire_core::DecisionWorkTerminalReason::SubjectChanged
        }
        CliError::Store(error)
            if matches!(
                error.as_ref(),
                codefire_store::StoreError::DecisionWorkAccounting(accounting)
                    if accounting.budget_exceeded().is_some()
            ) =>
        {
            codefire_core::DecisionWorkTerminalReason::BudgetExceeded
        }
        _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
    }
}

fn terminate_http_request(
    ledger: &mut codefire_core::DecisionWorkLedger,
    reason: codefire_core::DecisionWorkTerminalReason,
) -> Result<(), CliError> {
    for dimension in [
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        codefire_core::DecisionWorkDimension::RetainedBytes,
    ] {
        let current = ledger
            .snapshot()
            .current
            .get(&dimension)
            .copied()
            .unwrap_or_default();
        if current > 0 {
            ledger
                .release(dimension, current)
                .map_err(|error| upload_work_error(error, "release failed HTTP request memory"))?;
        }
    }
    if ledger.snapshot().terminal_reason.is_none() {
        ledger
            .finish(reason)
            .map_err(|error| upload_work_error(error, "finish failed HTTP request"))?;
    }
    Ok(())
}

fn write_http_json_with_ledger<S: Read + Write>(
    stream: &mut S,
    status: u16,
    payload: Value,
    payload_owner: &mut Option<AccountedObjectRecordPayloadOwnership>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<(), CliError> {
    if let Err(error) = deadline.check("response encoding") {
        drop(payload);
        release_object_record_payload_owner(payload_owner, ledger)?;
        terminate_http_request(
            ledger,
            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
        )?;
        let (status, payload) = http_error_response(&error);
        return write_http_json_body(stream, status, &encode_http_terminal_error(&payload));
    }
    let body = match encode_http_json_with_ledger_internal(&payload, ledger, Some(deadline)) {
        Ok(body) => body,
        Err(error) => {
            drop(payload);
            release_object_record_payload_owner(payload_owner, ledger)?;
            terminate_http_request(ledger, http_request_terminal_reason(&error))?;
            let (status, payload) = http_error_response(&error);
            let body = encode_http_terminal_error(&payload);
            return write_http_json_body(stream, status, &body);
        }
    };
    let header_bytes = match http_json_header_len(status, body.len()) {
        Ok(header_bytes) => header_bytes,
        Err(error) => {
            release_encoded_http_response(ledger, body, "release HTTP response body")?;
            drop(payload);
            release_object_record_payload_owner(payload_owner, ledger)?;
            terminate_http_request(ledger, http_request_terminal_reason(&error))?;
            let (status, payload) = http_error_response(&error);
            return write_http_json_body(stream, status, &encode_http_terminal_error(&payload));
        }
    };
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::ResponseHeaders,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(
            codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
            header_bytes,
        ),
    ) {
        let error = upload_work_error(error, "account HTTP response headers");
        release_encoded_http_response(ledger, body, "release HTTP response body")?;
        drop(payload);
        release_object_record_payload_owner(payload_owner, ledger)?;
        terminate_http_request(ledger, http_request_terminal_reason(&error))?;
        let (status, payload) = http_error_response(&error);
        let body = encode_http_terminal_error(&payload);
        return write_http_json_body(stream, status, &body);
    }
    if let Err(error) = deadline.check("response publication") {
        drop(body);
        drop(payload);
        release_object_record_payload_owner(payload_owner, ledger)?;
        terminate_http_request(
            ledger,
            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
        )?;
        let (status, payload) = http_error_response(&error);
        return write_http_json_body(stream, status, &encode_http_terminal_error(&payload));
    }
    let write_result = write_http_json_body(stream, status, &body);
    drop(payload);
    release_object_record_payload_owner(payload_owner, ledger)?;
    release_encoded_http_response(ledger, body, "release HTTP response body")?;
    release_all_http_current_memory(ledger, "release completed HTTP route memory")?;
    if ledger.snapshot().terminal_reason.is_none() {
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .map_err(|error| upload_work_error(error, "finish HTTP request"))?;
    }
    write_result
}

fn release_all_http_current_memory(
    ledger: &mut codefire_core::DecisionWorkLedger,
    operation: &str,
) -> Result<(), CliError> {
    for dimension in [
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        codefire_core::DecisionWorkDimension::RetainedBytes,
    ] {
        let current = ledger
            .snapshot()
            .current
            .get(&dimension)
            .copied()
            .unwrap_or_default();
        if current > 0 {
            ledger
                .release(dimension, current)
                .map_err(|error| upload_work_error(error, operation))?;
        }
    }
    Ok(())
}

fn release_encoded_http_response(
    ledger: &mut codefire_core::DecisionWorkLedger,
    body: Vec<u8>,
    operation: &str,
) -> Result<(), CliError> {
    let body_bytes = u64::try_from(body.len())
        .map_err(|_| CliError::InvalidRepository("HTTP response size overflow".to_string()))?;
    drop(body);
    ledger
        .release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            body_bytes,
        )
        .map_err(|error| upload_work_error(error, operation))
}

#[cfg(test)]
fn prepare_http_json_with_ledger(
    status: u16,
    payload: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedHttpResponse, CliError> {
    prepare_http_json_with_ledger_internal(status, payload, ledger, None)
}

#[cfg(test)]
fn prepare_http_json_with_ledger_internal(
    status: u16,
    payload: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: Option<&HttpRequestDeadline>,
) -> Result<PreparedHttpResponse, CliError> {
    let body = encode_http_json_with_ledger_internal(payload, ledger, deadline)?;
    let header_bytes = http_json_header_len(status, body.len())?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::ResponseHeaders,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
                header_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "account HTTP response headers"))?;
    Ok(PreparedHttpResponse { status, body })
}

// This raw API is intentionally staged before its production route wiring so
// publication and Store owners can adopt it without introducing ledger borrows.
#[allow(dead_code)]
fn prepare_mutation_responses_with_deadline(
    success_status: u16,
    success_payload: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<PreparedMutationResponses, CliError> {
    let success =
        encode_prepared_mutation_response(success_status, success_payload, ledger, deadline)?;
    let durable = match encode_prepared_mutation_response(
        503,
        &RETRY_SAME_REQUEST_RESPONSE,
        ledger,
        deadline,
    ) {
        Ok(durable) => durable,
        Err(error) => {
            let mut first_error = Some(error);
            record_cleanup_error(
                &mut first_error,
                release_prepared_mutation_response(
                    ledger,
                    success,
                    "release success response after durable response planning failure",
                ),
            );
            return Err(first_error.expect("response planning error must be retained"));
        }
    };

    let response_upper = success.body_bytes.max(durable.body_bytes);
    let header_upper = success.header_bytes.max(durable.header_bytes);
    let response_permit = match ledger.reserve(http_response_work_reservation(
        codefire_core::DecisionWorkPhase::Encoding,
        codefire_core::DecisionWorkDimension::ResponseBytes,
        response_upper,
    )) {
        Ok(permit) => permit,
        Err(error) => {
            let mut first_error = Some(upload_work_error(
                error,
                "reserve prepared mutation response bytes",
            ));
            release_prepared_mutation_pair(ledger, success, durable, &mut first_error);
            return Err(first_error.expect("response permit error must be retained"));
        }
    };
    let header_permit = match ledger.reserve(http_response_work_reservation(
        codefire_core::DecisionWorkPhase::ResponseHeaders,
        codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
        header_upper,
    )) {
        Ok(permit) => permit,
        Err(error) => {
            let mut first_error = Some(upload_work_error(
                error,
                "reserve prepared mutation response headers",
            ));
            record_cleanup_error(
                &mut first_error,
                ledger.cancel_reserved(&response_permit).map_err(|cancel| {
                    upload_work_error(cancel, "cancel prepared mutation response-byte permit")
                }),
            );
            release_prepared_mutation_pair(ledger, success, durable, &mut first_error);
            return Err(first_error.expect("response header permit error must be retained"));
        }
    };
    Ok(PreparedMutationResponses {
        ledger_identity: ledger.identity(),
        success: Some(success),
        durable: Some(durable),
        response_permit: Some(response_permit),
        header_permit: Some(header_permit),
        response_upper,
        header_upper,
    })
}

#[allow(dead_code)]
fn encode_prepared_mutation_response<T>(
    status: u16,
    payload: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<PreparedMutationResponse, CliError>
where
    T: Serialize + ?Sized,
{
    let body = encode_http_json_with_accounting(payload, ledger, Some(deadline), false)?;
    let body_bytes = match u64::try_from(body.len()) {
        Ok(body_bytes) => body_bytes,
        Err(_) => {
            let error = CliError::InvalidRepository("HTTP response size overflow".to_string());
            release_encoded_http_response(
                ledger,
                body,
                "release mutation response with invalid body size",
            )?;
            return Err(error);
        }
    };
    let header_bytes = match http_json_header_len(status, body.len()) {
        Ok(header_bytes) => header_bytes,
        Err(error) => {
            release_encoded_http_response(
                ledger,
                body,
                "release mutation response with invalid headers",
            )?;
            return Err(error);
        }
    };
    Ok(PreparedMutationResponse {
        prepared: PreparedHttpResponse { status, body },
        response_work: Some(http_response_work_reservation(
            codefire_core::DecisionWorkPhase::Encoding,
            codefire_core::DecisionWorkDimension::ResponseBytes,
            body_bytes,
        )),
        header_work: Some(http_response_work_reservation(
            codefire_core::DecisionWorkPhase::ResponseHeaders,
            codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
            header_bytes,
        )),
        body_bytes,
        header_bytes,
    })
}

#[derive(Serialize)]
#[allow(dead_code)]
struct RetrySameRequestResponse {
    error: &'static str,
    kind: &'static str,
    retry_same_request: bool,
}

#[allow(dead_code)]
const RETRY_SAME_REQUEST_RESPONSE: RetrySameRequestResponse = RetrySameRequestResponse {
    error: "the durable mutation completed without a confirmed response; retry the same request",
    kind: "retry_same_request",
    retry_same_request: true,
};

#[allow(dead_code)]
fn http_response_work_reservation(
    phase: codefire_core::DecisionWorkPhase,
    dimension: codefire_core::DecisionWorkDimension,
    amount: u64,
) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(phase, codefire_core::DecisionWorkSurface::Atomic)
        .charge(dimension, amount)
}

#[allow(dead_code)]
fn record_cleanup_error(first_error: &mut Option<CliError>, result: Result<(), CliError>) {
    if first_error.is_none() {
        if let Err(error) = result {
            *first_error = Some(error);
        }
    }
}

#[allow(dead_code)]
fn release_prepared_mutation_response(
    ledger: &mut codefire_core::DecisionWorkLedger,
    response: PreparedMutationResponse,
    operation: &str,
) -> Result<(), CliError> {
    release_prepared_http_response(ledger, response.prepared, operation)
}

#[allow(dead_code)]
fn release_prepared_mutation_pair(
    ledger: &mut codefire_core::DecisionWorkLedger,
    success: PreparedMutationResponse,
    durable: PreparedMutationResponse,
    first_error: &mut Option<CliError>,
) {
    record_cleanup_error(
        first_error,
        release_prepared_mutation_response(
            ledger,
            success,
            "release prepared mutation success response",
        ),
    );
    record_cleanup_error(
        first_error,
        release_prepared_mutation_response(
            ledger,
            durable,
            "release prepared mutation durable response",
        ),
    );
}

fn release_prepared_http_response(
    ledger: &mut codefire_core::DecisionWorkLedger,
    prepared: PreparedHttpResponse,
    operation: &str,
) -> Result<(), CliError> {
    let body_bytes = u64::try_from(prepared.body.len())
        .map_err(|_| CliError::InvalidRepository("HTTP response size overflow".to_string()))?;
    drop(prepared);
    ledger
        .release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            body_bytes,
        )
        .map_err(|error| upload_work_error(error, operation))
}

fn encode_http_json_with_ledger_internal(
    payload: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: Option<&HttpRequestDeadline>,
) -> Result<Vec<u8>, CliError> {
    encode_http_json_with_accounting(payload, ledger, deadline, true)
}

fn encode_http_json_with_accounting<T>(
    payload: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: Option<&HttpRequestDeadline>,
    charge_response_bytes: bool,
) -> Result<Vec<u8>, CliError>
where
    T: Serialize + ?Sized,
{
    let (serialized, accounting_error, body) = {
        let mut writer = AccountedHttpResponseWriter {
            body: Vec::new(),
            ledger,
            accounting_error: None,
            deadline,
            deadline_error: None,
            charge_response_bytes,
        };
        let serialized = serde_json::to_writer(&mut writer, payload);
        let error = writer
            .deadline_error
            .take()
            .map(Err)
            .unwrap_or(serialized.map_err(CliError::from));
        (error, writer.accounting_error.take(), writer.body)
    };
    if let Some(error) = accounting_error {
        release_encoded_http_response(ledger, body, "release partial HTTP response")?;
        return Err(upload_work_error(error, "encode HTTP response"));
    }
    if let Err(error) = serialized {
        release_encoded_http_response(ledger, body, "release partial HTTP response")?;
        return Err(error);
    }
    Ok(body)
}

struct AccountedHttpResponseWriter<'a> {
    body: Vec<u8>,
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
    deadline: Option<&'a HttpRequestDeadline>,
    deadline_error: Option<CliError>,
    charge_response_bytes: bool,
}

impl Write for AccountedHttpResponseWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(deadline) = self.deadline {
            if let Err(error) = deadline.check("response encoding") {
                self.deadline_error = Some(error);
                return Err(std::io::Error::other(
                    "HTTP response encoding deadline exceeded",
                ));
            }
        }
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("HTTP response size overflow"))?;
        let mut reservation = codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::Encoding,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount);
        if self.charge_response_bytes {
            reservation =
                reservation.charge(codefire_core::DecisionWorkDimension::ResponseBytes, amount);
        }
        if let Err(error) = self.ledger.apply(reservation) {
            self.accounting_error = Some(error);
            return Err(std::io::Error::other("HTTP response budget exceeded"));
        }
        self.body.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn encode_http_terminal_error(payload: &Value) -> Vec<u8> {
    let mut writer = BoundedJsonWriter::new(MAX_HTTP_ERROR_RESPONSE_BYTES);
    if serde_json::to_writer(&mut writer, payload).is_ok() {
        return writer.into_inner();
    }
    br#"{"error":"terminal HTTP request failure","exit_code":60}"#.to_vec()
}

fn http_json_header_len(status: u16, body_len: usize) -> Result<u64, CliError> {
    let reason = http_status_reason(status);
    let len = [
        "HTTP/1.1 ".len(),
        decimal_digits(usize::from(status)),
        " ".len(),
        reason.len(),
        "\r\nContent-Type: application/json\r\nContent-Length: ".len(),
        decimal_digits(body_len),
        "\r\nConnection: close\r\n\r\n".len(),
    ]
    .into_iter()
    .try_fold(0_usize, |total, part| total.checked_add(part))
    .ok_or_else(|| CliError::InvalidRepository("HTTP response header size overflow".to_string()))?;
    u64::try_from(len)
        .map_err(|_| CliError::InvalidRepository("HTTP response header size overflow".to_string()))
}

fn decimal_digits(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn http_error_response(error: &CliError) -> (u16, Value) {
    let store_budget = match error {
        CliError::Store(error) => match error.as_ref() {
            codefire_store::StoreError::DecisionWorkAccounting(accounting) => {
                accounting.budget_exceeded().cloned()
            }
            _ => None,
        },
        _ => None,
    };
    let status = if matches!(error, CliError::IdempotencyConflict(_)) {
        409
    } else if matches!(error, CliError::ResourceBudget(_)) || store_budget.is_some() {
        413
    } else if matches!(error, CliError::RequestDeadlineExceeded { .. }) {
        503
    } else {
        400
    };
    let diagnostic = store_budget
        .map(CliError::from)
        .as_ref()
        .map_or_else(|| error.diagnostic(), CliError::diagnostic);
    (
        status,
        json!({
            "error": error.to_string(),
            "exit_code": error.exit_code(),
            "kind": diagnostic.get("kind").cloned().unwrap_or(Value::Null),
            "diagnostics": [diagnostic],
        }),
    )
}

#[derive(Debug)]
struct HttpRequest {
    routing: HttpRequestRoutingOwnership,
    body: HttpRequestBodyOwnership,
}

/// Parsed request-line strings retained after the header scratch buffer is
/// released. Mutation dispatch releases this source owner before Store entry.
#[derive(Debug)]
struct HttpRequestRoutingOwnership {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    method: Option<String>,
    path: Option<String>,
    retained_bytes: Option<u64>,
}

impl HttpRequestRoutingOwnership {
    fn take(&mut self) -> Self {
        Self {
            ledger_identity: self.ledger_identity,
            method: self.method.take(),
            path: self.path.take(),
            retained_bytes: self.retained_bytes.take(),
        }
    }

    fn method(&self) -> &str {
        self.method
            .as_deref()
            .expect("HTTP request method remains available until explicit release")
    }

    fn path(&self) -> &str {
        self.path
            .as_deref()
            .expect("HTTP request path remains available until explicit release")
    }

    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "HTTP request routing belongs to a different decision-work ledger".to_string(),
            ));
        }
        drop(self.method.take());
        drop(self.path.take());
        let Some(retained_bytes) = self.retained_bytes else {
            return Ok(());
        };
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|error| upload_work_error(error, "release HTTP request routing"))?;
        self.retained_bytes = None;
        Ok(())
    }
}

/// Exact ownership of the parsed JSON body charged to one request ledger.
///
/// The body and its accounting amount deliberately share one owner. A foreign
/// ledger is rejected before either is touched. On the owning ledger the
/// `Value` is dropped before its scalar is released; if that release fails,
/// the amount remains available for an explicit retry without resurrecting
/// or double-dropping the payload.
#[must_use = "parsed HTTP request bodies must be released with their owning ledger"]
struct HttpRequestBodyOwnership {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    body: Option<Value>,
    retained_bytes: Option<u64>,
}

impl std::fmt::Debug for HttpRequestBodyOwnership {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpRequestBodyOwnership")
            .field("body_live", &self.body.is_some())
            .field("retained_bytes", &self.retained_bytes)
            .finish_non_exhaustive()
    }
}

impl HttpRequestBodyOwnership {
    fn new(body: Value, retained_bytes: u64, ledger: &codefire_core::DecisionWorkLedger) -> Self {
        Self {
            ledger_identity: ledger.identity(),
            body: Some(body),
            retained_bytes: Some(retained_bytes),
        }
    }

    fn value(&self) -> &Value {
        self.body
            .as_ref()
            .expect("HTTP request body remains available until explicit release")
    }

    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "HTTP request body belongs to a different decision-work ledger".to_string(),
            ));
        }
        drop(self.body.take());
        let Some(retained_bytes) = self.retained_bytes else {
            return Ok(());
        };
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|error| upload_work_error(error, "release HTTP request body"))?;
        self.retained_bytes = None;
        Ok(())
    }
}

/// Move-only owner of the canonical request and pre-reserved routing required
/// by the post-gate signature/nonce check and publication planner.
#[must_use = "an HTTP mutation route intent must be attached or explicitly released"]
struct HttpMutationRouteIntent {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    source_routing: Option<HttpRequestRoutingOwnership>,
    routing: Option<HttpMutationRoutingOwnership>,
    request: Option<Value>,
    request_retained_bytes: Option<u64>,
    derived_retained_bytes: u64,
    derived: HttpMutationDerivedIntent,
}

struct HttpUploadOverlayHandoffFailure {
    error: CliError,
    ledger: Box<codefire_core::DecisionWorkLedger>,
}

impl HttpUploadOverlayHandoffFailure {
    fn into_parts(self) -> (CliError, codefire_core::DecisionWorkLedger) {
        (self.error, *self.ledger)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HttpMutationRoutingKind {
    Upload,
    Apply,
}

/// Exact decoded routing values used to select one mutation. Its scalar is
/// acquired from raw component lengths before any destination String exists.
#[must_use = "HTTP mutation routing must be transferred or explicitly released"]
struct HttpMutationRoutingOwnership {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    kind: HttpMutationRoutingKind,
    org: Option<String>,
    app: Option<String>,
    selector: Option<String>,
    retained_bytes: Option<u64>,
}

impl HttpMutationRoutingOwnership {
    fn prepare(
        method: &str,
        path: &str,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Option<Self>, CliError> {
        if method != "POST" {
            return Ok(None);
        }
        let path = path.split_once('?').map_or(path, |(path, _)| path);
        let mut components = [None; 7];
        let mut count = 0_usize;
        for component in path.split('/').filter(|component| !component.is_empty()) {
            if count == components.len() {
                return Ok(None);
            }
            components[count] = Some(component);
            count += 1;
        }
        if count != components.len()
            || !decoded_http_component_eq(components[0].unwrap(), "v1")?
            || !decoded_http_component_eq(components[1].unwrap(), "projects")?
        {
            return Ok(None);
        }
        let kind = if decoded_http_component_eq(components[4].unwrap(), "branches")?
            && decoded_http_component_eq(components[6].unwrap(), "upload")?
        {
            HttpMutationRoutingKind::Upload
        } else if decoded_http_component_eq(components[4].unwrap(), "merge-requests")?
            && decoded_http_component_eq(components[6].unwrap(), "apply")?
        {
            HttpMutationRoutingKind::Apply
        } else {
            return Ok(None);
        };
        let org_source = components[2].unwrap();
        let app_source = components[3].unwrap();
        let selector_source = components[5].unwrap();
        let retained_bytes = sum_http_owner_bytes([
            logical_decoded_http_component_bytes(org_source)?,
            logical_decoded_http_component_bytes(app_source)?,
            logical_decoded_http_component_bytes(selector_source)?,
        ])?;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                ),
            )
            .map_err(|error| upload_work_error(error, "reserve HTTP mutation routing"))?;
        let materialized = (|| {
            Ok::<_, CliError>((
                decode_http_path_component(org_source)?,
                decode_http_path_component(app_source)?,
                decode_http_path_component(selector_source)?,
            ))
        })();
        let (org, app, selector) = match materialized {
            Ok(values) => values,
            Err(error) => {
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    )
                    .map_err(|release| {
                        upload_work_error(release, "release invalid HTTP mutation routing")
                    })?;
                return Err(error);
            }
        };
        Ok(Some(Self {
            ledger_identity: ledger.identity(),
            kind,
            org: Some(org),
            app: Some(app),
            selector: Some(selector),
            retained_bytes: Some(retained_bytes),
        }))
    }

    fn parts(&self) -> (&str, &str, &str) {
        (
            self.org.as_deref().expect("mutation routing owns org"),
            self.app.as_deref().expect("mutation routing owns app"),
            self.selector
                .as_deref()
                .expect("mutation routing owns route selector"),
        )
    }

    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation routing belongs to a different decision-work ledger".to_string(),
            ));
        }
        drop(self.org.take());
        drop(self.app.take());
        drop(self.selector.take());
        let Some(retained_bytes) = self.retained_bytes else {
            return Ok(());
        };
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|error| upload_work_error(error, "release HTTP mutation routing"))?;
        self.retained_bytes = None;
        Ok(())
    }
}

enum HttpMutationDerivedIntent {
    None,
    Upload(Box<HttpUploadDerivedIntent>),
    Apply(Box<HttpApplyDerivedIntent>),
}

struct HttpUploadDerivedIntent {
    project_root: PathBuf,
    remote_branch: String,
    head: String,
    local_branch: String,
    actor: String,
    request_target: String,
    active_project_root: Option<PathBuf>,
    objects: Option<PathBuf>,
    publication_request: Option<PublicationRequest>,
    response: Option<Value>,
}

struct HttpApplyDerivedIntent {
    project_root: PathBuf,
    mr_path: PathBuf,
    merge_request_id: String,
    actor: String,
    request_target: String,
    identity: Option<ApplyMutationIdentity>,
}

fn upload_store_structure_is_complete(
    routing_released: bool,
    upload: &HttpUploadDerivedIntent,
) -> bool {
    routing_released
        && upload.active_project_root.is_some()
        && upload.objects.is_some()
        && upload.publication_request.is_some()
        && upload.response.is_some()
}

fn apply_store_structure_is_complete(
    routing_released: bool,
    apply: &HttpApplyDerivedIntent,
) -> bool {
    routing_released && apply.identity.is_some()
}

#[derive(Serialize)]
struct AccountedUploadSemantic<'a> {
    stable_project_id: &'a str,
    remote_branch: &'a str,
    local_branch: &'a str,
    head: &'a str,
    object_set_digest: &'a str,
}

#[derive(Serialize)]
struct AccountedUploadResponse<'a> {
    branch: &'a str,
    head: &'a str,
}

#[derive(Serialize)]
struct AccountedApplySemantic<'a> {
    stable_project_id: &'a str,
    merge_request_id: &'a str,
    subject_digest: &'a str,
    target_branch: &'a str,
    source_head: &'a str,
    target_head_at_request: &'a str,
}

#[derive(Serialize)]
struct AccountedApplyResponse<'a> {
    id: &'a str,
    target_branch: &'a str,
    head: &'a str,
}

fn logical_string_owner_bytes(value: &str) -> Result<u64, CliError> {
    codefire_core::logical_retained_frame_bytes(value)
        .map_err(|error| upload_work_error(error, "measure HTTP mutation string owner"))
}

fn decoded_http_component_byte(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn decoded_http_component_len(value: &str) -> Result<usize, CliError> {
    let bytes = value.as_bytes();
    let mut index = 0_usize;
    let mut decoded = 0_usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hi = bytes
                .get(index + 1)
                .copied()
                .and_then(decoded_http_component_byte);
            let lo = bytes
                .get(index + 2)
                .copied()
                .and_then(decoded_http_component_byte);
            if hi.is_none() || lo.is_none() {
                return Err(CliError::Usage(
                    "invalid percent escape in HTTP path component".to_string(),
                ));
            }
            index += 3;
        } else {
            index += 1;
        }
        decoded = decoded.checked_add(1).ok_or_else(|| {
            CliError::InvalidRepository("HTTP route component size overflow".to_string())
        })?;
    }
    Ok(decoded)
}

fn decoded_http_component_eq(value: &str, expected: &str) -> Result<bool, CliError> {
    if decoded_http_component_len(value)? != expected.len() {
        return Ok(false);
    }
    let bytes = value.as_bytes();
    let expected = expected.as_bytes();
    let mut source_index = 0_usize;
    let mut expected_index = 0_usize;
    while source_index < bytes.len() {
        let decoded = if bytes[source_index] == b'%' {
            let hi = decoded_http_component_byte(bytes[source_index + 1]).unwrap();
            let lo = decoded_http_component_byte(bytes[source_index + 2]).unwrap();
            source_index += 3;
            hi << 4 | lo
        } else {
            let decoded = bytes[source_index];
            source_index += 1;
            decoded
        };
        if decoded != expected[expected_index] {
            return Ok(false);
        }
        expected_index += 1;
    }
    Ok(true)
}

fn logical_decoded_http_component_bytes(value: &str) -> Result<u64, CliError> {
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(
            u64::try_from(decoded_http_component_len(value)?).map_err(|_| {
                CliError::InvalidRepository("HTTP route component size overflow".to_string())
            })?,
        )
        .ok_or_else(|| {
            CliError::InvalidRepository("HTTP route component size overflow".to_string())
        })
}

fn logical_path_owner_bytes(value: &Path) -> Result<u64, CliError> {
    let bytes = u64::try_from(value.as_os_str().as_encoded_bytes().len())
        .map_err(|_| CliError::InvalidRepository("HTTP mutation path size overflow".to_string()))?;
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(bytes)
        .ok_or_else(|| CliError::InvalidRepository("HTTP mutation path size overflow".to_string()))
}

fn logical_joined_path_owner_bytes(parent: &Path, child: &str) -> Result<u64, CliError> {
    let parent = parent.as_os_str().as_encoded_bytes();
    let separator = u64::from(!parent.is_empty() && parent.last() != Some(&b'/'));
    sum_http_owner_bytes([
        codefire_core::LOGICAL_FRAME_BASE_BYTES,
        u64::try_from(parent.len()).map_err(|_| {
            CliError::InvalidRepository("HTTP mutation path size overflow".to_string())
        })?,
        separator,
        u64::try_from(child.len()).map_err(|_| {
            CliError::InvalidRepository("HTTP mutation path size overflow".to_string())
        })?,
    ])
}

fn http_merge_request_path_logical_bytes(
    project_root_logical_bytes: u64,
    mr_id: &str,
) -> Result<u64, CliError> {
    const DIRECT_STEM_BYTE_LIMIT: usize = 160;
    let file_name_bytes = if mr_id.len() <= DIRECT_STEM_BYTE_LIMIT {
        mr_id.len().checked_add(".json".len())
    } else {
        crate::remote::REMOTE_INTERNAL_HASH_PREFIX
            .len()
            .checked_add(64)
            .and_then(|length| length.checked_add(".json".len()))
    }
    .ok_or_else(|| CliError::InvalidRepository("HTTP apply MR path size overflow".to_string()))?;
    let separators = 2_u64;
    sum_http_owner_bytes([
        project_root_logical_bytes,
        separators,
        u64::try_from("merge_requests".len()).unwrap(),
        u64::try_from(file_name_bytes).map_err(|_| {
            CliError::InvalidRepository("HTTP apply MR path size overflow".to_string())
        })?,
    ])
}

fn sum_http_owner_bytes(values: impl IntoIterator<Item = u64>) -> Result<u64, CliError> {
    values.into_iter().try_fold(0_u64, |total, value| {
        total.checked_add(value).ok_or_else(|| {
            CliError::InvalidRepository("HTTP mutation ownership size overflow".to_string())
        })
    })
}

fn http_project_root_logical_bytes(
    storage_root: &Path,
    org: &str,
    app: &str,
) -> Result<u64, CliError> {
    let storage = storage_root.as_os_str().as_encoded_bytes();
    let mut path_bytes = u64::try_from(storage.len())
        .map_err(|_| CliError::InvalidRepository("HTTP project path size overflow".to_string()))?;
    let mut needs_separator = !storage.is_empty() && storage.last() != Some(&b'/');
    for component in [".codefire-server", "projects", org, app] {
        if needs_separator {
            path_bytes = path_bytes.checked_add(1).ok_or_else(|| {
                CliError::InvalidRepository("HTTP project path size overflow".to_string())
            })?;
        }
        path_bytes = path_bytes
            .checked_add(u64::try_from(component.len()).map_err(|_| {
                CliError::InvalidRepository("HTTP project path size overflow".to_string())
            })?)
            .ok_or_else(|| {
                CliError::InvalidRepository("HTTP project path size overflow".to_string())
            })?;
        needs_separator = true;
    }
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(path_bytes)
        .ok_or_else(|| CliError::InvalidRepository("HTTP project path size overflow".to_string()))
}

fn http_request_target_logical_bytes(
    org: &str,
    app: &str,
    operation: &str,
    branch: Option<&str>,
    mr_id: Option<&str>,
) -> Result<u64, CliError> {
    let suffix = match operation {
        "upload" => sum_http_owner_bytes([
            u64::try_from("/branches/".len()).unwrap(),
            u64::try_from(branch.unwrap_or_default().len()).map_err(|_| {
                CliError::InvalidRepository("HTTP request target size overflow".to_string())
            })?,
            u64::try_from("/upload".len()).unwrap(),
        ])?,
        "apply" => sum_http_owner_bytes([
            u64::try_from("/merge-requests/".len()).unwrap(),
            u64::try_from(mr_id.unwrap_or_default().len()).map_err(|_| {
                CliError::InvalidRepository("HTTP request target size overflow".to_string())
            })?,
            u64::try_from("/apply".len()).unwrap(),
        ])?,
        _ => {
            return Err(CliError::InvalidRepository(
                "unsupported HTTP mutation request-target accounting".to_string(),
            ))
        }
    };
    let bytes = sum_http_owner_bytes([
        u64::try_from(org.len()).map_err(|_| {
            CliError::InvalidRepository("HTTP request target size overflow".to_string())
        })?,
        1,
        u64::try_from(app.len()).map_err(|_| {
            CliError::InvalidRepository("HTTP request target size overflow".to_string())
        })?,
        suffix,
    ])?;
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(bytes)
        .ok_or_else(|| CliError::InvalidRepository("HTTP request target size overflow".to_string()))
}

impl std::fmt::Debug for HttpMutationRouteIntent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpMutationRouteIntent")
            .field("request_live", &self.request.is_some())
            .field("request_retained_bytes", &self.request_retained_bytes)
            .field("derived_retained_bytes", &self.derived_retained_bytes)
            .finish_non_exhaustive()
    }
}

impl HttpMutationRouteIntent {
    #[cfg(test)]
    fn request(&self) -> &Value {
        self.request
            .as_ref()
            .expect("HTTP mutation route intent retains its request")
    }

    fn matches_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        ledger.matches_identity(self.ledger_identity)
    }

    fn prepare_upload_overlay_input(
        &mut self,
        mut ledger: codefire_core::DecisionWorkLedger,
        document_bytes: u64,
        deadline: &HttpRequestDeadline,
    ) -> Result<codefire_store::SealedCommitOverlayValidationInput, HttpUploadOverlayHandoffFailure>
    {
        let fail = |error, ledger| HttpUploadOverlayHandoffFailure {
            error,
            ledger: Box::new(ledger),
        };
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(fail(
                CliError::InvalidRepository(
                    "HTTP upload object handoff belongs to a different decision-work ledger"
                        .to_string(),
                ),
                ledger,
            ));
        }
        if let Err(error) = deadline.check("upload overlay destination reservation") {
            return Err(fail(error, ledger));
        }
        let request = self
            .request
            .as_mut()
            .expect("HTTP upload object handoff retains its canonical request");
        let records = match request.get_mut("objects").and_then(Value::as_array_mut) {
            Some(records) => records,
            None => {
                return Err(fail(
                    CliError::Usage("upload request must include objects".to_string()),
                    ledger,
                ));
            }
        };
        let separator_bytes = match u64::try_from(records.len().saturating_sub(1)) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(fail(
                    CliError::InvalidRepository(
                        "HTTP upload object separator size overflow".to_string(),
                    ),
                    ledger,
                ));
            }
        };
        let transferred_bytes = match document_bytes.checked_add(separator_bytes) {
            Some(bytes) => bytes,
            None => {
                return Err(fail(
                    CliError::InvalidRepository(
                        "HTTP upload object transfer size overflow".to_string(),
                    ),
                    ledger,
                ));
            }
        };
        let request_retained_bytes = match self.request_retained_bytes {
            Some(bytes) => bytes,
            None => {
                return Err(fail(
                    CliError::InvalidRepository(
                        "HTTP upload object handoff lost request ownership".to_string(),
                    ),
                    ledger,
                ));
            }
        };
        let remaining_request_bytes = match request_retained_bytes.checked_sub(transferred_bytes) {
            Some(bytes) => bytes,
            None => {
                return Err(fail(
                    CliError::InvalidRepository(
                        "HTTP upload object transfer exceeds request ownership".to_string(),
                    ),
                    ledger,
                ));
            }
        };

        let destination_slot_bytes = u64::try_from(records.len())
            .ok()
            .and_then(|count| {
                count.checked_mul(
                    u64::try_from(std::mem::size_of::<codefire_store::ObjectRecordDocument>())
                        .ok()?,
                )
            })
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP upload overlay destination size overflow".to_string(),
                )
            });
        let destination_slot_bytes = match destination_slot_bytes {
            Ok(bytes) => bytes,
            Err(error) => return Err(fail(error, ledger)),
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                destination_slot_bytes,
            ),
        ) {
            return Err(fail(
                upload_work_error(error, "reserve upload overlay destination slots"),
                ledger,
            ));
        }
        let mut documents = Vec::new();
        if let Err(error) = documents.try_reserve_exact(records.len()) {
            if let Err(release) = ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                destination_slot_bytes,
            ) {
                return Err(fail(
                    upload_work_error(release, "release rejected upload overlay destination slots"),
                    ledger,
                ));
            }
            return Err(fail(
                CliError::Io(std::io::Error::other(format!(
                    "HTTP upload overlay destination allocation failed: {error}"
                ))),
                ledger,
            ));
        }
        let records = std::mem::take(records);
        let conversion = (|| -> Result<(), CliError> {
            for record in records {
                deadline.check("upload overlay move")?;
                documents.push(codefire_store::object_record_document_from_value(record)?);
            }
            Ok(())
        })();
        if let Err(error) = conversion {
            drop(documents);
            let destination_release = ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                destination_slot_bytes,
            );
            let payload_release = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                transferred_bytes,
            );
            if let Err(release) = destination_release.and(payload_release) {
                return Err(fail(
                    upload_work_error(release, "release failed upload overlay move"),
                    ledger,
                ));
            }
            self.request_retained_bytes = Some(remaining_request_bytes);
            return Err(fail(error, ledger));
        }
        if let Err(error) = ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            destination_slot_bytes,
        ) {
            drop(documents);
            if let Err(release) = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                transferred_bytes,
            ) {
                return Err(fail(
                    upload_work_error(release, "release failed upload overlay move payload"),
                    ledger,
                ));
            }
            self.request_retained_bytes = Some(remaining_request_bytes);
            return Err(fail(
                upload_work_error(error, "release upload overlay destination slots"),
                ledger,
            ));
        }

        self.request_retained_bytes = Some(remaining_request_bytes);
        match codefire_store::prepare_sealed_commit_overlay_validation_input_with_cancellation_and_retained_transfer(
            documents,
            ledger,
            transferred_bytes,
            || deadline.is_expired(),
        ) {
            Ok(input) => Ok(input),
            Err(error) => {
                let (error, ledger) = error.discard_documents_and_recover_ledger();
                Err(fail(CliError::from(error), ledger))
            }
        }
    }

    fn upload_request_after_object_handoff(&self) -> Result<&Value, CliError> {
        let request = self.request.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP upload object handoff lost its canonical request".to_string(),
            )
        })?;
        if !request
            .get("objects")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        {
            return Err(CliError::InvalidRepository(
                "HTTP upload object handoff did not consume its object array".to_string(),
            ));
        }
        Ok(request)
    }

    fn acquire_derived_retained_parts(
        ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
        derived_retained_bytes: &mut u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        retained_bytes: u64,
        operation: &str,
    ) -> Result<(), CliError> {
        if !ledger.matches_identity(ledger_identity) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation derived intent belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        let next = derived_retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP mutation derived ownership size overflow".to_string(),
                )
            })?;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                ),
            )
            .map_err(|error| upload_work_error(error, operation))?;
        *derived_retained_bytes = next;
        Ok(())
    }

    fn prepare_upload_initial_from_routing(
        &mut self,
        storage_root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        let mut routing = self.routing.take().ok_or_else(|| {
            CliError::InvalidRepository("HTTP upload route is missing typed routing".to_string())
        })?;
        if routing.kind != HttpMutationRoutingKind::Upload {
            self.routing = Some(routing);
            return Err(CliError::InvalidRepository(
                "HTTP upload route has mismatched typed routing".to_string(),
            ));
        }
        let result = {
            let (org, app, branch_name) = routing.parts();
            self.prepare_upload_initial(storage_root, org, app, branch_name, ledger)
        };
        if let Err(error) = result {
            self.routing = Some(routing);
            return Err(error);
        }
        if let Err(error) = routing.release(ledger) {
            self.routing = Some(routing);
            return Err(error);
        }
        let mut source_routing = self.source_routing.take().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP upload route is missing parser routing ownership".to_string(),
            )
        })?;
        if let Err(error) = source_routing.release(ledger) {
            self.source_routing = Some(source_routing);
            return Err(error);
        }
        Ok(())
    }

    fn prepare_apply_initial_from_routing(
        &mut self,
        storage_root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        let mut routing = self.routing.take().ok_or_else(|| {
            CliError::InvalidRepository("HTTP apply route is missing typed routing".to_string())
        })?;
        if routing.kind != HttpMutationRoutingKind::Apply {
            self.routing = Some(routing);
            return Err(CliError::InvalidRepository(
                "HTTP apply route has mismatched typed routing".to_string(),
            ));
        }
        let result = {
            let (org, app, mr_id) = routing.parts();
            self.prepare_apply_initial(storage_root, org, app, mr_id, ledger)
        };
        if let Err(error) = result {
            self.routing = Some(routing);
            return Err(error);
        }
        if let Err(error) = routing.release(ledger) {
            self.routing = Some(routing);
            return Err(error);
        }
        let mut source_routing = self.source_routing.take().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP apply route is missing parser routing ownership".to_string(),
            )
        })?;
        if let Err(error) = source_routing.release(ledger) {
            self.source_routing = Some(source_routing);
            return Err(error);
        }
        Ok(())
    }

    fn prepare_upload_initial(
        &mut self,
        storage_root: &Path,
        org: &str,
        app: &str,
        branch_name: &str,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        if !matches!(self.derived, HttpMutationDerivedIntent::None) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation derived intent was already selected".to_string(),
            ));
        }
        let Self {
            ledger_identity,
            request,
            derived_retained_bytes,
            derived,
            ..
        } = self;
        let request = request
            .as_ref()
            .expect("HTTP mutation route intent retains its request");
        let head = request
            .get("head")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidMarker("missing head".to_string()))?;
        let local_branch = request
            .get("branch")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let actor = request
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("local");
        let retained_bytes = sum_http_owner_bytes([
            http_project_root_logical_bytes(storage_root, org, app)?,
            logical_string_owner_bytes(branch_name)?,
            logical_string_owner_bytes(head)?,
            logical_string_owner_bytes(local_branch)?,
            logical_string_owner_bytes(actor)?,
            http_request_target_logical_bytes(org, app, "upload", Some(branch_name), None)?,
        ])?;
        Self::acquire_derived_retained_parts(
            *ledger_identity,
            derived_retained_bytes,
            ledger,
            retained_bytes,
            "reserve HTTP upload initial derived ownership",
        )?;
        *derived = HttpMutationDerivedIntent::Upload(Box::new(HttpUploadDerivedIntent {
            project_root: http_project_root(storage_root, org, app),
            remote_branch: branch_name.to_string(),
            head: head.to_string(),
            local_branch: local_branch.to_string(),
            actor: actor.to_string(),
            request_target: super::signatures::remote_request_target(
                org,
                app,
                "upload",
                Some(branch_name),
                None,
            ),
            active_project_root: None,
            objects: None,
            publication_request: None,
            response: None,
        }));
        Ok(())
    }

    fn prepare_apply_initial(
        &mut self,
        storage_root: &Path,
        org: &str,
        app: &str,
        mr_id: &str,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        if !matches!(self.derived, HttpMutationDerivedIntent::None) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation derived intent was already selected".to_string(),
            ));
        }
        let Self {
            ledger_identity,
            request,
            derived_retained_bytes,
            derived,
            ..
        } = self;
        let actor = request
            .as_ref()
            .expect("HTTP mutation route intent retains its request")
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("local");
        let project_root_bytes = http_project_root_logical_bytes(storage_root, org, app)?;
        let mr_path_bytes = http_merge_request_path_logical_bytes(project_root_bytes, mr_id)?;
        let retained_bytes = sum_http_owner_bytes([
            project_root_bytes,
            mr_path_bytes,
            logical_string_owner_bytes(mr_id)?,
            logical_string_owner_bytes(actor)?,
            http_request_target_logical_bytes(org, app, "apply", None, Some(mr_id))?,
        ])?;
        Self::acquire_derived_retained_parts(
            *ledger_identity,
            derived_retained_bytes,
            ledger,
            retained_bytes,
            "reserve HTTP apply initial derived ownership",
        )?;
        let project_root = http_project_root(storage_root, org, app);
        *derived = HttpMutationDerivedIntent::Apply(Box::new(HttpApplyDerivedIntent {
            mr_path: crate::remote::remote_merge_request_path(&project_root, mr_id),
            project_root,
            merge_request_id: mr_id.to_string(),
            actor: actor.to_string(),
            request_target: super::signatures::remote_request_target(
                org,
                app,
                "apply",
                None,
                Some(mr_id),
            ),
            identity: None,
        }));
        Ok(())
    }

    fn upload(&self) -> &HttpUploadDerivedIntent {
        match &self.derived {
            HttpMutationDerivedIntent::Upload(upload) => upload,
            _ => panic!("HTTP upload route requires upload-derived ownership"),
        }
    }

    fn apply(&self) -> &HttpApplyDerivedIntent {
        match &self.derived {
            HttpMutationDerivedIntent::Apply(apply) => apply,
            _ => panic!("HTTP apply route requires apply-derived ownership"),
        }
    }

    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if !self.matches_ledger(ledger) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation route intent belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        if let Some(source_routing) = self.source_routing.as_mut() {
            source_routing.release(ledger)?;
        }
        self.source_routing = None;
        if let Some(routing) = self.routing.as_mut() {
            routing.release(ledger)?;
        }
        self.routing = None;
        self.derived = HttpMutationDerivedIntent::None;
        drop(self.request.take());
        let Some(request_retained_bytes) = self.request_retained_bytes else {
            return Ok(());
        };
        let retained_bytes = request_retained_bytes
            .checked_add(self.derived_retained_bytes)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP mutation route ownership size overflow".to_string(),
                )
            })?;
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|error| upload_work_error(error, "release HTTP mutation route intent"))?;
        self.request_retained_bytes = None;
        self.derived_retained_bytes = 0;
        Ok(())
    }
}

impl HttpRequest {
    fn method(&self) -> &str {
        self.routing.method()
    }

    fn path(&self) -> &str {
        self.routing.path()
    }

    fn body(&self) -> &Value {
        self.body.value()
    }

    fn transfer_mutation_intent(
        &mut self,
        routing: &mut Option<HttpMutationRoutingOwnership>,
        ledger: &codefire_core::DecisionWorkLedger,
        deadline: &HttpRequestDeadline,
    ) -> Result<HttpMutationRouteIntent, CliError> {
        if !ledger.matches_identity(self.body.ledger_identity)
            || !ledger.matches_identity(self.routing.ledger_identity)
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation transfer belongs to a different decision-work ledger".to_string(),
            ));
        }
        deadline.check("HTTP mutation intent transfer")?;
        let request_retained_bytes = self.body.retained_bytes.ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP mutation transfer requires parsed request accounting".to_string(),
            )
        })?;
        let request = self.body.body.take().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP mutation transfer requires a live parsed request body".to_string(),
            )
        })?;
        self.body.retained_bytes = None;
        Ok(HttpMutationRouteIntent {
            ledger_identity: ledger.identity(),
            source_routing: Some(self.routing.take()),
            routing: routing.take(),
            request: Some(request),
            request_retained_bytes: Some(request_retained_bytes),
            derived_retained_bytes: 0,
            derived: HttpMutationDerivedIntent::None,
        })
    }

    fn release_routing(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.routing.release(ledger)
    }

    fn release_body(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.body.release(ledger)
    }
}

struct PreparedHttpResponse {
    status: u16,
    body: Vec<u8>,
}

#[allow(dead_code)]
struct PreparedMutationResponse {
    prepared: PreparedHttpResponse,
    response_work: Option<codefire_core::DecisionWorkReservation>,
    header_work: Option<codefire_core::DecisionWorkReservation>,
    body_bytes: u64,
    header_bytes: u64,
}

/// Two pre-encoded terminal responses for a mutation whose durable reducer may
/// complete without returning its ordinary success result. The raw owner does
/// not borrow the request ledger, so it can coexist with Store/publication
/// prepared owners until their shared prepublication seal.
#[must_use = "prepared mutation responses must be selected or explicitly cancelled"]
#[allow(dead_code)]
struct PreparedMutationResponses {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    success: Option<PreparedMutationResponse>,
    durable: Option<PreparedMutationResponse>,
    response_permit: Option<codefire_core::DecisionWorkPermit>,
    header_permit: Option<codefire_core::DecisionWorkPermit>,
    response_upper: u64,
    header_upper: u64,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PreparedMutationResponseChoice {
    Success,
    Durable,
}

#[allow(dead_code)]
impl PreparedMutationResponses {
    fn select_success(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        prepared: &mut Option<PreparedHttpResponse>,
    ) -> Result<(), CliError> {
        self.select(PreparedMutationResponseChoice::Success, ledger, prepared)
    }

    fn select_durable(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        prepared: &mut Option<PreparedHttpResponse>,
    ) -> Result<(), CliError> {
        self.select(PreparedMutationResponseChoice::Durable, ledger, prepared)
    }

    fn select(
        &mut self,
        choice: PreparedMutationResponseChoice,
        ledger: &mut codefire_core::DecisionWorkLedger,
        prepared: &mut Option<PreparedHttpResponse>,
    ) -> Result<(), CliError> {
        let validation = if !ledger.matches_identity(self.ledger_identity) {
            Err(CliError::InvalidRepository(
                "prepared mutation responses belong to a different decision-work ledger"
                    .to_string(),
            ))
        } else if !ledger.is_prepublication_sealed() {
            Err(CliError::InvalidRepository(
                "prepared mutation response selection requires a sealed prepublication ledger"
                    .to_string(),
            ))
        } else if prepared.is_some() {
            Err(CliError::InvalidRepository(
                "HTTP route already owns a prepared response".to_string(),
            ))
        } else {
            Ok(())
        };
        validation?;

        let (response_work, header_work, selected_bytes, selected_headers) = match choice {
            PreparedMutationResponseChoice::Success => {
                let selected = self
                    .success
                    .as_mut()
                    .expect("prepared mutation success response must be present");
                (
                    selected
                        .response_work
                        .take()
                        .expect("prepared success response work must be present"),
                    selected
                        .header_work
                        .take()
                        .expect("prepared success header work must be present"),
                    selected.body_bytes,
                    selected.header_bytes,
                )
            }
            PreparedMutationResponseChoice::Durable => {
                let selected = self
                    .durable
                    .as_mut()
                    .expect("prepared mutation durable response must be present");
                (
                    selected
                        .response_work
                        .take()
                        .expect("prepared durable response work must be present"),
                    selected
                        .header_work
                        .take()
                        .expect("prepared durable header work must be present"),
                    selected.body_bytes,
                    selected.header_bytes,
                )
            }
        };
        self.consume_selected_work(
            ledger,
            response_work,
            header_work,
            selected_bytes,
            selected_headers,
        )?;

        let selected = match choice {
            PreparedMutationResponseChoice::Success => {
                let unused = self
                    .durable
                    .take()
                    .expect("prepared mutation durable response must be present");
                release_prepared_http_response(
                    ledger,
                    unused.prepared,
                    "release unselected durable mutation response",
                )?;
                self.success
                    .take()
                    .expect("prepared mutation success response must be present")
                    .prepared
            }
            PreparedMutationResponseChoice::Durable => {
                let unused = self
                    .success
                    .take()
                    .expect("prepared mutation success response must be present");
                release_prepared_http_response(
                    ledger,
                    unused.prepared,
                    "release unselected success mutation response",
                )?;
                self.durable
                    .take()
                    .expect("prepared mutation durable response must be present")
                    .prepared
            }
        };
        *prepared = Some(selected);
        Ok(())
    }

    fn consume_selected_work(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        response_work: codefire_core::DecisionWorkReservation,
        header_work: codefire_core::DecisionWorkReservation,
        response_bytes: u64,
        header_bytes: u64,
    ) -> Result<(), CliError> {
        let response_permit = self.response_permit.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "prepared mutation response-byte permit is missing".to_string(),
            )
        })?;
        let header_permit = self.header_permit.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "prepared mutation response-header permit is missing".to_string(),
            )
        })?;
        ledger
            .consume_reserved_pair(response_permit, response_work, header_permit, header_work)
            .map_err(|error| {
                upload_work_error(error, "consume prepared mutation response and header work")
            })?;
        if response_bytes < self.response_upper {
            ledger.cancel_reserved(response_permit).map_err(|error| {
                upload_work_error(error, "cancel unused mutation response-byte work")
            })?;
        }
        if header_bytes < self.header_upper {
            ledger.cancel_reserved(header_permit).map_err(|error| {
                upload_work_error(error, "cancel unused mutation response-header work")
            })?;
        }
        self.response_permit = None;
        self.header_permit = None;
        Ok(())
    }

    fn cancel(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "prepared mutation responses belong to a different decision-work ledger"
                    .to_string(),
            ));
        }
        let mut first_error = None;
        if let Some(permit) = self.response_permit.take() {
            record_cleanup_error(
                &mut first_error,
                ledger.cancel_reserved(&permit).map_err(|error| {
                    upload_work_error(error, "cancel prepared mutation response-byte permit")
                }),
            );
        }
        if let Some(permit) = self.header_permit.take() {
            record_cleanup_error(
                &mut first_error,
                ledger.cancel_reserved(&permit).map_err(|error| {
                    upload_work_error(error, "cancel prepared mutation response-header permit")
                }),
            );
        }
        if let Some(success) = self.success.take() {
            record_cleanup_error(
                &mut first_error,
                release_prepared_mutation_response(
                    ledger,
                    success,
                    "release cancelled mutation success response",
                ),
            );
        }
        if let Some(durable) = self.durable.take() {
            record_cleanup_error(
                &mut first_error,
                release_prepared_mutation_response(
                    ledger,
                    durable,
                    "release cancelled mutation durable response",
                ),
            );
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for PreparedMutationResponses {
    fn drop(&mut self) {
        debug_assert!(
            self.success.is_none()
                && self.durable.is_none()
                && self.response_permit.is_none()
                && self.header_permit.is_none(),
            "raw prepared mutation responses were dropped without selection or cancellation"
        );
    }
}

/// Borrowing convenience wrapper for simple mutation paths and focused tests.
/// Combined Store/publication paths retain the raw owner above instead.
#[allow(dead_code)]
struct PreparedMutationResponsesOwner<'a> {
    raw: &'a mut PreparedMutationResponses,
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    prepared: &'a mut Option<PreparedHttpResponse>,
}

#[allow(dead_code)]
impl<'a> PreparedMutationResponsesOwner<'a> {
    fn new(
        raw: &'a mut PreparedMutationResponses,
        ledger: &'a mut codefire_core::DecisionWorkLedger,
        prepared: &'a mut Option<PreparedHttpResponse>,
    ) -> Result<Self, CliError> {
        if !ledger.matches_identity(raw.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "prepared mutation response owner requires its originating decision-work ledger"
                    .to_string(),
            ));
        }
        Ok(Self {
            raw,
            ledger,
            prepared,
        })
    }

    fn select_success(self) -> Result<(), CliError> {
        self.raw.select_success(self.ledger, self.prepared)
    }

    fn select_durable(self) -> Result<(), CliError> {
        self.raw.select_durable(self.ledger, self.prepared)
    }

    fn cancel(self) -> Result<(), CliError> {
        self.raw.cancel(self.ledger)
    }
}

impl Drop for PreparedMutationResponsesOwner<'_> {
    fn drop(&mut self) {
        let cancelled = self.raw.cancel(self.ledger);
        debug_assert!(cancelled.is_ok());
    }
}

/// Request-wide owner for the prepublication half of an HTTP mutation.
///
/// The Store binding is deliberately not a field of the completed owner. It
/// remains private inside `BoundPrepublicationValidationSession` until the
/// constructor atomically turns it into the detached proof and the command
/// ledger. Every other raw prepared owner is accepted only after a
/// side-effect-free identity check against that ledger.
#[must_use = "a prepared HTTP mutation must be committed or explicitly cancelled"]
#[allow(dead_code)]
struct PreparedHttpMutationTransaction<'project> {
    ledger: Option<codefire_core::DecisionWorkLedger>,
    route_intent: Option<HttpMutationRouteIntent>,
    validation_proof: Option<codefire_store::PrepublicationSealedValidationProof>,
    object_batch: Option<codefire_store::PreparedObjectRecordBatch>,
    publication: Option<PreparedJsonPublication<'project>>,
    responses: Option<PreparedMutationResponses>,
    prepared_response: Option<PreparedHttpResponse>,
    requires_validation_proof: bool,
    requires_publication: bool,
    requires_responses: bool,
    sealed: bool,
    cancellation_complete: bool,
}

#[must_use = "a prepared HTTP mutation terminal must be handed off or explicitly cancelled"]
struct PreparedHttpMutationTerminal {
    ledger: Option<codefire_core::DecisionWorkLedger>,
    responses: Option<PreparedMutationResponses>,
    prepared_response: Option<PreparedHttpResponse>,
}

/// Proof produced by the final pre-rename callback. Its private construction
/// prevents a visible bootstrap from being handed to the HTTP writer without
/// first validating the exact retained response ownership.
struct ValidatedHttpBootstrapHandoffProof;

/// Post-rename HTTP handoff. Construction consumes the callback proof, so its
/// only remaining operation is moving the already-validated ledger/response
/// pair to the connection owner.
struct ValidatedHttpBootstrapHandoff {
    terminal: PreparedHttpMutationTerminal,
}

struct PreparedHttpMutationValidationBuildFailure {
    cause: CliError,
    bound: Option<codefire_store::BoundPrepublicationValidationSession>,
}

impl std::fmt::Debug for PreparedHttpMutationValidationBuildFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedHttpMutationValidationBuildFailure")
            .field("cause", &self.cause.to_string())
            .finish_non_exhaustive()
    }
}

impl PreparedHttpMutationValidationBuildFailure {
    fn cancel_and_take_ledger(
        mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) -> Result<
        (CliError, codefire_core::DecisionWorkLedger),
        (
            CliError,
            codefire_store::BoundPrepublicationValidationSessionCancellationError,
        ),
    > {
        let bound = self
            .bound
            .take()
            .expect("validation build failure retains its bound session");
        match bound.cancel_and_take_decision_work_ledger(reason) {
            Ok(ledger) => Ok((self.cause, ledger)),
            Err(error) => Err((self.cause, error)),
        }
    }

    fn cancel_and_take_unfinished_ledger(
        mut self,
    ) -> Result<
        (CliError, codefire_core::DecisionWorkLedger),
        (
            CliError,
            codefire_store::BoundPrepublicationValidationSessionCancellationError,
        ),
    > {
        let bound = self
            .bound
            .take()
            .expect("validation build failure retains its bound session");
        match bound.cancel_and_take_decision_work_ledger_unfinished() {
            Ok(ledger) => Ok((self.cause, ledger)),
            Err(error) => Err((self.cause, error)),
        }
    }
}

#[allow(dead_code)]
impl<'project> PreparedHttpMutationTransaction<'project> {
    fn from_bound_validation(
        bound: codefire_store::BoundPrepublicationValidationSession,
    ) -> Result<Self, PreparedHttpMutationValidationBuildFailure> {
        let (proof, ledger) = match bound.into_prepublication_validation_proof() {
            Ok(detached) => detached,
            Err(error) => {
                let (error, bound) = error.into_parts();
                return Err(PreparedHttpMutationValidationBuildFailure {
                    cause: http_store_error(error),
                    bound: Some(bound),
                });
            }
        };
        debug_assert!(proof.matches_ledger(&ledger));
        Ok(Self {
            ledger: Some(ledger),
            route_intent: None,
            validation_proof: Some(proof),
            object_batch: None,
            publication: None,
            responses: None,
            prepared_response: None,
            requires_validation_proof: true,
            requires_publication: true,
            requires_responses: true,
            sealed: false,
            cancellation_complete: false,
        })
    }

    fn plan_publication<F>(
        &mut self,
        gate: &'project RemoteProjectGate,
        project_root: &Path,
        request: &PublicationRequest,
        nonce: &mut Option<crate::signatures::PendingRequestNonce>,
        result: &Value,
        build_business_writes: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(
            &codefire_store::ProvisionalSealedValidationSubject<'_>,
            &mut codefire_core::DecisionWorkLedger,
        ) -> Result<Vec<AuthorityWrite>, CliError>,
    {
        if self.publication.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns a JSON publication".to_string(),
            ));
        }
        let proof = self.validation_proof.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP mutation lost its prepublication validation proof".to_string(),
            )
        })?;
        let ledger = self
            .ledger
            .as_mut()
            .expect("prepared HTTP mutation retains its ledger");
        let subject = proof
            .provisional_subject_for_planning(ledger)
            .map_err(http_store_error)?;
        let nonce = nonce.take().ok_or_else(|| {
            CliError::InvalidRepository("HTTP mutation request nonce is missing".to_string())
        })?;
        let publication = prepare_json_publication_plan_lazy_with_nonce_and_provisional_roots(
            gate,
            project_root,
            request,
            nonce,
            result,
            &subject,
            ledger,
            |ledger| build_business_writes(&subject, ledger),
        )?;
        debug_assert!(proof.matches_ledger(ledger));
        debug_assert!(publication.matches_ledger(ledger));
        self.publication = Some(publication);
        self.cancellation_complete = false;
        Ok(())
    }

    #[cfg(test)]
    fn from_ledger_for_test(ledger: codefire_core::DecisionWorkLedger) -> Self {
        Self {
            ledger: Some(ledger),
            route_intent: None,
            validation_proof: None,
            object_batch: None,
            publication: None,
            responses: None,
            prepared_response: None,
            requires_validation_proof: false,
            requires_publication: false,
            requires_responses: false,
            sealed: false,
            cancellation_complete: false,
        }
    }

    fn ledger(&self) -> &codefire_core::DecisionWorkLedger {
        self.ledger
            .as_ref()
            .expect("prepared HTTP mutation retains its ledger")
    }

    fn attach_route_intent(
        &mut self,
        candidate: &mut Option<HttpMutationRouteIntent>,
    ) -> Result<(), CliError> {
        if self.route_intent.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns a route intent".to_string(),
            ));
        }
        let intent = candidate.as_ref().ok_or_else(|| {
            CliError::InvalidRepository("HTTP mutation route intent is missing".to_string())
        })?;
        if !intent.matches_ledger(self.ledger()) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation route intent belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        self.route_intent = candidate.take();
        self.cancellation_complete = false;
        Ok(())
    }

    fn release_route_intent(&mut self) -> Result<(), CliError> {
        let Some(mut intent) = self.route_intent.take() else {
            return Ok(());
        };
        let result = intent.release(
            self.ledger
                .as_mut()
                .expect("HTTP mutation retains its route-intent ledger"),
        );
        if let Err(error) = result {
            self.route_intent = Some(intent);
            return Err(error);
        }
        Ok(())
    }

    fn attach_object_batch(
        &mut self,
        candidate: &mut Option<codefire_store::PreparedObjectRecordBatch>,
    ) -> Result<(), CliError> {
        if self.object_batch.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns an object batch".to_string(),
            ));
        }
        let Some(batch) = candidate.as_ref() else {
            return Err(CliError::InvalidRepository(
                "HTTP mutation object batch is missing".to_string(),
            ));
        };
        if !batch.matches_ledger(self.ledger()) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation object batch belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        self.object_batch = candidate.take();
        self.cancellation_complete = false;
        Ok(())
    }

    fn prepare_and_attach_overlay_object_batch(
        &mut self,
        deadline: &HttpRequestDeadline,
    ) -> Result<(), CliError> {
        if self.object_batch.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns an object batch".to_string(),
            ));
        }
        let proof = self.validation_proof.as_mut().ok_or_else(|| {
            CliError::InvalidRepository("HTTP upload lost its overlay validation proof".to_string())
        })?;
        let batch = proof
            .prepare_overlay_object_batch(
                self.ledger
                    .as_mut()
                    .expect("HTTP upload retains its overlay batch ledger"),
                || deadline.is_expired(),
            )
            .map_err(http_store_error)?;
        debug_assert!(batch.matches_ledger(
            self.ledger
                .as_ref()
                .expect("HTTP upload retains its overlay batch ledger")
        ));
        self.object_batch = Some(batch);
        self.cancellation_complete = false;
        Ok(())
    }

    fn attach_publication(
        &mut self,
        candidate: &mut Option<PreparedJsonPublication<'project>>,
    ) -> Result<(), CliError> {
        if self.publication.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns a JSON publication".to_string(),
            ));
        }
        let Some(publication) = candidate.as_ref() else {
            return Err(CliError::InvalidRepository(
                "HTTP mutation JSON publication is missing".to_string(),
            ));
        };
        if !publication.matches_ledger(self.ledger()) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation JSON publication belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        self.publication = candidate.take();
        self.cancellation_complete = false;
        Ok(())
    }

    fn attach_responses(
        &mut self,
        candidate: &mut Option<PreparedMutationResponses>,
    ) -> Result<(), CliError> {
        if self.responses.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns terminal responses".to_string(),
            ));
        }
        let Some(responses) = candidate.as_ref() else {
            return Err(CliError::InvalidRepository(
                "HTTP mutation terminal responses are missing".to_string(),
            ));
        };
        if !self.ledger().matches_identity(responses.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "HTTP mutation terminal responses belong to a different decision-work ledger"
                    .to_string(),
            ));
        }
        self.responses = candidate.take();
        self.cancellation_complete = false;
        Ok(())
    }

    fn prepare_and_attach_responses(
        &mut self,
        success_status: u16,
        success_payload: &Value,
        deadline: &HttpRequestDeadline,
    ) -> Result<(), CliError> {
        if self.responses.is_some() || self.prepared_response.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation already owns terminal responses".to_string(),
            ));
        }
        let responses = prepare_mutation_responses_with_deadline(
            success_status,
            success_payload,
            self.ledger
                .as_mut()
                .expect("prepared HTTP mutation retains its ledger"),
            deadline,
        )?;
        debug_assert!(self.ledger().matches_identity(responses.ledger_identity));
        self.responses = Some(responses);
        self.cancellation_complete = false;
        Ok(())
    }

    fn prepare_and_attach_responses_from_upload_route(
        &mut self,
        success_status: u16,
        deadline: &HttpRequestDeadline,
    ) -> Result<(), CliError> {
        let route_intent = self.route_intent.take().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP upload terminal response requires its route intent".to_string(),
            )
        })?;
        let result = self.prepare_and_attach_responses(
            success_status,
            route_intent
                .upload()
                .response
                .as_ref()
                .expect("upload route retains its response"),
            deadline,
        );
        self.route_intent = Some(route_intent);
        result
    }

    fn prepare_and_attach_responses_from_apply_route(
        &mut self,
        success_status: u16,
        deadline: &HttpRequestDeadline,
    ) -> Result<(), CliError> {
        let route_intent = self.route_intent.take().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP apply terminal response requires its route intent".to_string(),
            )
        })?;
        let result = self.prepare_and_attach_responses(
            success_status,
            &route_intent
                .apply()
                .identity
                .as_ref()
                .expect("apply route retains its mutation identity")
                .response,
            deadline,
        );
        self.route_intent = Some(route_intent);
        result
    }

    /// Runs every fallible snapshot check before the single ledger seal. The
    /// proof is last because it is the authoritative final raw-document pass.
    fn revalidate_and_seal<C>(&mut self, mut should_cancel: C) -> Result<(), CliError>
    where
        C: FnMut() -> bool,
    {
        if self.sealed || self.ledger().is_prepublication_sealed() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation was already prepublication sealed".to_string(),
            ));
        }
        if self.requires_validation_proof && self.validation_proof.is_none() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation lost its prepublication validation proof".to_string(),
            ));
        }
        if self.requires_publication && self.publication.is_none() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation lost its prepared JSON publication".to_string(),
            ));
        }
        if self.requires_responses && self.responses.is_none() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation lost its prepared terminal responses".to_string(),
            ));
        }

        if let Some(mut batch) = self.object_batch.take() {
            let result = batch
                .revalidate_preseal(
                    self.ledger
                        .as_mut()
                        .expect("prepared HTTP mutation retains its ledger"),
                    &mut should_cancel,
                )
                .map_err(http_store_error);
            self.object_batch = Some(batch);
            result?;
        }
        if let Some(mut publication) = self.publication.take() {
            let result = publication.revalidate_preseal(
                self.ledger
                    .as_mut()
                    .expect("prepared HTTP mutation retains its ledger"),
            );
            self.publication = Some(publication);
            result?;
        }
        if let Some(mut proof) = self.validation_proof.take() {
            let ledger = self
                .ledger
                .take()
                .expect("prepared HTTP mutation retains its ledger");
            match proof.revalidate_preseal(ledger, &mut should_cancel) {
                Ok(ledger) => self.ledger = Some(ledger),
                Err(error) => {
                    let (error, ledger) = error.into_parts();
                    self.ledger = Some(ledger);
                    self.validation_proof = Some(proof);
                    return Err(http_store_error(error));
                }
            }
            self.validation_proof = Some(proof);
        }

        self.ledger
            .as_mut()
            .expect("prepared HTTP mutation retains its ledger")
            .seal_prepublication()
            .map_err(|error| upload_work_error(error, "seal prepared HTTP mutation"))?;
        self.sealed = true;
        Ok(())
    }

    /// Commits immutable objects before authority JSON, then releases the
    /// validation proof. Response selection remains a separate terminal step.
    fn commit_effects_after_seal(&mut self) -> Result<(), CliError> {
        if !self.sealed || !self.ledger().is_prepublication_sealed() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation must be sealed before commit".to_string(),
            ));
        }
        if let Some(mut batch) = self.object_batch.take() {
            let result = batch
                .commit_after_seal(
                    self.ledger
                        .as_mut()
                        .expect("prepared HTTP mutation retains its ledger"),
                )
                .map_err(http_store_error);
            self.object_batch = Some(batch);
            result?;
            drop(self.object_batch.take());
        }
        if let Some(mut publication) = self.publication.take() {
            let result = publication.commit_after_seal(
                self.ledger
                    .as_mut()
                    .expect("prepared HTTP mutation retains its ledger"),
            );
            self.publication = Some(publication);
            result?;
            drop(self.publication.take());
        }
        if let Some(mut proof) = self.validation_proof.take() {
            let result = proof
                .finish_after_seal(
                    self.ledger
                        .as_mut()
                        .expect("prepared HTTP mutation retains its ledger"),
                )
                .map_err(http_store_error);
            if let Err(error) = result {
                self.validation_proof = Some(proof);
                return Err(error);
            }
            self.validation_proof = Some(proof);
            drop(self.validation_proof.take());
        }
        Ok(())
    }

    fn select_success_after_effect_commit(&mut self) -> Result<(), CliError> {
        if self.validation_proof.is_some()
            || self.object_batch.is_some()
            || self.publication.is_some()
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation cannot select success before every effect owner commits".to_string(),
            ));
        }
        let mut responses = self.responses.take().ok_or_else(|| {
            CliError::InvalidRepository("HTTP mutation terminal responses are missing".to_string())
        })?;
        let result = responses.select_success(
            self.ledger
                .as_mut()
                .expect("prepared HTTP mutation retains its ledger"),
            &mut self.prepared_response,
        );
        if let Err(error) = result {
            self.responses = Some(responses);
            return Err(error);
        }
        Ok(())
    }

    fn commit_and_select_success(&mut self) -> Result<(), CliError> {
        self.commit_effects_after_seal()?;
        self.select_success_after_effect_commit()
    }

    fn validate_visibility_terminal(&self) -> Result<(), CliError> {
        if !self.sealed || !self.ledger().is_prepublication_sealed() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal requires a sealed ledger".to_string(),
            ));
        }
        if self.validation_proof.is_some()
            || self.object_batch.is_some()
            || self.publication.is_some()
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal still owns uncommitted effects".to_string(),
            ));
        }
        if self.responses.is_none() || self.prepared_response.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal requires unselected responses".to_string(),
            ));
        }
        Ok(())
    }

    fn into_visibility_terminal_after_validation(mut self) -> PreparedHttpMutationTerminal {
        assert!(
            self.validate_visibility_terminal().is_ok(),
            "HTTP mutation visibility terminal must be validated before extraction"
        );
        PreparedHttpMutationTerminal {
            ledger: self.ledger.take(),
            responses: self.responses.take(),
            prepared_response: None,
        }
    }

    fn into_selected_terminal_after_effect_resolution(
        mut self,
    ) -> Result<PreparedHttpMutationTerminal, CliError> {
        if !self.sealed
            || !self.ledger().is_prepublication_sealed()
            || self.validation_proof.is_some()
            || self.object_batch.is_some()
            || self.publication.is_some()
            || self.responses.is_some()
            || self.prepared_response.is_none()
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation selected terminal still owns unresolved effects".to_string(),
            ));
        }
        Ok(PreparedHttpMutationTerminal {
            ledger: self.ledger.take(),
            responses: None,
            prepared_response: self.prepared_response.take(),
        })
    }

    /// A post-seal error may already have made immutable progress. Cancel all
    /// still-live effect owners and select the fixed retry response.
    fn select_durable_after_commit_error(&mut self) -> Result<(), CliError> {
        let cancellation = self.cancel_effect_owners();
        if cancellation.is_err() {
            self.abandon_object_batch_for_durable_retry()?;
        }
        if self.publication.is_some()
            || self.object_batch.is_some()
            || self.validation_proof.is_some()
        {
            return Err(cancellation.expect_err(
                "a durable-retry effect owner remains only after cancellation failed",
            ));
        }
        let mut responses = self.responses.take().ok_or_else(|| {
            CliError::InvalidRepository("HTTP mutation terminal responses are missing".to_string())
        })?;
        let result = responses.select_durable(
            self.ledger
                .as_mut()
                .expect("prepared HTTP mutation retains its ledger"),
            &mut self.prepared_response,
        );
        if let Err(error) = result {
            self.responses = Some(responses);
            return Err(error);
        }
        Ok(())
    }

    /// Store stages are private `.tmp` files. If their ordinary unlink-based
    /// cancellation fails after the shared seal, leave them for the bounded
    /// recovery pass while releasing the batch's permit and memory ownership.
    /// Other effect owners must still cancel normally before a response can be
    /// transferred.
    fn abandon_object_batch_for_durable_retry(&mut self) -> Result<(), CliError> {
        let Some(mut batch) = self.object_batch.take() else {
            return Ok(());
        };
        let result = batch
            .abandon_for_durable_retry(
                self.ledger
                    .as_mut()
                    .expect("prepared HTTP mutation retains its ledger"),
            )
            .map_err(http_store_error);
        if let Err(error) = result {
            self.object_batch = Some(batch);
            return Err(error);
        }
        drop(batch);
        Ok(())
    }

    /// Checks that the writer handoff owns exactly one selected response and no
    /// other live request resource. This deliberately borrows `self`: a failed
    /// handoff check must leave the combined owner available for cancellation.
    fn validate_http_handoff(&self) -> Result<(), CliError> {
        if self.route_intent.is_some()
            || self.validation_proof.is_some()
            || self.object_batch.is_some()
            || self.publication.is_some()
            || self.responses.is_some()
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation still owns uncommitted resources".to_string(),
            ));
        }
        let response = self.prepared_response.as_ref().ok_or_else(|| {
            CliError::InvalidRepository("HTTP mutation has no selected response".to_string())
        })?;
        let response_bytes = u64::try_from(response.body.len()).map_err(|_| {
            CliError::InvalidRepository("selected HTTP response size overflow".to_string())
        })?;
        let current = &self.ledger().snapshot().current;
        let retained = current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let unrelated_current = current.iter().any(|(dimension, amount)| {
            *dimension != codefire_core::DecisionWorkDimension::RetainedBytes && *amount != 0
        });
        if retained != response_bytes || unrelated_current || self.ledger().has_open_permits() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation retained ownership does not exactly match its selected response"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Infallible extraction after `validate_http_handoff` succeeds. Keeping
    /// validation separate prevents an invariant error from consuming the only
    /// owner capable of cancelling the request-wide resources.
    fn into_http_parts_after_validation(
        mut self,
    ) -> (codefire_core::DecisionWorkLedger, PreparedHttpResponse) {
        assert!(
            self.validate_http_handoff().is_ok(),
            "HTTP mutation handoff must be validated before extraction"
        );
        let response = self
            .prepared_response
            .take()
            .expect("validated HTTP mutation retains its selected response");
        let ledger = self
            .ledger
            .take()
            .expect("prepared HTTP mutation retains its ledger");
        (ledger, response)
    }

    /// Returns the ledger only after explicit cancellation has removed every
    /// child owner, current-memory charge, and publication permit. Failure is
    /// non-consuming so callers may retry cleanup or fail closed.
    fn take_ledger(&mut self) -> Result<codefire_core::DecisionWorkLedger, CliError> {
        let owns_child = self.route_intent.is_some()
            || self.validation_proof.is_some()
            || self.object_batch.is_some()
            || self.publication.is_some()
            || self.responses.is_some()
            || self.prepared_response.is_some();
        let owns_current = self
            .ledger()
            .snapshot()
            .current
            .values()
            .any(|amount| *amount != 0);
        if !self.cancellation_complete
            || owns_child
            || owns_current
            || self.ledger().has_open_permits()
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation ledger cannot be taken before complete cancellation".to_string(),
            ));
        }
        Ok(self
            .ledger
            .take()
            .expect("prepared HTTP mutation retains its ledger"))
    }

    fn take_ledger_preserving_external_permits(
        &mut self,
    ) -> Result<codefire_core::DecisionWorkLedger, CliError> {
        let owns_child = self.route_intent.is_some()
            || self.validation_proof.is_some()
            || self.object_batch.is_some()
            || self.publication.is_some()
            || self.responses.is_some()
            || self.prepared_response.is_some();
        let owns_current = self
            .ledger()
            .snapshot()
            .current
            .values()
            .any(|amount| *amount != 0);
        if !self.cancellation_complete || owns_child || owns_current {
            return Err(CliError::InvalidRepository(
                "HTTP mutation ledger cannot preserve external permits before complete child cancellation"
                    .to_string(),
            ));
        }
        Ok(self
            .ledger
            .take()
            .expect("prepared HTTP mutation retains its ledger"))
    }

    fn cancel_effect_owners(&mut self) -> Result<(), CliError> {
        let mut first_error = None;
        if let Some(mut publication) = self.publication.take() {
            let result = publication.cancel(
                self.ledger
                    .as_mut()
                    .expect("prepared HTTP mutation retains its ledger"),
            );
            if let Err(error) = result {
                self.publication = Some(publication);
                first_error.get_or_insert(error);
            }
        }
        if let Some(mut batch) = self.object_batch.take() {
            let result = batch
                .cancel(
                    self.ledger
                        .as_mut()
                        .expect("prepared HTTP mutation retains its ledger"),
                )
                .map_err(http_store_error);
            if let Err(error) = result {
                self.object_batch = Some(batch);
                first_error.get_or_insert(error);
            }
        }
        if let Some(mut proof) = self.validation_proof.take() {
            let result = proof
                .cancel(
                    self.ledger
                        .as_mut()
                        .expect("prepared HTTP mutation retains its ledger"),
                )
                .map_err(http_store_error);
            if let Err(error) = result {
                self.validation_proof = Some(proof);
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn cancel_all(&mut self) -> Result<(), CliError> {
        self.cancellation_complete = false;
        // The external request payload must disappear before Store/publication
        // cleanup releases their independently owned retained scalars.
        let mut first_error = self.release_route_intent().err();
        if let Err(error) = self.cancel_effect_owners() {
            first_error.get_or_insert(error);
        }
        if let Some(mut responses) = self.responses.take() {
            let result = responses.cancel(
                self.ledger
                    .as_mut()
                    .expect("prepared HTTP mutation retains its ledger"),
            );
            if let Err(error) = result {
                self.responses = Some(responses);
                first_error.get_or_insert(error);
            }
        }
        if let Some(response) = self.prepared_response.take() {
            record_cleanup_error(
                &mut first_error,
                release_prepared_http_response(
                    self.ledger
                        .as_mut()
                        .expect("prepared HTTP mutation retains its ledger"),
                    response,
                    "release selected cancelled mutation response",
                ),
            );
        }
        match first_error {
            Some(error) => Err(error),
            None => {
                self.cancellation_complete = true;
                Ok(())
            }
        }
    }
}

impl Drop for PreparedHttpMutationTransaction<'_> {
    fn drop(&mut self) {
        if self.ledger.is_none() {
            return;
        }

        if let Err(cancel_error) = self.cancel_all() {
            if !self.ledger().is_prepublication_sealed() {
                if let Err(seal_error) = self
                    .ledger
                    .as_mut()
                    .expect("prepared HTTP mutation retains its ledger")
                    .seal_prepublication()
                {
                    fail_closed_http_mutation_drop(format!(
                        "HTTP mutation drop could not seal after cancellation failed ({cancel_error}); seal failed: {seal_error}"
                    ));
                }
                self.sealed = true;
            }
            if let Err(abandon_error) = self.abandon_object_batch_for_durable_retry() {
                fail_closed_http_mutation_drop(format!(
                    "HTTP mutation drop could not abandon its durable-retry Store stage after cancellation failed ({cancel_error}): {abandon_error}"
                ));
            }
            if let Err(retry_error) = self.cancel_all() {
                fail_closed_http_mutation_drop(format!(
                    "HTTP mutation drop cancellation remained unrecoverable after durable Store abandon ({cancel_error}): {retry_error}"
                ));
            }
        }

        let current_is_zero = self
            .ledger()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0);
        if !current_is_zero || self.ledger().has_open_permits() {
            fail_closed_http_mutation_drop(
                "HTTP mutation drop retained current ownership or an open publication permit"
                    .to_string(),
            );
        }
        let terminal = self
            .ledger()
            .snapshot()
            .terminal_reason
            .unwrap_or(codefire_core::DecisionWorkTerminalReason::CorruptSubject);
        if !matches!(
            self.ledger
                .as_mut()
                .expect("prepared HTTP mutation retains its ledger")
                .finish(terminal),
            Ok(()) | Err(codefire_core::DecisionWorkAccountingError::TerminalAlreadySet { .. })
        ) {
            fail_closed_http_mutation_drop(
                "HTTP mutation drop could not close its decision-work ledger".to_string(),
            );
        }
    }
}

impl PreparedHttpMutationTerminal {
    fn ledger(&self) -> &codefire_core::DecisionWorkLedger {
        self.ledger
            .as_ref()
            .expect("prepared HTTP mutation terminal retains its ledger")
    }

    fn select_success(&mut self) -> Result<(), CliError> {
        let mut responses = self.responses.take().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP mutation visibility terminal responses are missing".to_string(),
            )
        })?;
        let ledger = self
            .ledger
            .as_mut()
            .expect("prepared HTTP mutation terminal retains its ledger");
        let result = responses.select_success(ledger, &mut self.prepared_response);
        if let Err(error) = result {
            self.responses = Some(responses);
            return Err(error);
        }
        Ok(())
    }

    fn validate_http_handoff(&self) -> Result<(), CliError> {
        Self::validate_http_handoff_parts(self.ledger(), &self.responses, &self.prepared_response)
    }

    fn validate_http_handoff_parts(
        ledger: &codefire_core::DecisionWorkLedger,
        responses: &Option<PreparedMutationResponses>,
        prepared_response: &Option<PreparedHttpResponse>,
    ) -> Result<(), CliError> {
        if responses.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal still owns unselected responses".to_string(),
            ));
        }
        let response = prepared_response.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP mutation visibility terminal has no selected response".to_string(),
            )
        })?;
        let response_bytes = u64::try_from(response.body.len()).map_err(|_| {
            CliError::InvalidRepository("selected HTTP response size overflow".to_string())
        })?;
        let current = &ledger.snapshot().current;
        let retained = current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let unrelated_current = current.iter().any(|(dimension, amount)| {
            *dimension != codefire_core::DecisionWorkDimension::RetainedBytes && *amount != 0
        });
        if retained != response_bytes || unrelated_current || ledger.has_open_permits() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal ownership does not exactly match its selected response"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn validate_http_bootstrap_handoff_parts(
        ledger: &codefire_core::DecisionWorkLedger,
        responses: &Option<PreparedMutationResponses>,
        prepared_response: &Option<PreparedHttpResponse>,
    ) -> Result<ValidatedHttpBootstrapHandoffProof, CliError> {
        #[cfg(test)]
        HTTP_UPLOAD_BOOTSTRAP_HANDOFF_VALIDATIONS.with(|count| {
            count.set(
                count
                    .get()
                    .checked_add(1)
                    .expect("bootstrap handoff validation count overflow"),
            );
        });
        Self::validate_http_handoff_parts(ledger, responses, prepared_response)?;
        Ok(ValidatedHttpBootstrapHandoffProof)
    }

    fn publish_bootstrap_success(
        &mut self,
        bootstrap: &mut ArmedRemoteProjectBootstrap,
    ) -> Result<ValidatedHttpBootstrapHandoffProof, CliError> {
        #[cfg(test)]
        let final_project_root = bootstrap.final_project_root_for_test();
        let ledger = self
            .ledger
            .as_mut()
            .expect("prepared HTTP mutation terminal retains its ledger");
        let responses = &mut self.responses;
        let prepared_response = &mut self.prepared_response;
        bootstrap.publish_with_terminal_ledger(ledger, |ledger| {
            let mut candidates = responses.take().ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP mutation visibility terminal responses are missing".to_string(),
                )
            })?;
            if let Err(error) = candidates.select_success(ledger, prepared_response) {
                *responses = Some(candidates);
                return Err(error);
            }
            let proof =
                Self::validate_http_bootstrap_handoff_parts(ledger, responses, prepared_response)?;
            #[cfg(test)]
            inject_http_upload_bootstrap_collision(&final_project_root)?;
            Ok(proof)
        })
    }

    fn into_validated_http_bootstrap_handoff(
        self,
        _proof: ValidatedHttpBootstrapHandoffProof,
    ) -> ValidatedHttpBootstrapHandoff {
        ValidatedHttpBootstrapHandoff { terminal: self }
    }

    fn into_http_parts_after_validation(
        mut self,
    ) -> (codefire_core::DecisionWorkLedger, PreparedHttpResponse) {
        assert!(
            self.validate_http_handoff().is_ok(),
            "HTTP mutation visibility handoff must be validated before extraction"
        );
        let response = self
            .prepared_response
            .take()
            .expect("validated HTTP mutation terminal retains its selected response");
        let ledger = self
            .ledger
            .take()
            .expect("prepared HTTP mutation terminal retains its ledger");
        (ledger, response)
    }

    fn cancel_and_take_ledger(&mut self) -> Result<codefire_core::DecisionWorkLedger, CliError> {
        let ledger = self
            .ledger
            .as_mut()
            .expect("prepared HTTP mutation terminal retains its ledger");
        if let Some(mut responses) = self.responses.take() {
            if let Err(error) = responses.cancel(ledger) {
                self.responses = Some(responses);
                return Err(error);
            }
        }
        if let Some(response) = self.prepared_response.take() {
            release_prepared_http_response(
                ledger,
                response,
                "release cancelled visibility-terminal response",
            )?;
        }
        if ledger
            .snapshot()
            .current
            .values()
            .any(|amount| *amount != 0)
            || ledger.has_open_permits()
        {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal cancellation left owned work".to_string(),
            ));
        }
        Ok(self
            .ledger
            .take()
            .expect("cancelled HTTP mutation terminal retains its ledger"))
    }
}

impl ValidatedHttpBootstrapHandoff {
    fn into_http_parts(mut self) -> (codefire_core::DecisionWorkLedger, PreparedHttpResponse) {
        let response = self
            .terminal
            .prepared_response
            .take()
            .expect("validated HTTP bootstrap handoff retains its selected response");
        let ledger = self
            .terminal
            .ledger
            .take()
            .expect("validated HTTP bootstrap handoff retains its ledger");
        (ledger, response)
    }
}

impl Drop for PreparedHttpMutationTerminal {
    fn drop(&mut self) {
        if self.ledger.is_none() {
            return;
        }
        let mut ledger = match self.cancel_and_take_ledger() {
            Ok(ledger) => ledger,
            Err(error) => fail_closed_http_mutation_drop(format!(
                "HTTP mutation visibility terminal drop could not cancel: {error}"
            )),
        };
        let terminal = ledger
            .snapshot()
            .terminal_reason
            .unwrap_or(codefire_core::DecisionWorkTerminalReason::CorruptSubject);
        if !matches!(
            ledger.finish(terminal),
            Ok(()) | Err(codefire_core::DecisionWorkAccountingError::TerminalAlreadySet { .. })
        ) {
            fail_closed_http_mutation_drop(
                "HTTP mutation visibility terminal drop could not close its ledger".to_string(),
            );
        }
    }
}

#[allow(dead_code)]
fn fail_closed_http_mutation_drop(message: String) -> ! {
    if std::thread::panicking() {
        std::process::abort();
    }
    panic!("{message}");
}

struct HttpRouteContext<'a> {
    ledger: &'a mut Option<codefire_core::DecisionWorkLedger>,
    prepared: &'a mut Option<PreparedHttpResponse>,
    deadline: &'a HttpRequestDeadline,
    payload_owner: Option<AccountedObjectRecordPayloadOwnership>,
}

struct AccountedHttpJson {
    value: Value,
    retained_bytes: u64,
}

impl AccountedHttpJson {
    fn into_parts(self) -> (Value, u64) {
        (self.value, self.retained_bytes)
    }
}

fn release_http_retained(
    context: &mut HttpRouteContext<'_>,
    retained_bytes: u64,
    operation: &str,
) -> Result<(), CliError> {
    context
        .ledger
        .as_mut()
        .expect("HTTP route ledger must be present while releasing retained JSON")
        .release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        )
        .map_err(|error| upload_work_error(error, operation))
}

fn acquire_http_retained(
    context: &mut HttpRouteContext<'_>,
    retained_bytes: u64,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<(), CliError> {
    context
        .ledger
        .as_mut()
        .expect("HTTP route ledger must be present while acquiring retained ownership")
        .apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, operation))
}

fn retain_compact_http_json<T>(
    context: &mut HttpRouteContext<'_>,
    value: &T,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    retain_compact_http_json_with_ledger(
        context
            .ledger
            .as_mut()
            .expect("HTTP route ledger must be present while retaining JSON"),
        context.deadline,
        value,
        phase,
        operation,
    )
}

fn retain_compact_http_json_with_ledger<T>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
    value: &T,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    deadline.check("HTTP JSON ownership planning")?;
    let retained_bytes = account_compact_json(ledger, value, phase, operation)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, operation))?;
    if let Err(error) = deadline.check("HTTP JSON ownership accounting") {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|release| {
                upload_work_error(release, "release cancelled HTTP JSON ownership")
            })?;
        return Err(error);
    }
    Ok(retained_bytes)
}

fn reserve_compact_http_temporary<T>(
    context: &mut HttpRouteContext<'_>,
    value: &T,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    reserve_compact_http_temporary_with_ledger(
        context
            .ledger
            .as_mut()
            .expect("HTTP route ledger must be present while reserving a JSON clone"),
        context.deadline,
        value,
        phase,
        operation,
    )
}

fn reserve_compact_http_temporary_with_ledger<T>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
    value: &T,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    deadline.check("HTTP JSON clone planning")?;
    let temporary_bytes = account_compact_json(ledger, value, phase, operation)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, operation))?;
    if let Err(error) = deadline.check("HTTP JSON clone accounting") {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )
            .map_err(|release| upload_work_error(release, "release cancelled HTTP JSON clone"))?;
        return Err(error);
    }
    Ok(temporary_bytes)
}

fn release_http_temporary(
    context: &mut HttpRouteContext<'_>,
    temporary_bytes: u64,
    operation: &str,
) -> Result<(), CliError> {
    context
        .ledger
        .as_mut()
        .expect("HTTP route ledger must be present while releasing temporary JSON")
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )
        .map_err(|error| upload_work_error(error, operation))
}

fn reserve_http_object_field_growth<T>(
    context: &mut HttpRouteContext<'_>,
    object: &Value,
    field: &str,
    new_value: &T,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    reserve_http_object_field_growth_with_ledger(
        context
            .ledger
            .as_mut()
            .expect("HTTP route ledger must be present while planning an object mutation"),
        object,
        field,
        new_value,
        operation,
    )
}

fn reserve_http_object_field_growth_with_ledger<T>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    object: &Value,
    field: &str,
    new_value: &T,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    let (growth, _) =
        measure_http_object_field_change(ledger, object, field, new_value, operation)?;
    if growth > 0 {
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, growth),
            )
            .map_err(|error| upload_work_error(error, operation))?;
    }
    Ok(growth)
}

fn measure_http_object_field_change<T>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    object: &Value,
    field: &str,
    new_value: &T,
    operation: &str,
) -> Result<(u64, u64), CliError>
where
    T: Serialize + ?Sized,
{
    let members = object.as_object().ok_or_else(|| {
        CliError::InvalidRepository("HTTP authority record must be an object".to_string())
    })?;
    let new_bytes = account_compact_json(
        ledger,
        new_value,
        codefire_core::DecisionWorkPhase::Validation,
        operation,
    )?;
    let (growth, shrink) = if let Some(old_value) = members.get(field) {
        let old_bytes = account_compact_json(
            ledger,
            old_value,
            codefire_core::DecisionWorkPhase::Validation,
            operation,
        )?;
        (
            new_bytes.saturating_sub(old_bytes),
            old_bytes.saturating_sub(new_bytes),
        )
    } else {
        let key_bytes = account_compact_json(
            ledger,
            field,
            codefire_core::DecisionWorkPhase::Validation,
            operation,
        )?;
        let growth = key_bytes
            .checked_add(1)
            .and_then(|bytes| bytes.checked_add(new_bytes))
            .and_then(|bytes| bytes.checked_add(u64::from(!members.is_empty())))
            .ok_or_else(|| {
                CliError::InvalidRepository("HTTP authority growth overflow".to_string())
            })?;
        (growth, 0)
    };
    Ok((growth, shrink))
}

fn reserve_http_merge_request_materialization(
    ledger: &mut codefire_core::DecisionWorkLedger,
    mr: &Value,
    plan: &MergeRequestSubjectMaterializationPlan,
    legacy_history: Option<&PlannedLegacyReviewHistory<'_>>,
) -> Result<MergeRequestMaterializationOwnership, CliError> {
    let (subject_growth, _) = measure_http_object_field_change(
        ledger,
        mr,
        "subject",
        &plan.subject,
        "measure merge-request subject growth",
    )?;
    let (digest_growth, _) = measure_http_object_field_change(
        ledger,
        mr,
        "subject_digest",
        &plan.subject_digest,
        "measure merge-request subject-digest growth",
    )?;
    let subject_scratch = account_compact_json(
        ledger,
        &plan.subject,
        codefire_core::DecisionWorkPhase::Validation,
        "measure merge-request subject scratch",
    )?;
    let digest_scratch = account_compact_json(
        ledger,
        &plan.subject_digest,
        codefire_core::DecisionWorkPhase::Validation,
        "measure merge-request subject-digest scratch",
    )?;
    let mut retained_bytes = subject_growth.checked_add(digest_growth).ok_or_else(|| {
        CliError::InvalidRepository("merge-request subject growth overflow".to_string())
    })?;
    let mut temporary_bytes = subject_scratch.checked_add(digest_scratch).ok_or_else(|| {
        CliError::InvalidRepository("merge-request subject scratch overflow".to_string())
    })?;
    if let Some(history) = legacy_history {
        let (history_growth, _) = measure_http_object_field_change(
            ledger,
            mr,
            "legacy_review_history",
            history,
            "measure legacy review-history growth",
        )?;
        let empty_reviews: &[Value] = &[];
        let (_, reviews_shrink) = measure_http_object_field_change(
            ledger,
            mr,
            "reviews",
            empty_reviews,
            "measure cleared legacy reviews",
        )?;
        retained_bytes = retained_bytes
            .checked_add(history_growth)
            .and_then(|bytes| bytes.checked_sub(reviews_shrink))
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "legacy review-history growth accounting underflow".to_string(),
                )
            })?;
        temporary_bytes = temporary_bytes
            .checked_add(account_compact_json(
                ledger,
                history,
                codefire_core::DecisionWorkPhase::Validation,
                "measure legacy review-history scratch",
            )?)
            .ok_or_else(|| {
                CliError::InvalidRepository("legacy review-history scratch overflow".to_string())
            })?;
    }
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            ),
        )
        .map_err(|error| {
            upload_work_error(error, "reserve merge-request subject materialization")
        })?;
    Ok(MergeRequestMaterializationOwnership {
        retained_bytes,
        temporary_bytes,
    })
}

const HTTP_DIRECTORY_ITERATOR_SCRATCH_BYTES: u64 = 4 * 1024;

fn read_http_directory_entries_accounted(
    directory: &Path,
    context: &mut HttpRouteContext<'_>,
    phase: &'static str,
) -> Result<BTreeMap<PathBuf, u64>, CliError> {
    context.deadline.check(phase)?;
    let mut iterator = fs::read_dir(directory)?;
    let mut entries = BTreeMap::new();
    loop {
        let ledger = context
            .ledger
            .as_mut()
            .expect("HTTP route ledger must be present while scanning a directory");
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    HTTP_DIRECTORY_ITERATOR_SCRATCH_BYTES,
                ),
            )
            .map_err(|error| upload_work_error(error, "reserve HTTP directory iterator"))?;
        let next = (|| -> Result<Option<(PathBuf, u64)>, CliError> {
            let Some(entry) = iterator.next().transpose()? else {
                return Ok(None);
            };
            context.deadline.check(phase)?;
            let encoded_name = entry.file_name().into_string().map_err(|_| {
                CliError::InvalidRepository(
                    "HTTP authority directory contains a non-UTF-8 entry".to_string(),
                )
            })?;
            let directory_bytes = u64::try_from(directory.as_os_str().len()).map_err(|_| {
                CliError::InvalidRepository(
                    "HTTP directory path ownership size overflow".to_string(),
                )
            })?;
            let name_bytes = u64::try_from(encoded_name.len()).map_err(|_| {
                CliError::InvalidRepository(
                    "HTTP directory entry ownership size overflow".to_string(),
                )
            })?;
            let directory_encoding = directory.as_os_str().as_encoded_bytes();
            let separator_bytes = u64::from(
                !directory_encoding.is_empty()
                    && !directory_encoding.ends_with(std::path::MAIN_SEPARATOR_STR.as_bytes()),
            );
            let retained_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
                .checked_add(directory_bytes)
                .and_then(|bytes| bytes.checked_add(separator_bytes))
                .and_then(|bytes| bytes.checked_add(name_bytes))
                .ok_or_else(|| {
                    CliError::InvalidRepository(
                        "HTTP directory membership ownership overflow".to_string(),
                    )
                })?;
            context
                .ledger
                .as_mut()
                .expect("HTTP route ledger must be present while retaining a directory entry")
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::RecordRead,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    ),
                )
                .map_err(|error| upload_work_error(error, "retain HTTP directory membership"))?;
            Ok(Some((entry.path(), retained_bytes)))
        })();
        context
            .ledger
            .as_mut()
            .expect("HTTP route ledger must remain after scanning a directory entry")
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                HTTP_DIRECTORY_ITERATOR_SCRATCH_BYTES,
            )
            .map_err(|error| upload_work_error(error, "release HTTP directory iterator"))?;
        let Some((path, retained_bytes)) = next? else {
            break;
        };
        if let Some(replaced) = entries.insert(path, retained_bytes) {
            release_http_retained(context, replaced, "release duplicate directory membership")?;
        }
        context.deadline.check(phase)?;
    }
    Ok(entries)
}

fn read_optional_http_json_accounted(
    path: &Path,
    context: &mut HttpRouteContext<'_>,
    phase: &'static str,
) -> Result<Option<AccountedHttpJson>, CliError> {
    read_optional_http_json_accounted_with_ledger(
        path,
        context
            .ledger
            .as_mut()
            .expect("HTTP authority JSON ledger must be present"),
        context.deadline,
        phase,
    )
}

fn read_optional_http_json_accounted_with_ledger(
    path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
    phase: &'static str,
) -> Result<Option<AccountedHttpJson>, CliError> {
    deadline.check(phase)?;
    let (initial_identity, physical_bytes) = match secure_worktree::file_identity_nofollow(path) {
        Ok(snapshot) => snapshot,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CliError::Io(error)),
    };
    let parse_scratch = physical_bytes.checked_mul(2).ok_or_else(|| {
        CliError::InvalidRepository("HTTP authority JSON scratch overflow".to_string())
    })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::AggregateBytes,
                physical_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                parse_scratch,
            )
            .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
            .count(
                codefire_core::DecisionWorkCounter::FileBytesRead,
                physical_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "reserve HTTP authority JSON read"))?;
    let (physical_json, opened_identity) =
        match secure_worktree::read_file_nofollow_bounded_with_identity(path, physical_bytes) {
            Ok(read) => read,
            Err(error) => {
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        parse_scratch,
                    )
                    .map_err(|accounting| {
                        upload_work_error(accounting, "release failed HTTP authority JSON read")
                    })?;
                return Err(CliError::Io(error));
            }
        };
    let final_snapshot = secure_worktree::file_identity_nofollow(path);
    let stable_read = matches!(
        final_snapshot,
        Ok((final_identity, final_bytes))
            if initial_identity == opened_identity
                && opened_identity == final_identity
                && physical_bytes == final_bytes
                && physical_bytes == u64::try_from(physical_json.len()).unwrap_or(u64::MAX)
    );
    if !stable_read {
        drop(physical_json);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                parse_scratch,
            )
            .map_err(|accounting| {
                upload_work_error(accounting, "release unstable HTTP authority JSON read")
            })?;
        return Err(CliError::InvalidRepository(format!(
            "HTTP authority JSON changed identity or size while reading: {}",
            path.display()
        )));
    }
    let value = match serde_json::from_slice(&physical_json) {
        Ok(value) => value,
        Err(error) => {
            drop(physical_json);
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    parse_scratch,
                )
                .map_err(|accounting| {
                    upload_work_error(accounting, "release invalid HTTP authority JSON read")
                })?;
            return Err(CliError::Json(error));
        }
    };
    drop(physical_json);
    let retained_bytes = match account_compact_json(
        ledger,
        &value,
        codefire_core::DecisionWorkPhase::RecordRead,
        "measure HTTP authority JSON",
    ) {
        Ok(retained_bytes) => retained_bytes,
        Err(error) => {
            drop(value);
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    parse_scratch,
                )
                .map_err(|accounting| {
                    upload_work_error(accounting, "release unmeasured HTTP authority JSON")
                })?;
            return Err(error);
        }
    };
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        ),
    ) {
        drop(value);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                parse_scratch,
            )
            .map_err(|accounting| {
                upload_work_error(accounting, "release unretained HTTP authority JSON")
            })?;
        return Err(upload_work_error(error, "retain HTTP authority JSON"));
    }
    if let Err(error) = ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        parse_scratch,
    ) {
        drop(value);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|cleanup| {
                upload_work_error(
                    cleanup,
                    "release retained HTTP authority after scratch failure",
                )
            })?;
        return Err(upload_work_error(
            error,
            "release HTTP authority JSON scratch",
        ));
    }
    if let Err(error) = deadline.check(phase) {
        drop(value);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(|release| {
                upload_work_error(release, "release cancelled HTTP authority JSON")
            })?;
        return Err(error);
    }
    Ok(Some(AccountedHttpJson {
        value,
        retained_bytes,
    }))
}

fn read_http_branch_head_accounted(
    project_root: &Path,
    branch_name: &str,
    context: &mut HttpRouteContext<'_>,
    phase: &'static str,
) -> Result<Option<(String, u64)>, CliError> {
    let Some(accounted_branch) = read_optional_http_json_accounted(
        &remote_branch_path(project_root, branch_name),
        context,
        phase,
    )?
    else {
        return Ok(None);
    };
    let (branch, branch_retained_bytes) = accounted_branch.into_parts();
    let head_result = (|| -> Result<(String, u64), CliError> {
        validate_remote_branch_record(&branch)?;
        let head = branch
            .get("head")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidRepository("branch head is missing".to_string()))?;
        let head_frame_bytes = codefire_core::logical_retained_frame_bytes(head)
            .map_err(|error| upload_work_error(error, "account HTTP branch head"))?;
        acquire_http_retained(
            context,
            head_frame_bytes,
            codefire_core::DecisionWorkPhase::RecordRead,
            "retain HTTP branch head",
        )?;
        Ok((head.to_string(), head_frame_bytes))
    })();
    release_http_retained(
        context,
        branch_retained_bytes,
        "release HTTP branch authority JSON",
    )?;
    head_result.map(Some)
}

fn http_store_error(error: codefire_store::StoreError) -> CliError {
    match error {
        codefire_store::StoreError::DecisionWorkAccounting(accounting) => {
            match accounting.budget_exceeded() {
                Some(exhaustion) => CliError::from(exhaustion.clone()),
                None => CliError::InvalidRepository(format!(
                    "decision-work accounting failed during HTTP store validation: {accounting}"
                )),
            }
        }
        error => CliError::from(error),
    }
}

fn normalize_http_store_cli_error(error: CliError) -> CliError {
    match error {
        CliError::Store(store) => http_store_error(*store),
        error => error,
    }
}

fn http_store_terminal_reason(
    error: &codefire_store::StoreError,
) -> codefire_core::DecisionWorkTerminalReason {
    match error {
        codefire_store::StoreError::ValidationCancelled => {
            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded
        }
        codefire_store::StoreError::SealedValidationSubjectChanged { .. } => {
            codefire_core::DecisionWorkTerminalReason::SubjectChanged
        }
        codefire_store::StoreError::DecisionWorkAccounting(accounting)
            if accounting.budget_exceeded().is_some() =>
        {
            codefire_core::DecisionWorkTerminalReason::BudgetExceeded
        }
        _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
    }
}

fn with_http_sealed_validation<T, F>(
    context: &mut HttpRouteContext<'_>,
    objects: &Path,
    commit_ids: &[&str],
    phase: &'static str,
    operation: F,
) -> Result<T, CliError>
where
    F: FnOnce(&mut codefire_store::SealedCommitValidationSession) -> Result<T, CliError>,
{
    context.deadline.check(phase)?;
    let owned_ledger = context
        .ledger
        .take()
        .expect("HTTP sealed validation ledger must be present");
    let mut session =
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            objects,
            owned_ledger,
        ) {
            Ok(session) => session,
            Err(error) => {
                let (error, ledger) = error.into_parts();
                *context.ledger = Some(ledger);
                return Err(http_store_error(error));
            }
        };
    for commit_id in commit_ids {
        if let Err(error) =
            session.validate_commit_with_cancellation(commit_id, || context.deadline.is_expired())
        {
            let reason = http_store_terminal_reason(&error);
            *context.ledger = session.abort_and_take_decision_work_ledger(reason)?;
            if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                return Err(context.deadline.error(phase));
            }
            return Err(http_store_error(error));
        }
    }
    if let Err(error) = session.finalize_with_cancellation(|| context.deadline.is_expired()) {
        *context.ledger = session.take_decision_work_ledger()?;
        if matches!(error, codefire_store::StoreError::ValidationCancelled) {
            return Err(context.deadline.error(phase));
        }
        return Err(http_store_error(error));
    }
    let result = operation(&mut session);
    *context.ledger = session.take_decision_work_ledger()?;
    result
}

fn validate_http_merge_request_records_in_one_session(
    storage_root: &Path,
    project_root: &Path,
    context: &mut HttpRouteContext<'_>,
    objects: &Path,
    records: &[AccountedHttpJson],
    subject_digests: &mut BTreeMap<String, (String, u64)>,
) -> Result<(), CliError> {
    const PHASE: &str = "merge-request list sealed validation";
    context.deadline.check(PHASE)?;
    let owned_ledger = context
        .ledger
        .take()
        .expect("HTTP merge-request list ledger must be present");
    let mut session =
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            objects,
            owned_ledger,
        ) {
            Ok(session) => session,
            Err(error) => {
                let (error, ledger) = error.into_parts();
                *context.ledger = Some(ledger);
                return Err(http_store_error(error));
            }
        };
    for record in records {
        for field in ["source_head", "target_head_at_request", "applied_head"] {
            let Some(commit_id) = record.value.get(field).and_then(Value::as_str) else {
                if field == "applied_head" {
                    continue;
                }
                *context.ledger = session
                    .abort_and_take_decision_work_ledger(
                        codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                    )
                    .map_err(http_store_error)?;
                return Err(CliError::InvalidRepository(format!(
                    "merge request is missing {field}"
                )));
            };
            if let Err(error) = session
                .validate_commit_with_cancellation(commit_id, || context.deadline.is_expired())
            {
                let reason = http_store_terminal_reason(&error);
                *context.ledger = session
                    .abort_and_take_decision_work_ledger(reason)
                    .map_err(http_store_error)?;
                if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                    return Err(context.deadline.error(PHASE));
                }
                return Err(http_store_error(error));
            }
        }
        if record.value.get("subject").is_some() {
            let subject_context = (|| -> Result<_, CliError> {
                let source_info =
                    parse_cf_http_url(&required_string(&record.value, &["source_url"])?)?;
                let target_info =
                    parse_cf_http_url(&required_string(&record.value, &["target_url"])?)?;
                let source_head = required_string(&record.value, &["source_head"])?;
                Ok((source_info, target_info, source_head))
            })();
            let (source_info, target_info, source_head) = match subject_context {
                Ok(context) => context,
                Err(error) => {
                    *context.ledger = session
                        .abort_and_take_decision_work_ledger(
                            codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                        )
                        .map_err(http_store_error)?;
                    return Err(error);
                }
            };
            let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
            if target_root != project_root {
                *context.ledger = session
                    .abort_and_take_decision_work_ledger(
                        codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                    )
                    .map_err(http_store_error)?;
                return Err(CliError::InvalidRepository(
                    "merge request target does not belong to the listed project".to_string(),
                ));
            }
            let mut callback_error = None;
            let validation = session.with_provisional_decision_work_ledger(
                #[expect(
                    clippy::result_large_err,
                    reason = "the Store session callback contract requires StoreError"
                )]
                |ledger| {
                    let operation = (|| -> Result<(), CliError> {
                        let inserted = if subject_digests.contains_key(&source_head) {
                            false
                        } else {
                            let digest = merge_request_object_set_digest_with_work_and_poll(
                                &record.value,
                                &target_root,
                                ledger,
                                || {
                                    context
                                        .deadline
                                        .check("merge-request list subject graph digest")
                                },
                            )?;
                            insert_merge_request_subject_digest_cache_entry(
                                ledger,
                                subject_digests,
                                &source_head,
                                digest,
                            )?;
                            true
                        };
                        let object_set_digest = &subject_digests
                            .get(&source_head)
                            .expect("subject digest cache entry was prepared")
                            .0;
                        if let Err(error) = validate_merge_request_subject_with_object_set_digest(
                            &record.value,
                            &target_root,
                            &source_info.branch,
                            &target_info.branch,
                            object_set_digest,
                        ) {
                            if inserted {
                                remove_merge_request_subject_digest_cache_entry(
                                    ledger,
                                    subject_digests,
                                    &source_head,
                                )?;
                            }
                            return Err(error);
                        }
                        Ok(())
                    })();
                    match operation {
                        Ok(()) => Ok(()),
                        Err(error) => {
                            callback_error = Some(error);
                            Err(codefire_store::StoreError::ValidationCancelled)
                        }
                    }
                },
            );
            if let Err(error) = validation {
                let cli_error = callback_error.unwrap_or_else(|| http_store_error(error));
                let reason = http_request_terminal_reason(&cli_error);
                *context.ledger = session
                    .abort_and_take_decision_work_ledger(reason)
                    .map_err(http_store_error)?;
                return Err(cli_error);
            }
        }
    }
    if let Err(error) = session.finalize_with_cancellation(|| context.deadline.is_expired()) {
        *context.ledger = session
            .take_decision_work_ledger()
            .map_err(http_store_error)?;
        if matches!(error, codefire_store::StoreError::ValidationCancelled) {
            return Err(context.deadline.error(PHASE));
        }
        return Err(http_store_error(error));
    }
    *context.ledger = session
        .take_decision_work_ledger()
        .map_err(http_store_error)?;
    Ok(())
}

fn retain_merge_request_subject_digest_cache_entry(
    ledger: &mut codefire_core::DecisionWorkLedger,
    source_head: &str,
    digest: &str,
) -> Result<u64, CliError> {
    let frame_bytes = codefire_core::logical_retained_frame_bytes(source_head)
        .and_then(|bytes| {
            codefire_core::logical_retained_frame_bytes(digest).and_then(|digest_bytes| {
                bytes
                    .checked_add(digest_bytes)
                    .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)
            })
        })
        .map_err(|error| upload_work_error(error, "account merge-request subject digest cache"))?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::GraphDiscovery,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                frame_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "retain merge-request subject digest cache"))?;
    Ok(frame_bytes)
}

#[must_use = "HTTP digest destination ownership must be released after its values are dropped"]
#[derive(Debug)]
struct HttpDigestDestinationOwnership {
    retained_bytes: u64,
    ledger_identity: Option<codefire_core::DecisionWorkLedgerIdentity>,
}

impl HttpDigestDestinationOwnership {
    fn acquire(
        ledger: &mut codefire_core::DecisionWorkLedger,
        digest: &AccountedSourceObjectSetDigest,
        copies: u64,
        phase: codefire_core::DecisionWorkPhase,
        operation: &str,
    ) -> Result<Self, CliError> {
        digest.ensure_ledger(ledger)?;
        let retained_bytes = digest.retained_bytes().checked_mul(copies).ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP object-set digest destination ownership overflow".to_string(),
            )
        })?;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    phase,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                ),
            )
            .map_err(|error| upload_work_error(error, operation))?;
        Ok(Self {
            retained_bytes,
            ledger_identity: Some(ledger.identity()),
        })
    }

    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        let identity = self.ledger_identity.ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP object-set digest destination ownership was already released".to_string(),
            )
        })?;
        if !ledger.matches_identity(identity) {
            return Err(CliError::InvalidRepository(
                "HTTP object-set digest destinations belong to a different decision-work ledger"
                    .to_string(),
            ));
        }
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            )
            .map_err(|error| {
                upload_work_error(error, "release HTTP object-set digest destinations")
            })?;
        self.retained_bytes = 0;
        self.ledger_identity = None;
        Ok(())
    }
}

impl Drop for HttpDigestDestinationOwnership {
    fn drop(&mut self) {
        debug_assert!(
            self.retained_bytes == 0 && self.ledger_identity.is_none(),
            "HTTP object-set digest destination ownership was dropped before release"
        );
    }
}

fn record_http_cleanup_error(first_error: &mut Option<CliError>, result: Result<(), CliError>) {
    if first_error.is_none() {
        if let Err(error) = result {
            *first_error = Some(error);
        }
    }
}

fn insert_merge_request_subject_digest_cache_entry(
    ledger: &mut codefire_core::DecisionWorkLedger,
    subject_digests: &mut BTreeMap<String, (String, u64)>,
    source_head: &str,
    mut digest: AccountedSourceObjectSetDigest,
) -> Result<(), CliError> {
    digest.ensure_ledger(ledger)?;
    if subject_digests.contains_key(source_head) {
        digest.release(ledger)?;
        return Err(CliError::InvalidRepository(
            "merge-request subject digest cache entry already exists".to_string(),
        ));
    }
    let frame_bytes =
        match retain_merge_request_subject_digest_cache_entry(ledger, source_head, digest.as_str())
        {
            Ok(frame_bytes) => frame_bytes,
            Err(error) => {
                digest.release(ledger)?;
                return Err(error);
            }
        };
    let key = source_head.to_string();
    let value = digest.as_str().to_string();
    subject_digests.insert(key, (value, frame_bytes));
    if let Err(error) = digest.release(ledger) {
        let cleanup =
            remove_merge_request_subject_digest_cache_entry(ledger, subject_digests, source_head);
        return match cleanup {
            Ok(()) => Err(error),
            Err(cleanup) => Err(CliError::InvalidRepository(format!(
                "digest source handoff failed ({error}); cache cleanup also failed ({cleanup})"
            ))),
        };
    }
    Ok(())
}

fn remove_merge_request_subject_digest_cache_entry(
    ledger: &mut codefire_core::DecisionWorkLedger,
    subject_digests: &mut BTreeMap<String, (String, u64)>,
    source_head: &str,
) -> Result<(), CliError> {
    let Some((digest, retained_bytes)) = subject_digests.remove(source_head) else {
        return Ok(());
    };
    drop(digest);
    ledger
        .release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        )
        .map_err(|error| {
            upload_work_error(error, "release merge-request subject digest cache entry")
        })
}

#[cfg(test)]
fn read_http_request<S: Read + Write>(stream: &mut S) -> Result<HttpRequest, CliError> {
    let profile = codefire_core::DecisionWorkProfile::http_v1();
    let deadline = HttpRequestDeadline::from_profile(&profile);
    let mut ledger = codefire_core::DecisionWorkLedger::new(
        profile,
        codefire_core::DecisionWorkCommandKind::HttpRequest,
        "http request",
    )
    .map_err(|error| upload_work_error(error, "initialize request parser"))?;
    read_http_request_with_ledger(stream, &mut ledger, &deadline)
}

fn read_http_request_with_ledger<S: Read + Write>(
    stream: &mut S,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<HttpRequest, CliError> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];
    let header_end = loop {
        deadline.check("request header parsing")?;
        let read = stream.read(&mut temp)?;
        if read == 0 {
            return Err(CliError::InvalidRepository(
                "incomplete HTTP request".to_string(),
            ));
        }
        let completed = find_header_end_after_append(&buffer, &temp[..read]);
        let header_after = completed
            .map(|position| position + 4)
            .unwrap_or_else(|| buffer.len() + read);
        let header_increment = header_after.saturating_sub(buffer.len());
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestHeaders,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::RequestHeaderBytes,
                    u64::try_from(header_increment).map_err(|_| {
                        CliError::InvalidRepository("HTTP request header size overflow".to_string())
                    })?,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    u64::try_from(read).map_err(|_| {
                        CliError::InvalidRepository("HTTP request size overflow".to_string())
                    })?,
                ),
            )
            .map_err(|error| upload_work_error(error, "account request header input"))?;
        buffer.extend_from_slice(&temp[..read]);
        if let Some(position) = completed {
            break position;
        }
    };
    let header_bytes = header_end + 4;
    let header = std::str::from_utf8(&buffer[..header_end])
        .map_err(|_| CliError::Usage("HTTP request header is not UTF-8".to_string()))?;
    deadline.check("request planning")?;
    let mut lines = header.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| CliError::InvalidRepository("missing HTTP request line".to_string()))?;
    let (method, path) = parse_http_request_line(request_line)?;
    let content_length = parse_content_length(
        lines,
        codefire_core::DecisionWorkDimension::RequestBodyBytes,
        codefire_core::DecisionWorkPhase::RequestBody,
        "http request",
        Some(ledger),
    )?;
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(charge_http_limit_failure(
            ledger,
            codefire_core::DecisionWorkDimension::RequestBodyBytes,
            codefire_core::DecisionWorkPhase::RequestBody,
            "http request",
        ));
    }
    let content_length_u64 = u64::try_from(content_length)
        .map_err(|_| CliError::InvalidRepository("HTTP request body size overflow".to_string()))?;
    let body_read_ahead = buffer.len().saturating_sub(header_bytes);
    if body_read_ahead > content_length {
        return Err(CliError::Usage(
            "HTTP request contains bytes beyond Content-Length".to_string(),
        ));
    }
    let routing_retained_bytes = sum_http_owner_bytes([
        logical_string_owner_bytes(method)?,
        logical_string_owner_bytes(path)?,
    ])?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RequestHeaders,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                routing_retained_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "retain parsed request routing"))?;
    let mut routing = HttpRequestRoutingOwnership {
        ledger_identity: ledger.identity(),
        method: Some(method.to_string()),
        path: Some(path.to_string()),
        retained_bytes: Some(routing_retained_bytes),
    };
    let body = (|| -> Result<HttpRequestBodyOwnership, CliError> {
        buffer.drain(..header_bytes);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                u64::try_from(header_bytes).map_err(|_| {
                    CliError::InvalidRepository("HTTP request header size overflow".to_string())
                })?,
            )
            .map_err(|error| upload_work_error(error, "release request header input"))?;
        let unread_body_bytes = content_length.saturating_sub(body_read_ahead);
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestBody,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::RequestBodyBytes,
                    content_length_u64,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    u64::try_from(unread_body_bytes).map_err(|_| {
                        CliError::InvalidRepository("HTTP request body size overflow".to_string())
                    })?,
                ),
            )
            .map_err(|error| upload_work_error(error, "reserve request body"))?;
        while buffer.len() < content_length {
            deadline.check("request body parsing")?;
            let remaining = content_length - buffer.len();
            let read_limit = remaining.min(temp.len());
            let read = stream.read(&mut temp[..read_limit])?;
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);
        }
        if buffer.len() < content_length {
            return Err(CliError::InvalidRepository(
                "incomplete HTTP request body".to_string(),
            ));
        }
        let body_buffer = std::mem::take(&mut buffer);
        let body_bytes = &body_buffer[..content_length];
        let body = if body_bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(body_bytes)?
        };
        drop(body_buffer);
        deadline.check("request body validation")?;
        let retained_body_bytes = compact_json_size(&body)?;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestBody,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_body_bytes,
                ),
            )
            .map_err(|error| upload_work_error(error, "retain parsed request body"))?;
        let mut body = HttpRequestBodyOwnership::new(body, retained_body_bytes, ledger);
        if let Err(error) = deadline.check("request dispatch") {
            body.release(ledger)?;
            return Err(error);
        }
        if let Err(error) = ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            content_length_u64,
        ) {
            body.release(ledger)?;
            return Err(upload_work_error(error, "release request body scratch"));
        }
        Ok(body)
    })();
    match body {
        Ok(body) => Ok(HttpRequest { routing, body }),
        Err(error) => {
            drop(buffer);
            routing.release(ledger)?;
            Err(error)
        }
    }
}

fn find_header_end_after_append(existing: &[u8], incoming: &[u8]) -> Option<usize> {
    const DELIMITER: &[u8; 4] = b"\r\n\r\n";
    let tail_start = existing.len().saturating_sub(DELIMITER.len() - 1);
    let tail = &existing[tail_start..];
    let mut boundary = [0_u8; 1027];
    let boundary_len = tail.len() + incoming.len();
    debug_assert!(boundary_len <= boundary.len());
    boundary[..tail.len()].copy_from_slice(tail);
    boundary[tail.len()..boundary_len].copy_from_slice(incoming);
    boundary[..boundary_len]
        .windows(DELIMITER.len())
        .position(|window| window == DELIMITER)
        .map(|position| tail_start + position)
}

fn parse_content_length<'a>(
    lines: impl Iterator<Item = &'a str>,
    dimension: codefire_core::DecisionWorkDimension,
    phase: codefire_core::DecisionWorkPhase,
    command: &str,
    mut ledger: Option<&mut codefire_core::DecisionWorkLedger>,
) -> Result<usize, CliError> {
    let mut content_length = None;
    for line in lines {
        let Some((name, raw_value)) = line.split_once(':') else {
            return Err(CliError::Usage("malformed HTTP header line".to_string()));
        };
        if !name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        if content_length.is_some() {
            return Err(CliError::Usage(
                "duplicate Content-Length header is not allowed".to_string(),
            ));
        }
        let value = raw_value.trim();
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(CliError::Usage(
                "Content-Length must be an unsigned decimal integer".to_string(),
            ));
        }
        let mut parsed = 0_u64;
        for digit in value.bytes() {
            parsed = match parsed
                .checked_mul(10)
                .and_then(|current| current.checked_add(u64::from(digit - b'0')))
            {
                Some(parsed) => parsed,
                None => {
                    return Err(match ledger.as_deref_mut() {
                        Some(ledger) => {
                            charge_http_limit_failure(ledger, dimension, phase, command)
                        }
                        None => http_budget_exceeded(dimension, phase, command),
                    });
                }
            };
            if parsed > MAX_HTTP_BODY_BYTES as u64 {
                return Err(match ledger.as_deref_mut() {
                    Some(ledger) => charge_http_limit_failure(ledger, dimension, phase, command),
                    None => http_budget_exceeded(dimension, phase, command),
                });
            }
        }
        content_length = Some(usize::try_from(parsed).map_err(
            |_| match ledger.as_deref_mut() {
                Some(ledger) => charge_http_limit_failure(ledger, dimension, phase, command),
                None => http_budget_exceeded(dimension, phase, command),
            },
        )?);
    }
    Ok(content_length.unwrap_or(0))
}

fn parse_http_request_line(request_line: &str) -> Result<(&str, &str), CliError> {
    let mut parts = request_line.split_whitespace();
    let Some(method) = parts.next() else {
        return Err(CliError::InvalidRepository(
            "invalid HTTP request line".to_string(),
        ));
    };
    let Some(path) = parts.next() else {
        return Err(CliError::InvalidRepository(
            "invalid HTTP request line".to_string(),
        ));
    };
    let Some(version) = parts.next() else {
        return Err(CliError::InvalidRepository(
            "invalid HTTP request line".to_string(),
        ));
    };
    if parts.next().is_some() {
        return Err(CliError::InvalidRepository(
            "invalid HTTP request line".to_string(),
        ));
    }
    if !matches!(method, "GET" | "POST") {
        return Err(CliError::Usage(format!(
            "unsupported HTTP method: {method}"
        )));
    }
    if version != "HTTP/1.1" {
        return Err(CliError::Usage(format!(
            "unsupported HTTP version: {}",
            version
        )));
    }
    validate_http_request_path(path)?;
    Ok((method, path))
}

fn validate_http_request_path(raw_path: &str) -> Result<(), CliError> {
    if !raw_path.starts_with('/') {
        return Err(CliError::Usage("HTTP path must be absolute".to_string()));
    }
    if raw_path.contains('\\')
        || raw_path.contains("://")
        || raw_path.as_bytes().iter().any(u8::is_ascii_control)
    {
        return Err(CliError::Usage("invalid HTTP path".to_string()));
    }
    let path = raw_path.split_once('?').map_or(raw_path, |(path, _)| path);
    for part in path.split('/').filter(|part| !part.is_empty()) {
        let _ = decode_http_path_component(part)?;
    }
    Ok(())
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn write_http_json<S: Read + Write>(
    stream: &mut S,
    status: u16,
    payload: &Value,
) -> Result<(), CliError> {
    let (status, body) = match encode_http_json(payload) {
        Ok(body) => (status, body),
        Err(error @ CliError::ResourceBudget(_)) => {
            let (error_status, error_payload) = http_error_response(&error);
            (error_status, encode_http_json(&error_payload)?)
        }
        Err(error) => return Err(error),
    };
    write_http_json_body(stream, status, &body)
}

fn encode_http_json(payload: &Value) -> Result<Vec<u8>, CliError> {
    let mut writer = BoundedJsonWriter::new(MAX_HTTP_BODY_BYTES);
    match serde_json::to_writer(&mut writer, payload) {
        Ok(()) => Ok(writer.into_inner()),
        Err(_) if writer.exceeded() => Err(http_budget_exceeded(
            codefire_core::DecisionWorkDimension::ResponseBytes,
            codefire_core::DecisionWorkPhase::Encoding,
            "http response",
        )),
        Err(error) => Err(CliError::Json(error)),
    }
}

fn write_http_json_body<S: Read + Write>(
    stream: &mut S,
    status: u16,
    body: &[u8],
) -> Result<(), CliError> {
    let reason = http_status_reason(status);
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}

fn http_status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Payload Too Large",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

struct BoundedJsonWriter {
    body: Vec<u8>,
    limit: usize,
    exceeded: bool,
}

impl BoundedJsonWriter {
    fn new(limit: usize) -> Self {
        Self {
            body: Vec::new(),
            limit,
            exceeded: false,
        }
    }

    fn exceeded(&self) -> bool {
        self.exceeded
    }

    fn into_inner(self) -> Vec<u8> {
        self.body
    }
}

impl Write for BoundedJsonWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.body.len()) {
            self.exceeded = true;
            return Err(std::io::Error::other(
                "serialized HTTP response exceeds the configured budget",
            ));
        }
        self.body.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
fn dispatch_http(storage_root: &Path, request: &mut HttpRequest) -> Result<(u16, Value), CliError> {
    let profile = codefire_core::DecisionWorkProfile::http_v1();
    let deadline = HttpRequestDeadline::from_profile(&profile);
    let ledger = codefire_core::DecisionWorkLedger::new(
        profile,
        codefire_core::DecisionWorkCommandKind::HttpRequest,
        "http request",
    )
    .map_err(|error| upload_work_error(error, "initialize request"))?;
    let mut ledger = Some(ledger);
    let mut prepared = None;
    let mut response =
        dispatch_http_with_ledger(storage_root, request, &mut ledger, &mut prepared, &deadline)?;
    if let Some(mut owner) = response.payload_owner.take() {
        drop(response.payload);
        owner.release_after_payload_drop(
            ledger
                .as_mut()
                .expect("test HTTP dispatch ledger must own its payload"),
        )?;
        return Err(CliError::InvalidRepository(
            "test-only dispatch helper cannot return an accounted payload".to_string(),
        ));
    }
    Ok((response.status, response.payload))
}

fn dispatch_http_with_ledger(
    storage_root: &Path,
    request: &mut HttpRequest,
    ledger: &mut Option<codefire_core::DecisionWorkLedger>,
    prepared: &mut Option<PreparedHttpResponse>,
    deadline: &HttpRequestDeadline,
) -> Result<HttpDispatchResponse, CliError> {
    deadline.check("request dispatch")?;
    let mut context = HttpRouteContext {
        ledger,
        prepared,
        deadline,
        payload_owner: None,
    };
    let mut mutation_routing = HttpMutationRoutingOwnership::prepare(
        request.method(),
        request.path(),
        context
            .ledger
            .as_mut()
            .expect("HTTP mutation routing ledger must be present"),
    )?;
    let mutation_kind = mutation_routing.as_ref().map(|routing| routing.kind);
    let mut mutation_intent = if mutation_kind.is_some() {
        let intent = match request.transfer_mutation_intent(
            &mut mutation_routing,
            context
                .ledger
                .as_ref()
                .expect("HTTP mutation intent ledger must be present"),
            deadline,
        ) {
            Ok(intent) => intent,
            Err(error) => {
                if let Some(routing) = mutation_routing.as_mut() {
                    routing.release(
                        context
                            .ledger
                            .as_mut()
                            .expect("HTTP rejected mutation routing ledger must be present"),
                    )?;
                }
                return Err(error);
            }
        };
        Some(intent)
    } else {
        None
    };
    let response = if let Some(kind) = mutation_kind {
        match kind {
            HttpMutationRoutingKind::Upload => handle_http_upload_with_route_intent(
                storage_root,
                &mut mutation_intent,
                &mut context,
            ),
            HttpMutationRoutingKind::Apply => handle_http_request_apply_with_route_intent(
                storage_root,
                &mut mutation_intent,
                &mut context,
            ),
        }
    } else {
        let (org, app, rest) = http_project_parts(request.path())?;
        let rest = rest.iter().map(String::as_str).collect::<Vec<_>>();
        match (request.method(), rest.as_slice()) {
            ("GET", ["doctor"]) => handle_http_doctor(
                storage_root,
                &org,
                &app,
                doctor_quick_requested(request.path())?,
                &mut context,
            ),
            ("GET", ["branches"]) => handle_http_list(storage_root, &org, &app, &mut context),
            ("GET", ["branches", branch, "bundle"]) => {
                handle_http_bundle(storage_root, &org, &app, branch, &mut context)
            }
            ("GET", ["merge-requests"]) => {
                handle_http_request_list(storage_root, &org, &app, &mut context)
            }
            ("POST", ["merge-requests"]) => {
                handle_http_request_merge(storage_root, &org, &app, request.body(), &mut context)
            }
            ("POST", ["merge-requests", mr_id, "review"]) => handle_http_request_review(
                storage_root,
                &org,
                &app,
                mr_id,
                request.body(),
                &mut context,
            ),
            _ => Ok((404, json!({"error": "unknown endpoint"}))),
        }
    };
    if let Some(mut intent) = mutation_intent.take() {
        intent.release(
            context
                .ledger
                .as_mut()
                .expect("HTTP route intent ledger must return from dispatch"),
        )?;
    }
    let response =
        match response {
            Ok(response) => response,
            Err(error) => {
                if let Some(mut owner) = context.payload_owner.take() {
                    owner.release_after_payload_drop(context.ledger.as_mut().expect(
                        "HTTP route ledger must return while releasing payload ownership",
                    ))?;
                }
                return Err(error);
            }
        };
    Ok(HttpDispatchResponse {
        status: response.0,
        payload: response.1,
        payload_owner: context.payload_owner,
    })
}

fn doctor_quick_requested(path: &str) -> Result<bool, CliError> {
    match path.split_once('?').map(|(_, query)| query) {
        None => Ok(false),
        Some("quick=1") => Ok(true),
        Some(_) => Err(CliError::Usage(
            "unsupported remote doctor query; expected quick=1".to_string(),
        )),
    }
}

fn handle_http_doctor(
    storage_root: &Path,
    org: &str,
    app: &str,
    quick: bool,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("remote doctor planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let project_url = format!("cf+http://local/{org}/{app}");
    let deadline = context.deadline;
    let report = diagnose_remote_project_with_work_and_poll(
        &project_root,
        &project_url,
        "http",
        quick,
        context.ledger,
        || deadline.check("remote doctor scan"),
    )?;
    let report_bytes = report.retained_bytes();
    let clone_bytes = reserve_compact_http_temporary(
        context,
        report.report(),
        codefire_core::DecisionWorkPhase::Encoding,
        "reserve remote doctor response projection",
    )?;
    if clone_bytes != report_bytes {
        return Err(CliError::InvalidRepository(
            "remote doctor report ownership does not match its projected payload".to_string(),
        ));
    }
    context
        .deadline
        .check("remote doctor response projection")?;
    let payload = remote_doctor_data_json(report.report());
    drop(report);
    release_http_retained(
        context,
        report_bytes,
        "release source remote doctor report after projection",
    )?;
    acquire_http_retained(
        context,
        report_bytes,
        codefire_core::DecisionWorkPhase::Encoding,
        "transfer remote doctor response projection to retained ownership",
    )?;
    release_http_temporary(
        context,
        clone_bytes,
        "release temporary remote doctor response projection",
    )?;
    Ok((200, payload))
}

fn http_project_parts(path: &str) -> Result<(String, String, Vec<String>), CliError> {
    let path = path.split_once('?').map_or(path, |(path, _)| path);
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(decode_http_path_component)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() < 4 || parts[0] != "v1" || parts[1] != "projects" {
        return Err(CliError::Usage("unknown endpoint".to_string()));
    }
    Ok((parts[2].clone(), parts[3].clone(), parts[4..].to_vec()))
}

fn decode_http_path_component(value: &str) -> Result<String, CliError> {
    let decoded = percent_decode(value)
        .map_err(|error| CliError::Usage(format!("invalid HTTP path component: {error}")))?;
    super::remote::validate_remote_component(&decoded, "HTTP path")
        .map_err(|error| CliError::Usage(format!("invalid HTTP path component: {error}")))?;
    Ok(decoded)
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

fn acquire_with_http_deadline<T, F>(
    request: &Value,
    deadline: &HttpRequestDeadline,
    phase: &'static str,
    acquire: F,
) -> Result<T, CliError>
where
    F: FnOnce(&LockOptions) -> Result<T, CliError>,
{
    let mut options = http_lock_options(request)?;
    let remaining_ms = deadline.remaining_ms(phase)?;
    let deadline_limited = options.wait
        && options
            .timeout_ms
            .map_or(true, |timeout_ms| timeout_ms > remaining_ms);
    if options.wait {
        options.timeout_ms = Some(
            options
                .timeout_ms
                .map_or(remaining_ms, |timeout_ms| timeout_ms.min(remaining_ms)),
        );
    }
    match acquire(&options) {
        Err(CliError::LockContention(_)) if deadline_limited => Err(deadline.error(phase)),
        result => result,
    }
}

fn handle_http_list(
    storage_root: &Path,
    org: &str,
    app: &str,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("branch list planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let dirs = remote_dirs(&project_root);
    if !dirs.branches.exists() {
        return Ok((404, json!({"error": "remote project not found"})));
    }
    let _project_gate = RemoteProjectGate::acquire_shared(&project_root, &LockOptions::default())?;
    let paths = read_http_directory_entries_accounted(&dirs.branches, context, "branch list scan")?;
    let mut branch_records = Vec::new();
    let mut heads = Vec::new();
    let mut head_frame_bytes = 0_u64;
    for (path, path_frame_bytes) in paths {
        context.deadline.check("branch list record validation")?;
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            release_http_retained(
                context,
                path_frame_bytes,
                "release ignored branch directory membership",
            )?;
            continue;
        }
        let accounted_branch =
            read_optional_http_json_accounted(&path, context, "branch authority JSON read")?
                .ok_or_else(|| {
                    CliError::InvalidRepository("branch record disappeared".to_string())
                })?;
        release_http_retained(
            context,
            path_frame_bytes,
            "release branch directory membership",
        )?;
        let (branch, branch_retained_bytes) = accounted_branch.into_parts();
        validate_remote_branch_record(&branch)?;
        let head = branch
            .get("head")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidRepository("branch head is missing".to_string()))?;
        let frame_bytes = codefire_core::logical_retained_frame_bytes(head)
            .map_err(|error| upload_work_error(error, "account branch head frame"))?;
        acquire_http_retained(
            context,
            frame_bytes,
            codefire_core::DecisionWorkPhase::RecordRead,
            "retain branch head frame",
        )?;
        head_frame_bytes = head_frame_bytes.checked_add(frame_bytes).ok_or_else(|| {
            CliError::InvalidRepository("branch head frame accounting overflow".to_string())
        })?;
        heads.push(head.to_string());
        branch_records.push((branch, branch_retained_bytes));
    }
    let head_refs = heads.iter().map(String::as_str).collect::<Vec<_>>();
    with_http_sealed_validation(
        context,
        &dirs.objects,
        &head_refs,
        "branch list sealed validation",
        |_| Ok(()),
    )?;
    release_http_retained(
        context,
        head_frame_bytes,
        "release branch head frames after validation",
    )?;
    #[derive(Serialize)]
    struct BranchListProjection<'a> {
        head: &'a str,
        name: &'a str,
        version: u64,
    }
    #[derive(Serialize)]
    struct BranchListEnvelope<'a> {
        branches: &'a [Value],
    }
    let mut branches = Vec::with_capacity(branch_records.len());
    retain_compact_http_json(
        context,
        &BranchListEnvelope { branches: &[] },
        codefire_core::DecisionWorkPhase::Encoding,
        "retain branch list envelope",
    )?;
    for (branch, branch_retained_bytes) in branch_records {
        context.deadline.check("branch list projection")?;
        let name = branch
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidRepository("branch name is missing".to_string()))?;
        let head = branch
            .get("head")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidRepository("branch head is missing".to_string()))?;
        if !branches.is_empty() {
            acquire_http_retained(
                context,
                1,
                codefire_core::DecisionWorkPhase::Encoding,
                "retain branch list separator",
            )?;
        }
        retain_compact_http_json(
            context,
            &BranchListProjection {
                head,
                name,
                version: 1,
            },
            codefire_core::DecisionWorkPhase::Encoding,
            "retain branch list projection",
        )?;
        branches.push(json!({
            "version": 1,
            "name": name,
            "head": head
        }));
        release_http_retained(
            context,
            branch_retained_bytes,
            "release projected branch authority JSON",
        )?;
    }
    Ok((200, json!({"branches": branches})))
}

fn handle_http_bundle(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("bundle planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let _project_gate = RemoteProjectGate::acquire_shared(&project_root, &LockOptions::default())?;
    let branch_path = remote_branch_path(&project_root, branch_name);
    let Some(accounted_branch) =
        read_optional_http_json_accounted(&branch_path, context, "bundle branch authority read")?
    else {
        return Ok((404, json!({"error": "remote branch not found"})));
    };
    let (branch, _branch_payload_retained_bytes) = accounted_branch.into_parts();
    validate_remote_branch_record(&branch)?;
    let head = branch
        .get("head")
        .and_then(Value::as_str)
        .ok_or_else(|| CliError::InvalidRepository("branch head is missing".to_string()))?;
    let head_frame_bytes = codefire_core::logical_retained_frame_bytes(head)
        .map_err(|error| upload_work_error(error, "account bundle branch head"))?;
    acquire_http_retained(
        context,
        head_frame_bytes,
        codefire_core::DecisionWorkPhase::RecordRead,
        "retain bundle branch head",
    )?;
    let head = head.to_string();
    let objects = remote_dirs(&project_root).objects;
    let deadline = context.deadline;
    let ledger = &mut *context.ledger;
    let owned_ledger = ledger
        .take()
        .expect("HTTP bundle request ledger must be present");
    let mut sealed =
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            &objects,
            owned_ledger,
        ) {
            Ok(session) => session,
            Err(error) => {
                let (error, recovered) = error.into_parts();
                *ledger = Some(recovered);
                return Err(CliError::from(error));
            }
        };
    if let Err(error) = sealed.validate_commit_with_cancellation(&head, || deadline.is_expired()) {
        if matches!(error, codefire_store::StoreError::ValidationCancelled) {
            *ledger = sealed.abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
            )?;
            return Err(deadline.error("bundle sealed validation"));
        }
        *ledger = sealed.abort_and_take_decision_work_ledger(
            codefire_core::DecisionWorkTerminalReason::CorruptSubject,
        )?;
        return Err(CliError::from(error));
    }
    if let Err(error) = deadline.check("bundle sealed validation") {
        *ledger = sealed.abort_and_take_decision_work_ledger(
            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
        )?;
        return Err(error);
    }
    if let Err(error) = sealed.finalize_with_cancellation(|| deadline.is_expired()) {
        *ledger = sealed.take_decision_work_ledger()?;
        if matches!(error, codefire_store::StoreError::ValidationCancelled) {
            return Err(deadline.error("bundle snapshot revalidation"));
        }
        return Err(CliError::from(error));
    }
    *ledger = sealed.take_decision_work_ledger()?;
    let mut records = collect_object_records_with_owned_work_and_poll(
        &objects,
        &head,
        ledger
            .as_mut()
            .expect("HTTP bundle validation must return its request ledger"),
        || deadline.check("bundle object collection"),
    )?;
    if let Err(error) = deadline.check("bundle response planning") {
        records.release(
            ledger
                .as_mut()
                .expect("HTTP bundle ledger must remain present during response planning"),
        )?;
        return Err(error);
    }
    if let Err(error) = release_http_retained(
        context,
        head_frame_bytes,
        "release bundle branch head after collection",
    ) {
        records.release(
            context
                .ledger
                .as_mut()
                .expect("HTTP bundle ledger must remain present during cleanup"),
        )?;
        return Err(error);
    }
    let (records, payload_owner) = records.transfer_to_http_payload(
        context
            .ledger
            .as_ref()
            .expect("HTTP bundle ledger must own the response payload"),
    )?;
    debug_assert!(context.payload_owner.is_none());
    context.payload_owner = Some(payload_owner);
    Ok((200, json!({"branch": branch, "objects": records})))
}

#[cfg(test)]
fn handle_http_upload(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
    request: &Value,
) -> Result<(u16, Value), CliError> {
    let profile = codefire_core::DecisionWorkProfile::http_v1();
    let deadline = HttpRequestDeadline::from_profile(&profile);
    let ledger = codefire_core::DecisionWorkLedger::new(
        profile,
        codefire_core::DecisionWorkCommandKind::HttpRequest,
        "http upload",
    )
    .map_err(|error| upload_work_error(error, "initialize"))?;
    let mut owned_ledger = Some(ledger);
    let mut prepared = None;
    handle_http_upload_with_ledger(
        storage_root,
        org,
        app,
        branch_name,
        request,
        &mut HttpRouteContext {
            ledger: &mut owned_ledger,
            prepared: &mut prepared,
            deadline: &deadline,
            payload_owner: None,
        },
    )
}

#[cfg(test)]
fn handle_http_upload_with_ledger(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    let mut route_intent = None;
    handle_http_upload_with_request_source(
        storage_root,
        org,
        app,
        branch_name,
        Some(request),
        &mut route_intent,
        context,
    )
}

fn handle_http_upload_with_route_intent(
    storage_root: &Path,
    route_intent: &mut Option<HttpMutationRouteIntent>,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    handle_http_upload_with_request_source(storage_root, "", "", "", None, route_intent, context)
}

fn prepare_borrowed_upload_overlay_input(
    records: &[Value],
    overlay_document_bytes: u64,
    context: &mut HttpRouteContext<'_>,
) -> Result<codefire_store::SealedCommitOverlayValidationInput, CliError> {
    context
        .ledger
        .as_mut()
        .expect("borrowed upload ledger is present for overlay clone reservation")
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                overlay_document_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "reserve borrowed upload overlay clones"))?;
    let mut overlay_documents = Vec::with_capacity(records.len());
    let cloned = (|| -> Result<(), CliError> {
        for record in records {
            context.deadline.check("borrowed upload overlay parsing")?;
            overlay_documents.push(codefire_store::object_record_document_from_value(
                record.clone(),
            )?);
        }
        Ok(())
    })();
    if let Err(error) = cloned {
        drop(overlay_documents);
        context
            .ledger
            .as_mut()
            .expect("borrowed upload ledger remains present on overlay failure")
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                overlay_document_bytes,
            )
            .map_err(|accounting| {
                upload_work_error(accounting, "release failed borrowed upload overlay clones")
            })?;
        return Err(error);
    }
    let owned_ledger = context
        .ledger
        .take()
        .expect("borrowed upload ledger is present before sealed validation");
    match codefire_store::prepare_sealed_commit_overlay_validation_input_with_cancellation_and_temporary_transfer(
        overlay_documents,
        owned_ledger,
        overlay_document_bytes,
        || context.deadline.is_expired(),
    ) {
        Ok(input) => Ok(input),
        Err(error) => {
            let (error, mut ledger) = error.discard_documents_and_recover_ledger();
            let temporary = ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default();
            let cleanup = if temporary >= overlay_document_bytes && overlay_document_bytes > 0 {
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        overlay_document_bytes,
                    )
                    .map_err(|accounting| {
                        upload_work_error(
                            accounting,
                            "release rejected borrowed upload overlay clones",
                        )
                    })
            } else {
                Ok(())
            };
            *context.ledger = Some(ledger);
            cleanup.and(Err(CliError::from(error)))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_http_upload_with_request_source(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
    borrowed_request: Option<&Value>,
    route_intent: &mut Option<HttpMutationRouteIntent>,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("upload planning")?;
    if let Some(intent) = route_intent.as_mut() {
        intent.prepare_upload_initial_from_routing(
            storage_root,
            context
                .ledger
                .as_mut()
                .expect("upload derived-intent ledger must be present"),
        )?;
    }
    let production_route = route_intent.is_some();
    let mut direct_upload = if !production_route {
        let request = borrowed_request.expect("direct upload route retains its borrowed request");
        Some(HttpUploadDerivedIntent {
            project_root: http_project_root(storage_root, org, app),
            remote_branch: branch_name.to_string(),
            head: required_string(request, &["head"])?,
            local_branch: request
                .get("branch")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            actor: request
                .get("actor")
                .and_then(Value::as_str)
                .unwrap_or("local")
                .to_string(),
            request_target: super::signatures::remote_request_target(
                org,
                app,
                "upload",
                Some(branch_name),
                None,
            ),
            active_project_root: None,
            objects: None,
            publication_request: None,
            response: None,
        })
    } else {
        None
    };
    let (
        request,
        upload,
        route_ledger_identity,
        route_request_retained,
        mut route_derived_retained,
        route_routing_released,
    ) = if let Some(intent) = route_intent.as_mut() {
        let routing_released = intent.source_routing.is_none() && intent.routing.is_none();
        let HttpMutationRouteIntent {
            ledger_identity,
            request,
            request_retained_bytes,
            derived_retained_bytes,
            derived,
            ..
        } = intent;
        let HttpMutationDerivedIntent::Upload(upload) = derived else {
            unreachable!("prepared upload route must own upload-derived inputs")
        };
        (
            request
                .as_ref()
                .expect("production upload route retains its canonical request"),
            upload.as_mut(),
            Some(*ledger_identity),
            Some(
                request_retained_bytes.expect("production upload route retains its request scalar"),
            ),
            Some(derived_retained_bytes),
            routing_released,
        )
    } else {
        (
            borrowed_request.expect("direct upload route retains its borrowed request"),
            direct_upload
                .as_mut()
                .expect("direct upload route retains its derived inputs"),
            None,
            None,
            None,
            false,
        )
    };
    let project_root = upload.project_root.as_path();
    let branch_name = upload.remote_branch.as_str();
    let head = upload.head.as_str();
    let local_branch = upload.local_branch.as_str();
    let actor = upload.actor.as_str();
    let request_target = upload.request_target.as_str();
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
    let request_payload_evidence = if production_route {
        Some(
            super::signatures::preflight_remote_request_signature_with_payload_evidence_and_poll(
                project_root,
                "upload",
                actor,
                request_target,
                request,
                request.get("request_signature"),
                context
                    .ledger
                    .as_mut()
                    .expect("upload signature preflight ledger must be present"),
                || context.deadline.check("upload request signature preflight"),
            )?,
        )
    } else {
        super::signatures::preflight_remote_request_signature_with_work_and_poll(
            project_root,
            "upload",
            actor,
            request_target,
            request,
            request.get("request_signature"),
            context
                .ledger
                .as_mut()
                .expect("upload signature preflight ledger must be present"),
            || context.deadline.check("upload request signature preflight"),
        )?;
        None
    };
    let bootstrap_footprint =
        http_upload_bootstrap_footprint(request, branch_name, local_branch, actor, head, records)?;
    let ledger = context
        .ledger
        .as_mut()
        .expect("upload bootstrap ledger must be present");
    let project_preparation = acquire_with_http_deadline(
        request,
        context.deadline,
        "upload project gate",
        |options| {
            RemoteProjectGate::prepare_exclusive_or_bootstrap_with_ledger(
                project_root,
                options,
                ledger,
                bootstrap_footprint,
            )
        },
    );
    let mut project_preparation = Some(match project_preparation {
        Ok(preparation) => preparation,
        Err(error)
            if error
                .to_string()
                .contains("remote bootstrap footprint exceeds bounded recovery limits") =>
        {
            return Err(http_upload_bootstrap_budget_error(bootstrap_footprint));
        }
        Err(error) => return Err(error),
    });
    macro_rules! fail_bootstrapped_upload {
        ($error:expr) => {{
            let primary = $error;
            let cleanup = cancel_failed_http_upload_bootstrap(&mut project_preparation, context);
            return Err(combine_http_upload_bootstrap_error(primary, cleanup));
        }};
    }
    macro_rules! try_bootstrapped_upload {
        ($result:expr) => {{
            match $result {
                Ok(value) => value,
                Err(error) => fail_bootstrapped_upload!(error),
            }
        }};
    }
    let project_gate = project_preparation
        .as_ref()
        .expect("upload project preparation remains owned")
        .gate();
    let active_root_source = project_gate.project().display_root();
    if let (Some(ledger_identity), Some(derived_retained_bytes)) =
        (route_ledger_identity, route_derived_retained.as_deref_mut())
    {
        let retained_bytes = try_bootstrapped_upload!(sum_http_owner_bytes([
            try_bootstrapped_upload!(logical_path_owner_bytes(active_root_source)),
            try_bootstrapped_upload!(logical_joined_path_owner_bytes(
                active_root_source,
                "objects"
            )),
        ]));
        try_bootstrapped_upload!(HttpMutationRouteIntent::acquire_derived_retained_parts(
            ledger_identity,
            derived_retained_bytes,
            context
                .ledger
                .as_mut()
                .expect("upload Store-path ledger must be present"),
            retained_bytes,
            "reserve HTTP upload Store path ownership",
        ));
    }
    upload.active_project_root = Some(active_root_source.to_path_buf());
    upload.objects = Some(active_root_source.join("objects"));
    let active_project_root = upload
        .active_project_root
        .as_deref()
        .expect("upload route retains its active project root");
    let objects = upload
        .objects
        .as_deref()
        .expect("upload route retains its object-store path");
    let preflight = try_bootstrapped_upload!(preflight_http_upload(
        objects,
        head,
        records,
        context
            .ledger
            .as_mut()
            .expect("upload ledger must be present"),
        context.deadline,
    ));
    try_bootstrapped_upload!(context.deadline.check("upload object preflight"));
    let stable_project_id = project_gate.project().stable_id();
    let idempotency_key = request
        .get("idempotency_key")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let semantic_bytes = try_bootstrapped_upload!(account_compact_json(
        context
            .ledger
            .as_mut()
            .expect("upload publication measurement ledger must be present"),
        &AccountedUploadSemantic {
            stable_project_id,
            remote_branch: branch_name,
            local_branch,
            head,
            object_set_digest: preflight.object_set_digest.as_str(),
        },
        codefire_core::DecisionWorkPhase::PublicationPreflight,
        "measure HTTP upload publication semantic owner",
    ));
    let response_bytes = try_bootstrapped_upload!(account_compact_json(
        context
            .ledger
            .as_mut()
            .expect("upload response measurement ledger must be present"),
        &AccountedUploadResponse {
            branch: branch_name,
            head,
        },
        codefire_core::DecisionWorkPhase::PublicationPreflight,
        "measure HTTP upload response owner",
    ));
    let publication_bytes = try_bootstrapped_upload!(sum_http_owner_bytes([
        try_bootstrapped_upload!(logical_string_owner_bytes("remote_upload")),
        try_bootstrapped_upload!(logical_string_owner_bytes(actor)),
        try_bootstrapped_upload!(idempotency_key.map(logical_string_owner_bytes).transpose())
            .unwrap_or_default(),
        codefire_core::LOGICAL_FRAME_BASE_BYTES,
        try_bootstrapped_upload!(logical_string_owner_bytes(head)),
        semantic_bytes,
    ]));
    if let (Some(ledger_identity), Some(derived_retained_bytes)) =
        (route_ledger_identity, route_derived_retained.as_deref_mut())
    {
        let retained_bytes =
            try_bootstrapped_upload!(sum_http_owner_bytes([publication_bytes, response_bytes,]));
        try_bootstrapped_upload!(HttpMutationRouteIntent::acquire_derived_retained_parts(
            ledger_identity,
            derived_retained_bytes,
            context
                .ledger
                .as_mut()
                .expect("upload publication-derived ledger must be present"),
            retained_bytes,
            "reserve HTTP upload publication-derived ownership",
        ));
    }
    upload.publication_request = Some(upload_publication(
        actor,
        idempotency_key.map(str::to_string),
        &UploadPublicationIdentity {
            stable_project_id,
            remote_branch: branch_name,
            local_branch,
            head,
            object_set_digest: preflight.object_set_digest.as_str(),
        },
    ));
    upload.response = Some(upload_result(branch_name, head));
    let publication_request = upload
        .publication_request
        .as_ref()
        .expect("upload route retains its publication request");
    try_bootstrapped_upload!(preflight_publication_signature_with_recovery_and_poll(
        project_gate,
        active_project_root,
        publication_request,
        "upload",
        actor,
        request_target,
        request,
        request.get("request_signature"),
        context
            .ledger
            .as_mut()
            .expect("upload nonce preflight ledger must be present"),
        || context.deadline.check("upload request nonce preflight"),
    ));

    let validation_ledger = context
        .ledger
        .as_ref()
        .expect("upload validation ledger must be present");
    if let (Some(ledger_identity), Some(request_retained), Some(derived_retained)) = (
        route_ledger_identity,
        route_request_retained,
        route_derived_retained.as_deref().copied(),
    ) {
        let allow_external_permits = match project_preparation.as_ref() {
            Some(RemoteProjectExclusivePreparation::Unpublished(prepared)) => {
                prepared.owns_prepublication_permits(validation_ledger)
            }
            _ => false,
        };
        let expected = try_bootstrapped_upload!(request_retained
            .checked_add(derived_retained)
            .ok_or_else(|| CliError::InvalidRepository(
                "HTTP upload route ownership size overflow".to_string()
            )));
        let current = &validation_ledger.snapshot().current;
        let retained = current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let structural = upload_store_structure_is_complete(route_routing_released, upload);
        #[cfg(test)]
        record_http_mutation_store_binding(expected, retained, structural);
        let unrelated = current.iter().any(|(dimension, amount)| {
            *dimension != codefire_core::DecisionWorkDimension::RetainedBytes && *amount != 0
        });
        if !validation_ledger.matches_identity(ledger_identity)
            || !structural
            || retained != expected
            || unrelated
            || (!allow_external_permits && validation_ledger.has_open_permits())
        {
            fail_bootstrapped_upload!(CliError::InvalidRepository(
                "HTTP upload Store binding requires exact route-derived ownership".to_string(),
            ));
        }
    } else if validation_ledger
        .snapshot()
        .current
        .values()
        .any(|amount| *amount != 0)
    {
        fail_bootstrapped_upload!(CliError::InvalidRepository(
            "upload Store binding requires all route-owned current memory to be released"
                .to_string(),
        ));
    }

    let overlay_document_bytes = preflight.overlay_clone_bytes;
    let borrowed_overlay_input = if borrowed_request.is_some() {
        Some(try_bootstrapped_upload!(
            prepare_borrowed_upload_overlay_input(records, overlay_document_bytes, context)
        ))
    } else {
        None
    };
    let overlay_input = if let Some(intent) = route_intent.as_mut() {
        let owned_ledger = context
            .ledger
            .take()
            .expect("upload ledger must be present before object handoff");
        match intent.prepare_upload_overlay_input(
            owned_ledger,
            overlay_document_bytes,
            context.deadline,
        ) {
            Ok(input) => input,
            Err(failure) => {
                let (error, ledger) = failure.into_parts();
                *context.ledger = Some(ledger);
                fail_bootstrapped_upload!(error);
            }
        }
    } else {
        borrowed_overlay_input.expect("borrowed upload prepared its overlay input")
    };
    let post_handoff_upload = route_intent
        .as_ref()
        .map(HttpMutationRouteIntent::upload)
        .or(direct_upload.as_ref())
        .expect("upload object handoff retains its derived owner");
    let active_project_root = post_handoff_upload
        .active_project_root
        .as_deref()
        .expect("upload object handoff retains its active project root");
    let objects = post_handoff_upload
        .objects
        .as_deref()
        .expect("upload object handoff retains its object-store path");
    let head = post_handoff_upload.head.as_str();
    let actor = post_handoff_upload.actor.as_str();
    let request_target = post_handoff_upload.request_target.as_str();
    let publication_request = post_handoff_upload
        .publication_request
        .as_ref()
        .expect("upload object handoff retains its publication request");
    let mut session = match codefire_store::begin_sealed_commit_overlay_validation_session_with_recoverable_ledger_and_cancellation(
        objects,
        overlay_input,
        || context.deadline.is_expired(),
    ) {
            Ok(session) => session,
            Err(error) => {
                let (error, recovered) = error.discard_documents_and_recover_ledger();
                *context.ledger = Some(recovered);
                if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                    fail_bootstrapped_upload!(context.deadline.error("upload overlay binding"));
                }
                fail_bootstrapped_upload!(CliError::from(error));
            }
        };
    if let Err(error) =
        session.validate_commit_with_cancellation(head, || context.deadline.is_expired())
    {
        *context.ledger = Some(if route_intent.is_some() {
            discard_http_validation_session(&mut session)
        } else {
            abort_http_validation_session(&mut session, http_store_terminal_reason(&error))
        });
        if matches!(error, codefire_store::StoreError::ValidationCancelled) {
            fail_bootstrapped_upload!(context.deadline.error("upload sealed validation"));
        }
        fail_bootstrapped_upload!(CliError::from(error));
    }
    let bound = match session.bind_prepublication_validation(&publication_request.object_roots) {
        Ok(bound) => bound,
        Err(error) => {
            let (error, mut session) = error.into_parts();
            *context.ledger = Some(discard_http_validation_session(&mut session));
            fail_bootstrapped_upload!(http_store_error(error));
        }
    };
    let mut owner = match PreparedHttpMutationTransaction::from_bound_validation(bound) {
        Ok(owner) => owner,
        Err(failure) => {
            let (error, mut ledger) =
                recover_http_validation_build_failure(failure, route_intent.is_some());
            if let Some(mut intent) = route_intent.take() {
                if let Err(cleanup) = intent.release(&mut ledger) {
                    fail_closed_http_mutation_drop(format!(
                        "upload validation-owner construction failed ({error}) and route-intent cleanup failed: {cleanup}"
                    ));
                }
            }
            *context.ledger = Some(ledger);
            fail_bootstrapped_upload!(error);
        }
    };

    let signature_request = if let Some(intent) = route_intent.as_ref() {
        intent.upload_request_after_object_handoff()?
    } else {
        borrowed_request.expect("borrowed upload retains its signature request")
    };
    let pending_nonce = if route_intent.is_some() {
        validate_publication_signature_with_recovery_evidence_and_poll(
            project_gate,
            active_project_root,
            publication_request,
            "upload",
            actor,
            request_target,
            signature_request,
            signature_request.get("request_signature"),
            request_payload_evidence
                .as_ref()
                .expect("production upload retains its request payload evidence"),
            owner
                .ledger
                .as_mut()
                .expect("upload mutation owner retains its ledger"),
            || {
                context
                    .deadline
                    .check("upload request signature validation")
            },
        )
    } else {
        validate_publication_signature_with_recovery_and_poll(
            project_gate,
            active_project_root,
            publication_request,
            "upload",
            actor,
            request_target,
            signature_request,
            signature_request.get("request_signature"),
            owner
                .ledger
                .as_mut()
                .expect("upload mutation owner retains its ledger"),
            || {
                context
                    .deadline
                    .check("upload request signature validation")
            },
        )
    };
    if route_intent.is_some() {
        if let Err(error) = owner.attach_route_intent(route_intent) {
            if let Some(mut intent) = route_intent.take() {
                if let Err(cleanup) = intent.release(
                    owner
                        .ledger
                        .as_mut()
                        .expect("upload rejected route intent retains its ledger"),
                ) {
                    fail_closed_http_mutation_drop(format!(
                        "upload route-intent attachment failed ({error}) and cleanup failed: {cleanup}"
                    ));
                }
            }
            *context.ledger = Some(cancel_http_upload_mutation_owner(
                &mut owner,
                &project_preparation,
            ));
            drop(owner);
            fail_bootstrapped_upload!(error);
        }
    }
    let mut pending_nonce = match pending_nonce {
        Ok(nonce) => Some(nonce),
        Err(error) => {
            *context.ledger = Some(cancel_http_upload_mutation_owner(
                &mut owner,
                &project_preparation,
            ));
            drop(owner);
            fail_bootstrapped_upload!(error);
        }
    };
    let mut authority_accounting = None;
    let mut fresh = false;
    let publication_plan = {
        let planning_route_intent = owner.route_intent.take();
        let planning_upload = planning_route_intent
            .as_ref()
            .map(HttpMutationRouteIntent::upload)
            .or(direct_upload.as_ref())
            .expect("upload planning retains its typed derived owner");
        let active_project_root = planning_upload
            .active_project_root
            .as_deref()
            .expect("upload planning retains its active project root");
        let publication_request = planning_upload
            .publication_request
            .as_ref()
            .expect("upload planning retains its publication request");
        let response = planning_upload
            .response
            .as_ref()
            .expect("upload planning retains its response");
        let local_branch = planning_upload.local_branch.as_str();
        let actor = planning_upload.actor.as_str();
        let head = planning_upload.head.as_str();
        let remote_branch = planning_upload.remote_branch.as_str();
        let result = owner.plan_publication(
            project_gate,
            active_project_root,
            publication_request,
            &mut pending_nonce,
            response,
            |subject, ledger| {
                let (values, accounting) = prepare_upload_authority_values(
                    UploadAuthorityInput {
                        project_root: active_project_root,
                        remote_branch,
                        local_branch,
                        actor,
                        head,
                        subject,
                    },
                    ledger,
                    context.deadline,
                )?;
                authority_accounting = Some(accounting);
                fresh = true;
                Ok(values.into_writes(remote_branch))
            },
        );
        owner.route_intent = planning_route_intent;
        result
    };
    let release = authority_accounting.map_or(Ok(()), |mut accounting| {
        accounting.release(
            owner
                .ledger
                .as_mut()
                .expect("upload mutation owner retains its ledger"),
        )
    });
    release_http_pending_nonce(&mut pending_nonce, &mut owner, "upload");
    if let Err(error) = release {
        *context.ledger = Some(cancel_http_upload_mutation_owner(
            &mut owner,
            &project_preparation,
        ));
        drop(owner);
        fail_bootstrapped_upload!(error);
    }
    if let Err(error) = publication_plan {
        *context.ledger = Some(cancel_http_upload_mutation_owner(
            &mut owner,
            &project_preparation,
        ));
        drop(owner);
        fail_bootstrapped_upload!(error);
    }

    if fresh {
        let batch = owner.prepare_and_attach_overlay_object_batch(context.deadline);
        if let Err(error) = batch {
            *context.ledger = Some(cancel_http_upload_mutation_owner(
                &mut owner,
                &project_preparation,
            ));
            drop(owner);
            fail_bootstrapped_upload!(error);
        }
    }
    let response_preparation = if owner.route_intent.is_some() {
        owner.prepare_and_attach_responses_from_upload_route(200, context.deadline)
    } else {
        owner.prepare_and_attach_responses(
            200,
            direct_upload
                .as_ref()
                .and_then(|upload| upload.response.as_ref())
                .expect("direct upload retains its response"),
            context.deadline,
        )
    };
    if let Err(error) = response_preparation {
        *context.ledger = Some(cancel_http_upload_mutation_owner(
            &mut owner,
            &project_preparation,
        ));
        drop(owner);
        fail_bootstrapped_upload!(error);
    }
    if let Err(error) = owner.release_route_intent() {
        *context.ledger = Some(cancel_http_upload_mutation_owner(
            &mut owner,
            &project_preparation,
        ));
        drop(owner);
        fail_bootstrapped_upload!(error);
    }

    if let Err(error) = owner.revalidate_and_seal(|| context.deadline.is_expired()) {
        let error = if matches!(
            &error,
            CliError::Store(store)
                if matches!(store.as_ref(), codefire_store::StoreError::ValidationCancelled)
        ) {
            context.deadline.error("upload prepublication revalidation")
        } else {
            error
        };
        *context.ledger = Some(cancel_http_upload_mutation_owner(
            &mut owner,
            &project_preparation,
        ));
        drop(owner);
        fail_bootstrapped_upload!(error);
    }
    if owner.commit_effects_after_seal().is_ok() {
        if let Err(error) = owner.validate_visibility_terminal() {
            *context.ledger = Some(cancel_http_upload_mutation_owner(
                &mut owner,
                &project_preparation,
            ));
            drop(owner);
            fail_bootstrapped_upload!(error);
        }
        let mut terminal = owner.into_visibility_terminal_after_validation();
        match project_preparation
            .take()
            .expect("upload project preparation remains owned until visibility")
        {
            RemoteProjectExclusivePreparation::Existing(_) => {
                if let Err(error) = terminal.select_success() {
                    *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
                    fail_bootstrapped_upload!(error);
                }
            }
            RemoteProjectExclusivePreparation::Unpublished(prepared) => {
                let mut armed = match (*prepared).arm_with_ledger(
                    terminal
                        .ledger
                        .as_mut()
                        .expect("upload terminal retains bootstrap ledger"),
                ) {
                    Ok(armed) => armed,
                    Err(failure) => {
                        let (mut prepared, error) = *failure;
                        let ledger = terminal
                            .ledger
                            .as_mut()
                            .expect("upload terminal retains rejected bootstrap ledger");
                        let cleanup = prepared.cancel_with_ledger(ledger);
                        *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
                        fail_bootstrapped_upload!(combine_http_upload_bootstrap_error(
                            error, cleanup
                        ));
                    }
                };
                let proof = match terminal.publish_bootstrap_success(&mut armed) {
                    Ok(proof) => proof,
                    Err(error) => {
                        *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
                        fail_bootstrapped_upload!(error);
                    }
                };
                drop(armed);
                let handoff = terminal.into_validated_http_bootstrap_handoff(proof);
                let (ledger, prepared) = handoff.into_http_parts();
                *context.ledger = Some(ledger);
                *context.prepared = Some(prepared);
                return Ok((200, Value::Null));
            }
        }
        if let Err(error) = terminal.validate_http_handoff() {
            *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
            fail_bootstrapped_upload!(error);
        }
        let (ledger, prepared) = terminal.into_http_parts_after_validation();
        *context.ledger = Some(ledger);
        *context.prepared = Some(prepared);
        return Ok((200, Value::Null));
    }

    if let Err(error) = owner.select_durable_after_commit_error() {
        *context.ledger = Some(cancel_http_upload_mutation_owner(
            &mut owner,
            &project_preparation,
        ));
        drop(owner);
        fail_bootstrapped_upload!(error);
    }
    let mut terminal =
        try_bootstrapped_upload!(owner.into_selected_terminal_after_effect_resolution());
    if let RemoteProjectExclusivePreparation::Unpublished(prepared) = project_preparation
        .take()
        .expect("durable upload retains project preparation")
    {
        let armed = match (*prepared).arm_with_ledger(
            terminal
                .ledger
                .as_mut()
                .expect("durable upload terminal retains bootstrap ledger"),
        ) {
            Ok(armed) => armed,
            Err(failure) => {
                let (mut prepared, error) = *failure;
                let ledger = terminal
                    .ledger
                    .as_mut()
                    .expect("durable upload terminal retains rejected bootstrap ledger");
                let cleanup = prepared.cancel_with_ledger(ledger);
                *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
                fail_bootstrapped_upload!(combine_http_upload_bootstrap_error(error, cleanup));
            }
        };
        drop(armed);
    }
    if let Err(error) = terminal.validate_http_handoff() {
        *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
        fail_bootstrapped_upload!(error);
    }
    let (ledger, prepared) = terminal.into_http_parts_after_validation();
    debug_assert_eq!(prepared.status, 503);
    *context.ledger = Some(ledger);
    *context.prepared = Some(prepared);
    Ok((503, Value::Null))
}

fn combine_http_upload_bootstrap_error(
    primary: CliError,
    cleanup: Result<(), CliError>,
) -> CliError {
    match cleanup {
        Ok(()) => primary,
        Err(cleanup) => CliError::InvalidRepository(format!(
            "HTTP upload failed: {primary}; bootstrap cleanup/accounting failed: {cleanup}"
        )),
    }
}

#[cfg(test)]
thread_local! {
    static INJECT_HTTP_UPLOAD_BOOTSTRAP_CALLBACK: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    static HTTP_UPLOAD_BOOTSTRAP_HANDOFF_VALIDATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn reset_http_upload_bootstrap_handoff_validations() {
    HTTP_UPLOAD_BOOTSTRAP_HANDOFF_VALIDATIONS.with(|count| count.set(0));
}

#[cfg(test)]
fn http_upload_bootstrap_handoff_validations() -> u64 {
    HTTP_UPLOAD_BOOTSTRAP_HANDOFF_VALIDATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
fn inject_http_upload_bootstrap_collision(project_root: &Path) -> Result<(), CliError> {
    match INJECT_HTTP_UPLOAD_BOOTSTRAP_CALLBACK.replace(0) {
        0 => Ok(()),
        1 => {
            fs::create_dir(project_root)?;
            Ok(())
        }
        2 => Err(CliError::RequestDeadlineExceeded {
            phase: "injected upload bootstrap callback",
            limit_ms: 1,
        }),
        _ => unreachable!("invalid upload bootstrap callback injection"),
    }
}

fn cancel_failed_http_upload_bootstrap(
    preparation: &mut Option<RemoteProjectExclusivePreparation>,
    context: &mut HttpRouteContext<'_>,
) -> Result<(), CliError> {
    let Some(RemoteProjectExclusivePreparation::Unpublished(prepared)) = preparation.as_mut()
    else {
        return Ok(());
    };
    let ledger = context.ledger.as_mut().ok_or_else(|| {
        CliError::InvalidRepository(
            "failed HTTP upload did not recover its bootstrap ledger".to_string(),
        )
    })?;
    prepared.cancel_with_ledger(ledger)?;
    drop(preparation.take());
    Ok(())
}

fn http_upload_bootstrap_footprint(
    request: &Value,
    remote_branch: &str,
    local_branch: &str,
    actor: &str,
    head: &str,
    records: &[Value],
) -> Result<BootstrapStageFootprint, CliError> {
    struct JsonByteCounter(u64);
    impl Write for JsonByteCounter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| std::io::Error::other("upload bootstrap JSON size overflow"))?;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    fn compact_len<T: Serialize + ?Sized>(value: &T) -> Result<u64, CliError> {
        let mut counter = JsonByteCounter(0);
        serde_json::to_writer(&mut counter, value)?;
        Ok(counter.0)
    }
    fn rendered_len(value: &Value) -> Result<u64, CliError> {
        let mut counter = JsonByteCounter(0);
        serde_json::to_writer_pretty(&mut counter, value)?;
        counter.0.checked_add(1).ok_or_else(|| {
            CliError::InvalidRepository("upload bootstrap object size overflow".to_string())
        })
    }

    let object_payload_bytes = records.iter().try_fold(0_u64, |total, record| {
        total.checked_add(rendered_len(record)?).ok_or_else(|| {
            CliError::InvalidRepository("upload bootstrap object size overflow".to_string())
        })
    })?;
    let object_entries = u64::try_from(records.len())
        .ok()
        .and_then(|count| count.checked_mul(2))
        .and_then(|count| count.checked_add(1)) // persistent object-stage lock
        .ok_or_else(|| {
            CliError::InvalidRepository("upload bootstrap object count overflow".to_string())
        })?;
    let object_payload_bytes = object_payload_bytes.checked_mul(2).ok_or_else(|| {
        CliError::InvalidRepository("upload bootstrap object size overflow".to_string())
    })?;

    #[derive(Serialize)]
    struct PlannedBootstrapBranch<'a> {
        version: u64,
        name: &'a str,
        head: &'a str,
        generation: u64,
        uploaded_from: &'a str,
        uploaded_by: &'a str,
        transport: &'static str,
        updated_at: &'static str,
    }
    #[derive(Serialize)]
    struct PlannedBootstrapGeneration {
        current_generation: u64,
    }
    // `now_iso_utc()` uses this fixed-width UTC representation. Only its byte
    // length participates in canonical JSON size planning.
    const FIXED_WIDTH_TIMESTAMP: &str = "2000-01-01T00:00:00Z";
    let authority_payload_bytes = compact_len(&PlannedBootstrapBranch {
        version: 1,
        name: remote_branch,
        head,
        generation: 1,
        uploaded_from: local_branch,
        uploaded_by: actor,
        transport: "http",
        updated_at: FIXED_WIDTH_TIMESTAMP,
    })?
    .checked_add(compact_len(&PlannedBootstrapGeneration {
        current_generation: 1,
    })?)
    .ok_or_else(|| {
        CliError::InvalidRepository("upload bootstrap authority size overflow".to_string())
    })?;
    let publication = missing_upload_publication_footprint(
        authority_payload_bytes,
        remote_idempotency_key(request).is_some(),
    )?;
    let entries = object_entries
        .checked_add(publication.entries)
        .ok_or_else(|| {
            CliError::InvalidRepository("upload bootstrap footprint overflow".to_string())
        })?;
    let payload_bytes = object_payload_bytes
        .checked_add(publication.payload_bytes)
        .ok_or_else(|| {
            CliError::InvalidRepository("upload bootstrap footprint overflow".to_string())
        })?;
    Ok(BootstrapStageFootprint::new(entries, payload_bytes))
}

fn http_upload_bootstrap_budget_error(footprint: BootstrapStageFootprint) -> CliError {
    let (dimension, observed, limit) =
        if footprint.entries > BootstrapStageFootprint::maximum_entries() {
            (
                codefire_core::DecisionWorkDimension::GraphNodes,
                footprint.entries,
                BootstrapStageFootprint::maximum_entries(),
            )
        } else {
            let limit = BootstrapStageFootprint::maximum_payload_bytes(footprint.entries)
                .expect("entry-bounded bootstrap footprint has a byte limit");
            (
                codefire_core::DecisionWorkDimension::AggregateBytes,
                footprint.payload_bytes,
                limit,
            )
        };
    CliError::ResourceBudget(Box::new(codefire_core::DecisionWorkBudgetExceeded {
        kind: "resource_budget_exceeded".to_string(),
        exit_code: codefire_core::RESOURCE_BUDGET_EXIT_CODE,
        profile: codefire_core::DECISION_WORK_PROFILE_V1.to_string(),
        dimension,
        observed,
        limit,
        phase: codefire_core::DecisionWorkPhase::Staging,
        command: "http upload bootstrap".to_string(),
        continuation_supported: false,
    }))
}

#[derive(Default)]
struct UploadAuthorityAccounting {
    policy_retained: u64,
    branch_retained: u64,
    generation_retained: u64,
    branch_postimage_retained: u64,
    generation_postimage_retained: u64,
}

impl UploadAuthorityAccounting {
    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        for (amount, operation) in [
            (
                &mut self.generation_postimage_retained,
                "release upload generation postimage",
            ),
            (
                &mut self.branch_postimage_retained,
                "release upload branch postimage",
            ),
            (
                &mut self.generation_retained,
                "release upload generation authority JSON",
            ),
            (
                &mut self.branch_retained,
                "release upload branch authority JSON",
            ),
            (
                &mut self.policy_retained,
                "release upload commit-signature policy",
            ),
        ] {
            release_apply_owned_amount(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                amount,
                operation,
            )?;
        }
        Ok(())
    }
}

struct UploadAuthorityValues {
    generation_before: Option<Value>,
    generation_after: Value,
    branch_before: Option<Value>,
    branch_after: Value,
}

impl UploadAuthorityValues {
    fn into_writes(self, remote_branch: &str) -> Vec<AuthorityWrite> {
        upload_writes(UploadPostimages {
            remote_branch,
            generation_before: self.generation_before,
            generation_after: self.generation_after,
            branch_before: self.branch_before,
            branch_after: self.branch_after,
        })
    }
}

struct UploadAuthorityInput<'a, 'subject> {
    project_root: &'a Path,
    remote_branch: &'a str,
    local_branch: &'a str,
    actor: &'a str,
    head: &'a str,
    subject: &'subject codefire_store::ProvisionalSealedValidationSubject<'subject>,
}

fn prepare_upload_authority_values(
    input: UploadAuthorityInput<'_, '_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<(UploadAuthorityValues, UploadAuthorityAccounting), CliError> {
    let UploadAuthorityInput {
        project_root,
        remote_branch,
        local_branch,
        actor,
        head,
        subject,
    } = input;
    let mut accounting = UploadAuthorityAccounting::default();
    let prepared = (|| -> Result<UploadAuthorityValues, CliError> {
        let branch_before = match read_optional_http_json_accounted_with_ledger(
            &remote_branch_path(project_root, remote_branch),
            ledger,
            deadline,
            "upload branch authority read",
        )? {
            Some(accounted) => {
                let (branch, retained) = accounted.into_parts();
                accounting.branch_retained = retained;
                validate_remote_branch_record(&branch)?;
                let current_head = required_string(&branch, &["head"])?;
                if !is_ancestor_in_provisional_subject_with_poll(
                    subject,
                    ledger,
                    &current_head,
                    head,
                    || deadline.check("upload ancestry validation"),
                )? {
                    return Err(CliError::IdempotencyConflict(
                        "upload rejected: remote branch is not an ancestor of local branch head"
                            .to_string(),
                    ));
                }
                Some(branch)
            }
            None => None,
        };

        let accounted_policy = read_optional_http_json_accounted_with_ledger(
            &project_root.join("server_policy.json"),
            ledger,
            deadline,
            "upload commit-signature policy read",
        )?;
        let policy = accounted_policy.map(|accounted| {
            let (policy, retained) = accounted.into_parts();
            accounting.policy_retained = retained;
            policy
        });
        let signature_validation =
            super::signatures::enforce_remote_commit_signature_policy_in_provisional_subject_with_poll(
                policy.as_ref(),
                subject,
                ledger,
                head,
                || deadline.check("upload commit signature validation"),
            );
        drop(policy);
        let policy_release = release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            &mut accounting.policy_retained,
            "release upload commit-signature policy",
        );
        match (signature_validation, policy_release) {
            (Ok(()), Ok(())) => {}
            (Err(error), Ok(())) => return Err(error),
            (_, Err(error)) => return Err(error),
        }

        let generation_before = match read_optional_http_json_accounted_with_ledger(
            &project_root.join("gc_state.json"),
            ledger,
            deadline,
            "upload generation authority read",
        )? {
            Some(accounted) => {
                let (generation, retained) = accounted.into_parts();
                accounting.generation_retained = retained;
                Some(generation)
            }
            None => None,
        };
        let generation = generation_before
            .as_ref()
            .and_then(|value| value.get("current_generation"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| CliError::InvalidRepository("remote generation overflow".to_string()))?;

        #[derive(Serialize)]
        struct PlannedUploadBranch<'a> {
            version: u64,
            name: &'a str,
            head: &'a str,
            generation: u64,
            uploaded_from: &'a str,
            uploaded_by: &'a str,
            transport: &'static str,
            updated_at: &'a str,
        }
        #[derive(Serialize)]
        struct PlannedUploadGeneration {
            current_generation: u64,
        }

        let updated_at = now_iso_utc();
        let planned_branch = PlannedUploadBranch {
            version: 1,
            name: remote_branch,
            head,
            generation,
            uploaded_from: local_branch,
            uploaded_by: actor,
            transport: "http",
            updated_at: &updated_at,
        };
        accounting.branch_postimage_retained = retain_compact_http_json_with_ledger(
            ledger,
            deadline,
            &planned_branch,
            codefire_core::DecisionWorkPhase::Validation,
            "retain upload branch postimage",
        )?;
        let planned_generation = PlannedUploadGeneration {
            current_generation: generation,
        };
        accounting.generation_postimage_retained = retain_compact_http_json_with_ledger(
            ledger,
            deadline,
            &planned_generation,
            codefire_core::DecisionWorkPhase::Validation,
            "retain upload generation postimage",
        )?;

        Ok(UploadAuthorityValues {
            generation_before,
            generation_after: json!({"current_generation": generation}),
            branch_before,
            branch_after: json!({
                "version": 1,
                "name": remote_branch,
                "head": head,
                "generation": generation,
                "uploaded_from": local_branch,
                "uploaded_by": actor,
                "transport": "http",
                "updated_at": updated_at,
            }),
        })
    })();

    match prepared {
        Ok(values) => Ok((values, accounting)),
        Err(error) => {
            accounting.release(ledger)?;
            Err(error)
        }
    }
}

#[derive(Debug)]
struct HttpUploadPreflight {
    #[cfg(test)]
    incoming_closure: Vec<usize>,
    #[cfg(test)]
    existing_closure: Vec<String>,
    object_set_digest: InlineHttpObjectSetDigest,
    overlay_clone_bytes: u64,
}

const HTTP_OBJECT_SET_DIGEST_TEMPLATE: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug)]
struct InlineHttpObjectSetDigest {
    bytes: [u8; HTTP_OBJECT_SET_DIGEST_TEMPLATE.len()],
}

impl InlineHttpObjectSetDigest {
    fn copy_from(value: &str) -> Result<Self, CliError> {
        let bytes: [u8; HTTP_OBJECT_SET_DIGEST_TEMPLATE.len()] =
            value.as_bytes().try_into().map_err(|_| {
                CliError::InvalidRepository(
                    "upload object-set digest has an invalid length".to_string(),
                )
            })?;
        if !value.starts_with("sha256:")
            || !value.as_bytes()["sha256:".len()..]
                .iter()
                .all(u8::is_ascii_hexdigit)
        {
            return Err(CliError::InvalidRepository(
                "upload object-set digest has an invalid format".to_string(),
            ));
        }
        Ok(Self { bytes })
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes)
            .expect("validated upload object-set digest must remain ASCII")
    }
}

#[derive(Default)]
struct UploadPreflightWorkOwner {
    temporary_bytes: u64,
    retained_bytes: u64,
}

impl UploadPreflightWorkOwner {
    fn acquire(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        dimension: codefire_core::DecisionWorkDimension,
        amount: u64,
        reservation: codefire_core::DecisionWorkReservation,
        operation: &str,
    ) -> Result<(), CliError> {
        let owned = match dimension {
            codefire_core::DecisionWorkDimension::TemporaryBytes => &mut self.temporary_bytes,
            codefire_core::DecisionWorkDimension::RetainedBytes => &mut self.retained_bytes,
            _ => {
                return Err(CliError::InvalidRepository(
                    "upload preflight owner supports only concurrent byte dimensions".to_string(),
                ))
            }
        };
        let next = owned.checked_add(amount).ok_or_else(|| {
            CliError::InvalidRepository("upload preflight ownership overflow".to_string())
        })?;
        ledger
            .apply(reservation.acquire(dimension, amount))
            .map_err(|error| upload_work_error(error, operation))?;
        *owned = next;
        Ok(())
    }

    fn release(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        dimension: codefire_core::DecisionWorkDimension,
        amount: u64,
        operation: &str,
    ) -> Result<(), CliError> {
        let owned = match dimension {
            codefire_core::DecisionWorkDimension::TemporaryBytes => &mut self.temporary_bytes,
            codefire_core::DecisionWorkDimension::RetainedBytes => &mut self.retained_bytes,
            _ => {
                return Err(CliError::InvalidRepository(
                    "upload preflight owner supports only concurrent byte dimensions".to_string(),
                ))
            }
        };
        let next = owned.checked_sub(amount).ok_or_else(|| {
            CliError::InvalidRepository("upload preflight ownership underflow".to_string())
        })?;
        ledger
            .release(dimension, amount)
            .map_err(|error| upload_work_error(error, operation))?;
        *owned = next;
        Ok(())
    }

    fn release_all(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        let mut first_error = None;
        for (dimension, amount, operation) in [
            (
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                self.temporary_bytes,
                "release cancelled upload preflight temporary ownership",
            ),
            (
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
                "release cancelled upload preflight retained ownership",
            ),
        ] {
            if amount == 0 {
                continue;
            }
            match ledger.release(dimension, amount) {
                Ok(()) => match dimension {
                    codefire_core::DecisionWorkDimension::TemporaryBytes => {
                        self.temporary_bytes = 0
                    }
                    codefire_core::DecisionWorkDimension::RetainedBytes => self.retained_bytes = 0,
                    _ => unreachable!(),
                },
                Err(error) => {
                    let cleanup = upload_work_error(error, operation);
                    if let Some(previous) = first_error.take() {
                        first_error = Some(CliError::InvalidRepository(format!(
                            "upload preflight cleanup failed twice: {previous}; {cleanup}"
                        )));
                    } else {
                        first_error = Some(cleanup);
                    }
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

fn preflight_http_upload(
    existing_objects: &Path,
    head: &str,
    records: &[Value],
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<HttpUploadPreflight, CliError> {
    preflight_http_upload_with_polls(
        UploadPreflightInput {
            existing_objects,
            head,
            records,
            deadline,
        },
        ledger,
        UploadPreflightPolls {
            digest: || deadline.check("upload object-set digest"),
            digest_result: || deadline.check("upload object-set digest"),
            traversal: || deadline.check("upload closure validation"),
        },
    )
}

#[derive(Clone, Copy)]
struct UploadPreflightInput<'a> {
    existing_objects: &'a Path,
    head: &'a str,
    records: &'a [Value],
    deadline: &'a HttpRequestDeadline,
}

struct UploadPreflightPolls<D, R, T> {
    digest: D,
    digest_result: R,
    traversal: T,
}

#[cfg(test)]
fn preflight_http_upload_with_traversal_poll<F>(
    existing_objects: &Path,
    head: &str,
    records: &[Value],
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
    traversal_poll: F,
) -> Result<HttpUploadPreflight, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    preflight_http_upload_with_polls(
        UploadPreflightInput {
            existing_objects,
            head,
            records,
            deadline,
        },
        ledger,
        UploadPreflightPolls {
            digest: || deadline.check("upload object-set digest"),
            digest_result: || deadline.check("upload object-set digest"),
            traversal: traversal_poll,
        },
    )
}

fn preflight_http_upload_with_polls<D, R, T>(
    input: UploadPreflightInput<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut polls: UploadPreflightPolls<D, R, T>,
) -> Result<HttpUploadPreflight, CliError>
where
    D: FnMut() -> Result<(), CliError>,
    R: FnMut() -> Result<(), CliError>,
    T: FnMut() -> Result<(), CliError>,
{
    let mut work = UploadPreflightWorkOwner::default();
    let result = preflight_http_upload_inner(input, ledger, &mut work, &mut polls);
    let cleanup = work.release_all(ledger);
    match (result, cleanup) {
        (Ok(preflight), Ok(())) => Ok(preflight),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(error), Err(cleanup)) => Err(CliError::InvalidRepository(format!(
            "upload preflight failed ({error}); cleanup also failed ({cleanup})"
        ))),
    }
}

fn preflight_http_upload_inner(
    input: UploadPreflightInput<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    work: &mut UploadPreflightWorkOwner,
    polls: &mut UploadPreflightPolls<
        impl FnMut() -> Result<(), CliError>,
        impl FnMut() -> Result<(), CliError>,
        impl FnMut() -> Result<(), CliError>,
    >,
) -> Result<HttpUploadPreflight, CliError> {
    let UploadPreflightInput {
        existing_objects,
        head,
        records,
        deadline,
    } = input;
    let incoming_count = u64::try_from(records.len()).map_err(|_| {
        CliError::InvalidRepository("HTTP upload object count overflow".to_string())
    })?;
    precharge_upload_record_count(ledger, incoming_count)?;

    let record_size_index_bytes = incoming_count
        .checked_mul(u64::try_from(std::mem::size_of::<u64>()).map_err(|_| {
            CliError::InvalidRepository("HTTP upload record-size slot overflow".to_string())
        })?)
        .ok_or_else(|| {
            CliError::InvalidRepository("HTTP upload record-size index overflow".to_string())
        })?;
    work.acquire(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        record_size_index_bytes,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::Validation,
            codefire_core::DecisionWorkSurface::Atomic,
        ),
        "reserve upload record-size index",
    )?;
    let mut record_sizes = Vec::with_capacity(records.len());
    let mut overlay_clone_bytes = 0_u64;
    for record in records {
        deadline.check("upload record measurement")?;
        let logical_size = account_compact_json(
            ledger,
            record,
            codefire_core::DecisionWorkPhase::Validation,
            "account incoming compact JSON",
        )?;
        overlay_clone_bytes = overlay_clone_bytes
            .checked_add(logical_size)
            .ok_or_else(|| {
                CliError::InvalidRepository("HTTP upload overlay clone size overflow".to_string())
            })?;
        record_sizes.push(logical_size);
    }

    let mut incoming = BTreeMap::new();
    for (index, record) in records.iter().enumerate() {
        deadline.check("upload record validation")?;
        let object_id = record
            .get("object_id")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::Usage("missing required field: object_id".to_string()))?;
        let clone_bytes = record_sizes[index];
        work.acquire(
            ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            clone_bytes,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            ),
            "reserve incoming validation clone",
        )?;
        let validation = validate_object_record_value(record, object_id);
        let release = work.release(
            ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            clone_bytes,
            "release incoming validation clone",
        );
        match (validation, release) {
            (Ok(()), Ok(())) => {}
            (Err(error), Ok(())) => return Err(error),
            (_, Err(error)) => return Err(error),
        }
        if incoming.contains_key(object_id) {
            return Err(CliError::Usage(
                "remote object set contains a duplicate object ID".to_string(),
            ));
        }
        work.acquire(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::LOGICAL_FRAME_BASE_BYTES,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            ),
            "reserve upload incoming index node",
        )?;
        incoming.insert(object_id, index);
    }
    let mut object_set_digest = source_object_set_digest_ordered_with_poll(
        records,
        incoming.values().copied(),
        ledger,
        &mut polls.digest,
    )?;
    let handoff = (|| -> Result<InlineHttpObjectSetDigest, CliError> {
        (polls.digest_result)()?;
        InlineHttpObjectSetDigest::copy_from(object_set_digest.as_str())
    })();
    let source_cleanup = object_set_digest.release(ledger);
    let inline_object_set_digest = match (handoff, source_cleanup) {
        (Ok(digest), Ok(())) => digest,
        (Err(error), Ok(())) => return Err(error),
        (_, Err(error)) => return Err(error),
    };

    preflight_incoming_collisions(
        existing_objects,
        records,
        &record_sizes,
        ledger,
        deadline,
        work,
    )?;
    drop(record_sizes);
    work.release(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        record_size_index_bytes,
        "release upload record-size index",
    )?;

    let root_retained = codefire_core::logical_retained_frame_bytes(head)
        .map_err(|error| upload_work_error(error, "account root frontier"))?;
    work.acquire(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        root_retained,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .observe_maximum(codefire_core::DecisionWorkDimension::LogicalDepth, 0)
        .observe_maximum(codefire_core::DecisionWorkDimension::MaximumFrontier, 1)
        .count(
            codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
            1,
        ),
        "account root frontier",
    )?;

    let mut seen = BTreeSet::new();
    let mut stack = vec![(head.to_string(), 0_u64)];
    #[cfg(test)]
    let mut incoming_closure = Vec::new();
    #[cfg(test)]
    let mut existing_closure = Vec::new();
    while let Some((object_id, depth)) = stack.pop() {
        (polls.traversal)()?;
        let frame_bytes = codefire_core::logical_retained_frame_bytes(&object_id)
            .map_err(|error| upload_work_error(error, "release frontier"))?;
        if seen.contains(&object_id) {
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::GraphDiscovery,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .count(codefire_core::DecisionWorkCounter::Revisits, 1),
                )
                .map_err(|error| upload_work_error(error, "account revisit"))?;
            drop(object_id);
            work.release(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                frame_bytes,
                "release revisited frontier",
            )?;
            continue;
        }
        work.acquire(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            frame_bytes,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::GraphDiscovery,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
            .count(codefire_core::DecisionWorkCounter::UniqueNodesDiscovered, 1),
            "account closure object and seen node",
        )?;
        seen.insert(object_id.clone());

        let incoming_index = incoming.get(object_id.as_str()).copied();
        if let Some(index) = incoming_index {
            #[cfg(test)]
            incoming_closure.push(index);
            try_for_each_object_value_reference(&records[index], |reference| {
                push_http_upload_reference(ledger, work, &mut stack, reference, depth)
            })?;
        } else {
            let path_frame_bytes = upload_object_path_frame_bytes(existing_objects, &object_id)?;
            work.acquire(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                path_frame_bytes,
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                ),
                "reserve existing closure object path",
            )?;
            let path = codefire_store::object_record_path(existing_objects, &object_id)
                .ok_or_else(|| codefire_store::StoreError::ObjectNotFound(object_id.clone()))?;
            let physical_bytes = fs::metadata(&path)?.len();
            work.acquire(
                ledger,
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                physical_bytes,
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
                .count(codefire_core::DecisionWorkCounter::RecordReads, 1)
                .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
                "reserve existing closure object",
            )?;
            let document = codefire_store::read_object_record_document_path(&path)?;
            codefire_store::validate_object_record(
                &document.record,
                Some(&object_id),
                Some(&path),
            )?;
            drop(path);
            work.release(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                path_frame_bytes,
                "release existing closure object path",
            )?;
            let logical_bytes = transfer_parsed_object_document_owned(
                ledger,
                work,
                &document,
                physical_bytes,
                codefire_core::DecisionWorkPhase::RecordRead,
                "account existing compact JSON",
            )?;
            try_for_each_object_document_reference(&document, |reference| {
                push_http_upload_reference(ledger, work, &mut stack, reference, depth)
            })?;
            drop(document);
            #[cfg(test)]
            existing_closure.push(object_id.clone());
            work.release(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                logical_bytes,
                "release traversal retained bytes",
            )?;
        }
        drop(object_id);
        work.release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            frame_bytes,
            "release processed frontier",
        )?;
    }
    #[cfg(test)]
    incoming_closure.sort_unstable();
    #[cfg(test)]
    existing_closure.sort();
    Ok(HttpUploadPreflight {
        #[cfg(test)]
        incoming_closure,
        #[cfg(test)]
        existing_closure,
        object_set_digest: inline_object_set_digest,
        overlay_clone_bytes,
    })
}

fn push_http_upload_reference(
    ledger: &mut codefire_core::DecisionWorkLedger,
    work: &mut UploadPreflightWorkOwner,
    stack: &mut Vec<(String, u64)>,
    reference: &str,
    depth: u64,
) -> Result<(), CliError> {
    let next_depth = depth.checked_add(1).ok_or_else(|| {
        CliError::InvalidRepository("HTTP upload object depth overflow".to_string())
    })?;
    let next_frontier = u64::try_from(stack.len())
        .ok()
        .and_then(|size| size.checked_add(1))
        .ok_or_else(|| CliError::InvalidRepository("HTTP upload frontier overflow".to_string()))?;
    let retained_bytes = codefire_core::logical_retained_frame_bytes(reference)
        .map_err(|error| upload_work_error(error, "account closure edge"))?;
    work.acquire(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        retained_bytes,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphEdges, 1)
        .observe_maximum(
            codefire_core::DecisionWorkDimension::LogicalDepth,
            next_depth,
        )
        .observe_maximum(
            codefire_core::DecisionWorkDimension::MaximumFrontier,
            next_frontier,
        )
        .count(codefire_core::DecisionWorkCounter::EdgeOccurrences, 1)
        .count(
            codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
            1,
        ),
        "account closure edge",
    )?;
    stack.push((reference.to_string(), next_depth));
    Ok(())
}

fn precharge_upload_record_count(
    ledger: &mut codefire_core::DecisionWorkLedger,
    incoming_count: u64,
) -> Result<(), CliError> {
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::ObjectRecords,
                incoming_count,
            )
            .count(
                codefire_core::DecisionWorkCounter::ObjectRecords,
                incoming_count,
            ),
        )
        .map_err(|error| upload_work_error(error, "precharge incoming records"))
}

fn upload_object_path_frame_bytes(objects: &Path, object_id: &str) -> Result<u64, CliError> {
    let root_bytes = u64::try_from(objects.as_os_str().as_encoded_bytes().len()).map_err(|_| {
        CliError::InvalidRepository("HTTP upload object-root path length overflow".to_string())
    })?;
    let subdir_bytes = codefire_store::known_object_subdirs()
        .map(str::len)
        .max()
        .and_then(|length| u64::try_from(length).ok())
        .ok_or_else(|| {
            CliError::InvalidRepository("HTTP upload object subdirectory is missing".to_string())
        })?;
    let object_id_bytes = u64::try_from(object_id.len()).map_err(|_| {
        CliError::InvalidRepository("HTTP upload object ID length overflow".to_string())
    })?;
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(root_bytes)
        .and_then(|bytes| bytes.checked_add(1))
        .and_then(|bytes| bytes.checked_add(subdir_bytes))
        .and_then(|bytes| bytes.checked_add(1))
        .and_then(|bytes| bytes.checked_add(object_id_bytes))
        .and_then(|bytes| bytes.checked_add(5))
        .ok_or_else(|| {
            CliError::InvalidRepository("HTTP upload object path size overflow".to_string())
        })
}

fn preflight_incoming_collisions(
    existing_objects: &Path,
    records: &[Value],
    record_sizes: &[u64],
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
    work: &mut UploadPreflightWorkOwner,
) -> Result<(), CliError> {
    for (index, record) in records.iter().enumerate() {
        deadline.check("upload collision validation")?;
        let object_id = record
            .get("object_id")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::Usage("missing required field: object_id".to_string()))?;
        let path_frame_bytes = upload_object_path_frame_bytes(existing_objects, object_id)?;
        work.acquire(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            path_frame_bytes,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            ),
            "reserve upload collision object path",
        )?;
        let Some(path) = codefire_store::object_record_path(existing_objects, object_id) else {
            work.release(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                path_frame_bytes,
                "release absent upload collision object path",
            )?;
            continue;
        };
        if !path.exists() {
            drop(path);
            work.release(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                path_frame_bytes,
                "release missing upload collision object path",
            )?;
            continue;
        }
        let physical_bytes = fs::metadata(&path)?.len();
        work.acquire(
            ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            physical_bytes,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .count(codefire_core::DecisionWorkCounter::RecordReads, 1),
            "reserve collision check",
        )?;
        let existing = codefire_store::read_object_record_document_path(&path)?;
        let logical_bytes = transfer_parsed_object_document_owned(
            ledger,
            work,
            &existing,
            physical_bytes,
            codefire_core::DecisionWorkPhase::Validation,
            "account collision compact JSON",
        )?;
        let requested_bytes = record_sizes[index];
        work.acquire(
            ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            requested_bytes,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            ),
            "reserve collision requested clone",
        )?;
        let requested = codefire_store::object_record_document_from_value(record.clone())?;
        if existing != requested {
            return Err(CliError::InvalidRepository(format!(
                "immutable object document diverges at {}",
                path.display()
            )));
        }
        drop(path);
        drop(requested);
        drop(existing);
        work.release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            path_frame_bytes,
            "release upload collision object path",
        )?;
        work.release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            logical_bytes,
            "release collision check",
        )?;
        work.release(
            ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            requested_bytes,
            "release collision requested clone",
        )?;
    }
    Ok(())
}

#[cfg(test)]
fn stage_http_upload_closure(
    existing_objects: &Path,
    staging_objects: &Path,
    records: &[Value],
    preflight: &HttpUploadPreflight,
) -> Result<(), CliError> {
    ensure_object_dirs(staging_objects)?;
    write_object_records(
        staging_objects,
        preflight
            .incoming_closure
            .iter()
            .map(|index| records[*index].clone()),
    )?;
    for object_id in &preflight.existing_closure {
        copy_object_record(existing_objects, staging_objects, object_id)?;
    }
    Ok(())
}

#[cfg(test)]
fn rendered_object_record_size(record: &Value) -> Result<u64, CliError> {
    let mut counter = JsonByteCounter::default();
    serde_json::to_writer_pretty(&mut counter, record)?;
    counter.bytes.checked_add(1).ok_or_else(|| {
        CliError::InvalidRepository("HTTP upload rendered byte count overflow".to_string())
    })
}

fn compact_json_size<T: Serialize + ?Sized>(value: &T) -> Result<u64, CliError> {
    let mut counter = JsonByteCounter::default();
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.bytes)
}

#[derive(Serialize)]
struct SerializableObjectRecordDocument<'a> {
    #[serde(flatten)]
    record: &'a codefire_store::ObjectRecord,
    #[serde(flatten)]
    extensions: &'a serde_json::Map<String, Value>,
}

fn compact_object_document_size(
    document: &codefire_store::ObjectRecordDocument,
) -> Result<u64, CliError> {
    compact_json_size(&SerializableObjectRecordDocument {
        record: &document.record,
        extensions: &document.extensions,
    })
}

fn transfer_parsed_object_document_owned(
    ledger: &mut codefire_core::DecisionWorkLedger,
    work: &mut UploadPreflightWorkOwner,
    document: &codefire_store::ObjectRecordDocument,
    physical_bytes: u64,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError> {
    let logical_bytes = compact_object_document_size(document)?;
    work.acquire(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        logical_bytes,
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(
            codefire_core::DecisionWorkDimension::AggregateBytes,
            logical_bytes,
        ),
        operation,
    )?;
    work.release(
        ledger,
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        physical_bytes,
        operation,
    )?;
    Ok(logical_bytes)
}

#[cfg(test)]
fn transfer_parsed_object_document(
    ledger: &mut codefire_core::DecisionWorkLedger,
    document: &codefire_store::ObjectRecordDocument,
    physical_bytes: u64,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError> {
    let logical_bytes = compact_object_document_size(document)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::AggregateBytes,
                logical_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                logical_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, operation))?;
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            physical_bytes,
        )
        .map_err(|error| upload_work_error(error, operation))?;
    Ok(logical_bytes)
}

fn account_compact_json<T>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    value: &T,
    phase: codefire_core::DecisionWorkPhase,
    operation: &str,
) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    let mut counter = AccountedJsonByteCounter {
        ledger,
        phase,
        bytes: 0,
        accounting_error: None,
    };
    let serialized = serde_json::to_writer(&mut counter, value);
    if let Some(error) = counter.accounting_error.take() {
        return Err(upload_work_error(error, operation));
    }
    serialized?;
    Ok(counter.bytes)
}

struct AccountedJsonByteCounter<'a> {
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    phase: codefire_core::DecisionWorkPhase,
    bytes: u64,
    accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
}

impl Write for AccountedJsonByteCounter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("JSON byte count overflow"))?;
        if let Err(error) = self.ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                self.phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::AggregateBytes, amount),
        ) {
            self.accounting_error = Some(error);
            return Err(std::io::Error::other(
                "HTTP upload compact JSON budget exceeded",
            ));
        }
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("JSON byte count overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct JsonByteCounter {
    bytes: u64,
}

impl Write for JsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("JSON byte count overflow"))?;
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("JSON byte count overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn upload_work_error(
    error: codefire_core::DecisionWorkAccountingError,
    operation: &str,
) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed during HTTP upload {operation}: {error}"
        )),
    }
}

#[cfg(test)]
fn unique_http_upload_tmp_dir(project_root: &Path, head: &str) -> PathBuf {
    let counter = HTTP_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    project_root.join("tmp").join(format!(
        "upload-{}-{}-{}",
        std::process::id(),
        stable_hash_48(head.as_bytes()),
        counter
    ))
}

fn handle_http_request_merge(
    storage_root: &Path,
    org: &str,
    app: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("merge-request planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let source_url = required_string(request, &["source_url"])?;
    let target_url = required_string(request, &["target_url"])?;
    let source_info = parse_cf_http_url(&source_url)?;
    let source_root = http_project_root(storage_root, &source_info.org, &source_info.app);
    let target_info = parse_cf_http_url(&target_url)?;
    let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
    if target_root != project_root || target_info.org != org || target_info.app != app {
        return Ok((
            400,
            json!({"error": "target_url must belong to this project"}),
        ));
    }
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
        request,
        request.get("request_signature"),
    )?;
    ensure_remote_layout(&project_root)?;
    let project_gates = acquire_with_http_deadline(
        request,
        context.deadline,
        "merge-request project gates",
        |options| RemoteProjectGates::acquire_for_merge(&source_root, &target_root, options),
    )?;
    let (source_head, source_head_frame_bytes) = read_http_branch_head_accounted(
        &source_root,
        &source_info.branch,
        context,
        "merge-request source branch read",
    )?
    .ok_or_else(|| CliError::Usage(format!("remote branch not found: {source_url}")))?;
    let (target_head, target_head_frame_bytes) = read_http_branch_head_accounted(
        &target_root,
        &target_info.branch,
        context,
        "merge-request target branch read",
    )?
    .ok_or_else(|| CliError::Usage(format!("remote branch not found: {target_url}")))?;
    let source_objects = remote_dirs(&source_root).objects;
    let target_objects = remote_dirs(&target_root).objects;
    with_http_sealed_validation(
        context,
        &source_objects,
        &[source_head.as_str()],
        "merge-request source validation",
        |_| Ok(()),
    )?;
    let mut source_records = collect_object_records_with_owned_work_and_poll(
        &source_objects,
        &source_head,
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must return after source validation"),
        || context.deadline.check("merge-request source graph"),
    )?;
    let object_set_digest = source_object_set_digest_ordered_with_poll(
        source_records.as_slice(),
        0..source_records.len(),
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must be present for source digest"),
        || context.deadline.check("merge-request source digest"),
    );
    let mut object_set_digest = match object_set_digest {
        Ok(digest) => digest,
        Err(error) => {
            source_records.release(
                context
                    .ledger
                    .as_mut()
                    .expect("merge-request ledger must remain present after digest failure"),
            )?;
            return Err(error);
        }
    };
    if let Err(error) = with_http_sealed_validation(
        context,
        &target_objects,
        &[target_head.as_str()],
        "merge-request target validation",
        |_| Ok(()),
    ) {
        let ledger = context
            .ledger
            .as_mut()
            .expect("merge-request ledger must return after target validation failure");
        let mut first_error = Some(error);
        record_http_cleanup_error(&mut first_error, object_set_digest.release(ledger));
        record_http_cleanup_error(&mut first_error, source_records.release(ledger));
        return Err(first_error.expect("target validation error must be retained"));
    }
    let mut destination_ownership = match HttpDigestDestinationOwnership::acquire(
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must be present for digest destination reservation"),
        &object_set_digest,
        2,
        codefire_core::DecisionWorkPhase::Validation,
        "reserve merge-request object-set digest destinations",
    ) {
        Ok(ownership) => ownership,
        Err(error) => {
            let ledger = context
                .ledger
                .as_mut()
                .expect("merge-request ledger must remain present after reservation failure");
            let mut first_error = Some(error);
            record_http_cleanup_error(&mut first_error, object_set_digest.release(ledger));
            record_http_cleanup_error(&mut first_error, source_records.release(ledger));
            return Err(first_error.expect("digest destination reservation error must be retained"));
        }
    };
    let preparation = (|| -> Result<_, CliError> {
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
        let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
            mr_id: &mr_id,
            source_project_root: &source_root,
            target_project_root: &target_root,
            source_branch: &source_info.branch,
            target_branch: &target_info.branch,
            source_head: &source_head,
            target_head: &target_head,
            source_object_set_digest: object_set_digest.as_str(),
        })?;
        mr_payload["subject_digest"] = Value::String(merge_request_subject_digest(&subject)?);
        mr_payload["subject"] = subject;
        let response = request_merge_result(
            &mr_id,
            &required_string(&mr_payload, &["source_head"])?,
            &required_string(&mr_payload, &["target_head_at_request"])?,
        );
        let source_project_id = remote_project_stable_id(&source_root)?;
        let target_project_id = remote_project_stable_id(&target_root)?;
        let publication_request = request_merge_publication(
            actor,
            remote_idempotency_key(request),
            &MergePublicationIdentity {
                source_project_id: &source_project_id,
                target_project_id: &target_project_id,
                source_branch: &source_info.branch,
                target_branch: &target_info.branch,
                source_head: &source_head,
                target_head_at_request: &target_head,
                source_object_set_digest: object_set_digest.as_str(),
            },
        );
        Ok((mr_payload, mr_id, response, publication_request))
    })();
    let (mr_payload, mr_id, response, publication_request) = match preparation {
        Ok(prepared) => prepared,
        Err(error) => {
            let ledger = context
                .ledger
                .as_mut()
                .expect("merge-request ledger must remain present after preparation failure");
            let mut first_error = Some(error);
            record_http_cleanup_error(&mut first_error, object_set_digest.release(ledger));
            record_http_cleanup_error(&mut first_error, destination_ownership.release(ledger));
            record_http_cleanup_error(&mut first_error, source_records.release(ledger));
            return Err(first_error.expect("merge-request preparation error must be retained"));
        }
    };
    if let Err(error) = object_set_digest.release(
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must be present for source digest handoff"),
    ) {
        drop(publication_request);
        drop(response);
        drop(mr_payload);
        let ledger = context
            .ledger
            .as_mut()
            .expect("merge-request ledger must remain present after digest handoff failure");
        let mut first_error = Some(error);
        record_http_cleanup_error(&mut first_error, destination_ownership.release(ledger));
        record_http_cleanup_error(&mut first_error, source_records.release(ledger));
        return Err(first_error.expect("source digest handoff error must be retained"));
    }
    let gate = match project_gates.exclusive_for(&target_root) {
        Ok(gate) => gate,
        Err(error) => {
            drop(publication_request);
            drop(response);
            drop(mr_payload);
            let ledger = context
                .ledger
                .as_mut()
                .expect("merge-request ledger must remain present after gate failure");
            let mut first_error = Some(error);
            record_http_cleanup_error(&mut first_error, destination_ownership.release(ledger));
            record_http_cleanup_error(&mut first_error, source_records.release(ledger));
            return Err(first_error.expect("merge-request gate error must be retained"));
        }
    };
    if let Err(error) = source_records.publish_with_work_and_poll(
        &target_objects,
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must be present for object escrow"),
        || context.deadline.check("merge-request object escrow"),
    ) {
        drop(publication_request);
        drop(response);
        drop(mr_payload);
        destination_ownership.release(
            context
                .ledger
                .as_mut()
                .expect("merge-request ledger must remain present after object escrow failure"),
        )?;
        return Err(error);
    }
    match replay_json_publication(gate, &target_root, &publication_request) {
        Ok(Some(published)) => {
            drop(publication_request);
            drop(response);
            drop(mr_payload);
            destination_ownership.release(
                context
                    .ledger
                    .as_mut()
                    .expect("merge-request ledger must be present after replay"),
            )?;
            release_http_retained(
                context,
                source_head_frame_bytes,
                "release replayed merge-request source head",
            )?;
            release_http_retained(
                context,
                target_head_frame_bytes,
                "release replayed merge-request target head",
            )?;
            return Ok((200, published));
        }
        Ok(None) => {}
        Err(error) => {
            drop(publication_request);
            drop(response);
            drop(mr_payload);
            destination_ownership.release(
                context
                    .ledger
                    .as_mut()
                    .expect("merge-request ledger must be present after replay failure"),
            )?;
            return Err(error);
        }
    }
    let publication = execute_json_publication(
        gate,
        &target_root,
        &publication_request,
        &[AuthorityWrite {
            authority: RemoteAuthority::MergeRequest(mr_id),
            before: None,
            after: mr_payload,
        }],
        &response,
    );
    drop(publication_request);
    drop(response);
    destination_ownership.release(
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must be present after publication"),
    )?;
    let published = publication?;
    release_http_retained(
        context,
        source_head_frame_bytes,
        "release published merge-request source head",
    )?;
    release_http_retained(
        context,
        target_head_frame_bytes,
        "release published merge-request target head",
    )?;
    Ok((200, published))
}

fn handle_http_request_list(
    storage_root: &Path,
    org: &str,
    app: &str,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("merge-request list planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let dirs = remote_dirs(&project_root);
    if !dirs.merge_requests.exists() {
        return Ok((404, json!({"error": "remote project not found"})));
    }
    let _project_gate = RemoteProjectGate::acquire_shared(&project_root, &LockOptions::default())?;
    #[derive(Serialize)]
    struct MergeRequestListProjection<'a> {
        id: &'a str,
        source_url: &'a str,
        status: &'a str,
        target_url: &'a str,
    }
    #[derive(Serialize)]
    struct MergeRequestListEnvelope<'a> {
        merge_requests: &'a [Value],
    }
    let mut response_retained_bytes = retain_compact_http_json(
        context,
        &MergeRequestListEnvelope {
            merge_requests: &[],
        },
        codefire_core::DecisionWorkPhase::Encoding,
        "retain merge-request list envelope",
    )?;
    let mut authority_records = VecDeque::new();
    let mut paths = read_http_directory_entries_accounted(
        &dirs.merge_requests,
        context,
        "merge-request list scan",
    )?
    .into_iter()
    .collect::<VecDeque<_>>();
    let scan_result = (|| -> Result<(), CliError> {
        while let Some((path, path_frame_bytes)) = paths.pop_front() {
            let record_result = (|| -> Result<Option<AccountedHttpJson>, CliError> {
                context.deadline.check("merge-request list validation")?;
                if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                    return Ok(None);
                }
                let accounted_mr = read_optional_http_json_accounted(
                    &path,
                    context,
                    "merge-request authority JSON read",
                )?
                .ok_or_else(|| {
                    CliError::InvalidRepository("merge-request record disappeared".to_string())
                })?;
                let (mr, mr_retained_bytes) = accounted_mr.into_parts();
                let validation = (|| -> Result<(), CliError> {
                    validate_record_version(&mr, "remote_merge_request")?;
                    let source_info = parse_cf_http_url(&required_string(&mr, &["source_url"])?)?;
                    let source_root =
                        http_project_root(storage_root, &source_info.org, &source_info.app);
                    let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
                    let target_root =
                        http_project_root(storage_root, &target_info.org, &target_info.app);
                    if target_root != project_root {
                        return Err(CliError::InvalidRepository(
                            "merge request target does not belong to the listed project"
                                .to_string(),
                        ));
                    }
                    reject_legacy_cross_project_merge_request_read(&mr, &source_root, &target_root)
                })();
                if let Err(error) = validation {
                    release_http_retained(
                        context,
                        mr_retained_bytes,
                        "release rejected merge-request authority JSON",
                    )?;
                    return Err(error);
                }
                Ok(Some(AccountedHttpJson {
                    value: mr,
                    retained_bytes: mr_retained_bytes,
                }))
            })();
            release_http_retained(
                context,
                path_frame_bytes,
                "release merge-request directory membership",
            )?;
            if let Some(accounted_mr) = record_result? {
                authority_records.push_back(accounted_mr);
            }
        }
        Ok(())
    })();
    if let Err(error) = scan_result {
        for (_, path_frame_bytes) in paths {
            release_http_retained(
                context,
                path_frame_bytes,
                "release unscanned merge-request directory membership",
            )?;
        }
        for accounted in authority_records {
            release_http_retained(
                context,
                accounted.retained_bytes,
                "release accepted merge-request authority JSON after scan failure",
            )?;
        }
        release_http_retained(
            context,
            response_retained_bytes,
            "release failed merge-request list response envelope",
        )?;
        return Err(error);
    }
    let mut subject_digests = BTreeMap::<String, (String, u64)>::new();
    let sealed_validation = validate_http_merge_request_records_in_one_session(
        storage_root,
        &project_root,
        context,
        &dirs.objects,
        authority_records.make_contiguous(),
        &mut subject_digests,
    );
    if let Err(error) = sealed_validation {
        let error = normalize_http_store_cli_error(error);
        for accounted in authority_records {
            release_http_retained(
                context,
                accounted.retained_bytes,
                "release rejected merge-request authority JSON",
            )?;
        }
        release_http_retained(
            context,
            response_retained_bytes,
            "release rejected merge-request list response envelope",
        )?;
        for (_, (_, frame_bytes)) in subject_digests {
            release_http_retained(
                context,
                frame_bytes,
                "release rejected merge-request subject digest cache",
            )?;
        }
        return Err(error);
    }

    let mut requests = Vec::new();
    let projection_result = (|| -> Result<(), CliError> {
        while let Some(accounted_mr) = authority_records.pop_front() {
            let (mr, mr_retained_bytes) = accounted_mr.into_parts();
            let record_result = (|| -> Result<(), CliError> {
                context
                    .deadline
                    .check("merge-request list response projection")?;
                let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
                let target_root =
                    http_project_root(storage_root, &target_info.org, &target_info.app);
                let current_target = read_optional_http_json_accounted(
                    &remote_branch_path(&target_root, &target_info.branch),
                    context,
                    "merge-request list target branch read",
                )?;
                let stale = if let Some(accounted_target) = current_target {
                    let (target, target_retained_bytes) = accounted_target.into_parts();
                    let target_result = (|| -> Result<bool, CliError> {
                        validate_remote_branch_record(&target)?;
                        Ok(target.get("head").and_then(Value::as_str)
                            != mr.get("target_head_at_request").and_then(Value::as_str))
                    })();
                    release_http_retained(
                        context,
                        target_retained_bytes,
                        "release merge-request list target branch",
                    )?;
                    target_result?
                } else {
                    false
                };
                let status = if stale {
                    "stale"
                } else {
                    mr.get("status").and_then(Value::as_str).unwrap_or("open")
                };
                let id = mr.get("id").and_then(Value::as_str).ok_or_else(|| {
                    CliError::InvalidRepository("merge-request ID is missing".to_string())
                })?;
                let source_url = mr
                    .get("source_url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        CliError::InvalidRepository("source_url is missing".to_string())
                    })?;
                let target_url = mr
                    .get("target_url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        CliError::InvalidRepository("target_url is missing".to_string())
                    })?;
                if !requests.is_empty() {
                    acquire_http_retained(
                        context,
                        1,
                        codefire_core::DecisionWorkPhase::Encoding,
                        "retain merge-request list separator",
                    )?;
                    response_retained_bytes =
                        response_retained_bytes.checked_add(1).ok_or_else(|| {
                            CliError::InvalidRepository(
                                "merge-request list response ownership overflow".to_string(),
                            )
                        })?;
                }
                let projection_bytes = retain_compact_http_json(
                    context,
                    &MergeRequestListProjection {
                        id,
                        source_url,
                        status,
                        target_url,
                    },
                    codefire_core::DecisionWorkPhase::Encoding,
                    "retain merge-request list projection",
                )?;
                response_retained_bytes = response_retained_bytes
                    .checked_add(projection_bytes)
                    .ok_or_else(|| {
                        CliError::InvalidRepository(
                            "merge-request list response ownership overflow".to_string(),
                        )
                    })?;
                requests.push(json!({
                    "id": id,
                    "status": status,
                    "source_url": source_url,
                    "target_url": target_url,
                }));
                Ok(())
            })();
            release_http_retained(
                context,
                mr_retained_bytes,
                "release projected merge-request authority JSON",
            )?;
            record_result?;
        }
        Ok(())
    })();
    for accounted in authority_records {
        release_http_retained(
            context,
            accounted.retained_bytes,
            "release unprojected merge-request authority JSON",
        )?;
    }
    for (_, (_, frame_bytes)) in subject_digests {
        release_http_retained(
            context,
            frame_bytes,
            "release merge-request subject digest cache",
        )?;
    }
    if let Err(error) = projection_result {
        release_http_retained(
            context,
            response_retained_bytes,
            "release failed merge-request list response",
        )?;
        return Err(error);
    }
    Ok((200, json!({"merge_requests": requests})))
}

fn handle_http_request_review(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("review planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let mr_path = crate::remote::remote_merge_request_path(&project_root, mr_id);
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
        request,
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
    let Some(accounted_mr) =
        read_optional_http_json_accounted(&mr_path, context, "review merge-request read")?
    else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    let (mr, initial_mr_retained_bytes) = accounted_mr.into_parts();
    let initial = (|| -> Result<_, CliError> {
        validate_record_version(&mr, "remote_merge_request")?;
        let initial_source_url = required_string(&mr, &["source_url"])?;
        let source_info = parse_cf_http_url(&initial_source_url)?;
        let source_root = http_project_root(storage_root, &source_info.org, &source_info.app);
        let initial_target_url = required_string(&mr, &["target_url"])?;
        let target_info = parse_cf_http_url(&initial_target_url)?;
        let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
        Ok((
            source_info,
            source_root,
            target_info,
            target_root,
            initial_source_url,
            initial_target_url,
            mr.get("subject").is_some(),
        ))
    })();
    drop(mr);
    release_http_retained(
        context,
        initial_mr_retained_bytes,
        "release initial review merge-request read",
    )?;
    let (
        source_info,
        source_root,
        target_info,
        target_root,
        initial_source_url,
        initial_target_url,
        initially_materialized,
    ) = initial?;
    if target_root != project_root {
        return Ok((
            400,
            json!({"error": "merge request target does not belong to this project"}),
        ));
    }
    let single_gate = if initially_materialized {
        Some(acquire_with_http_deadline(
            request,
            context.deadline,
            "review project gate",
            |options| RemoteProjectGate::acquire_exclusive(&target_root, options),
        )?)
    } else {
        None
    };
    let project_gates = if single_gate.is_none() {
        Some(acquire_with_http_deadline(
            request,
            context.deadline,
            "review project gates",
            |options| RemoteProjectGates::acquire_for_merge(&source_root, &target_root, options),
        )?)
    } else {
        None
    };
    let gate = if let Some(gate) = single_gate.as_ref() {
        gate
    } else {
        project_gates
            .as_ref()
            .ok_or_else(|| {
                CliError::InvalidRepository("remote review project gates are missing".to_string())
            })?
            .exclusive_for(&target_root)?
    };
    let Some(accounted_mr) =
        read_optional_http_json_accounted(&mr_path, context, "review locked merge-request read")?
    else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    let (mut mr, locked_mr_retained_bytes) = accounted_mr.into_parts();
    let mut review_clone_bytes = 0_u64;
    let mut review_record_growth_bytes = 0_u64;
    let mut review_status_growth_bytes = 0_u64;
    let mut review_materialization_retained_bytes = 0_u64;
    let result = (|| -> Result<(u16, Value), CliError> {
        validate_record_version(&mr, "remote_merge_request")?;
        if mr.get("source_url").and_then(Value::as_str) != Some(initial_source_url.as_str())
            || mr.get("target_url").and_then(Value::as_str) != Some(initial_target_url.as_str())
        {
            return Err(CliError::InvalidRepository(
                "merge request endpoints changed while acquiring project gates".to_string(),
            ));
        }
        validate_http_merge_request_record_with_context(storage_root, &mr, context)?;
        let subject_is_materialized = mr.get("subject").is_some();
        if !subject_is_materialized && http_merge_request_is_stale(storage_root, &mr, context)? {
            return Ok((
                409,
                json!({"error": format!("merge request is stale: {mr_id}")}),
            ));
        }
        review_clone_bytes = reserve_compact_http_temporary(
            context,
            &mr,
            codefire_core::DecisionWorkPhase::Validation,
            "reserve review authority clone",
        )?;
        let authority_before = mr.clone();
        let subject_digest = if subject_is_materialized {
            required_string(&mr, &["subject_digest"])?
        } else {
            let accounted = materialize_merge_request_subject_for_roots_with_work_and_poll(
                &mut mr,
                MergeRequestSubjectRoots::new(
                    &source_root,
                    &target_root,
                    &source_info.branch,
                    &target_info.branch,
                ),
                context
                    .ledger
                    .as_mut()
                    .expect("review ledger must be present for subject materialization"),
                || context.deadline.check("review subject materialization"),
                reserve_http_merge_request_materialization,
            )?;
            let (subject_digest, retained_bytes) = accounted.into_parts();
            review_materialization_retained_bytes = retained_bytes;
            subject_digest
        };
        let response = review_result(mr_id, &reviewer, &decision);
        let stable_project_id = remote_project_stable_id(&project_root)?;
        let source_head = required_string(&mr, &["source_head"])?;
        let publication_request = review_publication(
            actor,
            remote_idempotency_key(request),
            &ReviewPublicationIdentity {
                stable_project_id: &stable_project_id,
                merge_request_id: mr_id,
                reviewer: &reviewer,
                decision: &decision,
                comment,
                subject_digest: &subject_digest,
                source_head: &source_head,
            },
        );
        if let Some(published) = replay_json_publication(gate, &project_root, &publication_request)?
        {
            return Ok((200, published));
        }
        if subject_is_materialized && http_merge_request_is_stale(storage_root, &mr, context)? {
            return Ok((
                409,
                json!({"error": format!("merge request is stale: {mr_id}")}),
            ));
        }
        #[derive(Serialize)]
        struct PlannedReview<'a> {
            reviewer: &'a str,
            decision: &'a str,
            comment: &'a str,
            subject_digest: &'a str,
            reviewed_at: &'a str,
        }
        let reviewed_at = now_iso_utc();
        let planned_review = PlannedReview {
            reviewer: &reviewer,
            decision: &decision,
            comment,
            subject_digest: &subject_digest,
            reviewed_at: &reviewed_at,
        };
        let review_bytes = account_compact_json(
            context
                .ledger
                .as_mut()
                .expect("review ledger must be present while planning the review record"),
            &planned_review,
            codefire_core::DecisionWorkPhase::Validation,
            "measure review record growth",
        )?;
        let reviews_growth = match mr.get("reviews") {
            Some(Value::Array(reviews)) => review_bytes
                .checked_add(u64::from(!reviews.is_empty()))
                .ok_or_else(|| {
                    CliError::InvalidRepository("review array growth overflow".to_string())
                })?,
            Some(_) => {
                return Err(CliError::InvalidRepository(
                    "merge request reviews must be an array".to_string(),
                ));
            }
            None => {
                let planned_array_bytes = review_bytes.checked_add(2).ok_or_else(|| {
                    CliError::InvalidRepository("review array growth overflow".to_string())
                })?;
                let key_bytes = account_compact_json(
                    context
                        .ledger
                        .as_mut()
                        .expect("review ledger must be present while planning reviews field"),
                    "reviews",
                    codefire_core::DecisionWorkPhase::Validation,
                    "measure reviews field growth",
                )?;
                key_bytes
                    .checked_add(2)
                    .and_then(|bytes| bytes.checked_add(planned_array_bytes))
                    .ok_or_else(|| {
                        CliError::InvalidRepository("reviews field growth overflow".to_string())
                    })?
            }
        };
        acquire_http_retained(
            context,
            reviews_growth,
            codefire_core::DecisionWorkPhase::Validation,
            "reserve review record growth",
        )?;
        review_record_growth_bytes = reviews_growth;
        let new_status = if decision == "approve" {
            "approved"
        } else {
            "rejected"
        };
        let status_growth = reserve_http_object_field_growth(
            context,
            &mr,
            "status",
            new_status,
            "reserve review status growth",
        )?;
        review_status_growth_bytes = status_growth;
        if mr.get("reviews").is_none() {
            mr["reviews"] = Value::Array(Vec::new());
        }
        mr.get_mut("reviews")
            .and_then(Value::as_array_mut)
            .expect("review array was validated before mutation")
            .push(json!({
                "reviewer": reviewer,
                "decision": decision,
                "comment": comment,
                "subject_digest": subject_digest,
                "reviewed_at": reviewed_at,
            }));
        mr["status"] = Value::String(new_status.to_string());
        context.deadline.check("review publication preflight")?;
        let published = execute_json_publication(
            gate,
            &project_root,
            &publication_request,
            &[review_write(mr_id, authority_before, mr)],
            &response,
        )?;
        Ok((200, published))
    })();
    if review_clone_bytes != 0 {
        release_http_temporary(
            context,
            review_clone_bytes,
            "release review authority clone",
        )?;
    }
    if review_status_growth_bytes != 0 {
        release_http_retained(
            context,
            review_status_growth_bytes,
            "release review status growth",
        )?;
    }
    if review_record_growth_bytes != 0 {
        release_http_retained(
            context,
            review_record_growth_bytes,
            "release review record growth",
        )?;
    }
    if review_materialization_retained_bytes != 0 {
        release_http_retained(
            context,
            review_materialization_retained_bytes,
            "release review subject materialization growth",
        )?;
    }
    release_http_retained(
        context,
        locked_mr_retained_bytes,
        "release review authority JSON",
    )?;
    result
}

struct ApplyMutationIdentity {
    actor: String,
    target_root: PathBuf,
    objects: PathBuf,
    target_branch: String,
    source_head: String,
    target_head_at_request: String,
    subject_digest: String,
    publication_request: PublicationRequest,
    response: Value,
}

const APPLY_FRESH_REJECTION_SENTINEL: &str = "apply fresh builder rejected the publication";

#[derive(Default)]
struct ApplyAuthorityAccounting {
    mr_retained: u64,
    branch_retained: u64,
    generation_retained: u64,
    clone_temporary: u64,
    growth: [u64; 4],
    branch_postimage_retained: u64,
    generation_postimage_retained: u64,
}

impl ApplyAuthorityAccounting {
    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            &mut self.clone_temporary,
            "release apply merge-request clone",
        )?;
        for amount in self.growth.iter_mut().rev() {
            release_apply_owned_amount(
                ledger,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                amount,
                "release apply merge-request authority growth",
            )?;
        }
        release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            &mut self.generation_postimage_retained,
            "release apply generation postimage",
        )?;
        release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            &mut self.branch_postimage_retained,
            "release apply branch postimage",
        )?;
        release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            &mut self.generation_retained,
            "release apply generation authority JSON",
        )?;
        release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            &mut self.branch_retained,
            "release apply target branch authority JSON",
        )?;
        release_apply_owned_amount(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            &mut self.mr_retained,
            "release apply merge-request authority JSON",
        )
    }
}

fn release_apply_owned_amount(
    ledger: &mut codefire_core::DecisionWorkLedger,
    dimension: codefire_core::DecisionWorkDimension,
    amount: &mut u64,
    operation: &str,
) -> Result<(), CliError> {
    if *amount == 0 {
        return Ok(());
    }
    ledger
        .release(dimension, *amount)
        .map_err(|error| upload_work_error(error, operation))?;
    *amount = 0;
    Ok(())
}

struct ApplyAuthorityValues {
    generation_before: Option<Value>,
    generation_after: Value,
    branch_before: Value,
    branch_after: Value,
    merge_request_before: Value,
    merge_request_after: Value,
}

impl ApplyAuthorityValues {
    fn into_writes(self, target_branch: &str, mr_id: &str) -> Vec<AuthorityWrite> {
        apply_writes(ApplyPostimages {
            target_branch,
            merge_request_id: mr_id,
            generation_before: self.generation_before,
            generation_after: self.generation_after,
            branch_before: self.branch_before,
            branch_after: self.branch_after,
            merge_request_before: self.merge_request_before,
            merge_request_after: self.merge_request_after,
        })
    }
}

enum ApplyAuthorityPreparation {
    Ready(Box<ApplyAuthorityValues>, ApplyAuthorityAccounting),
    Rejected(u16, Value),
}

fn cancel_http_mutation_owner(
    owner: &mut PreparedHttpMutationTransaction<'_>,
) -> codefire_core::DecisionWorkLedger {
    if let Err(error) = owner.cancel_all() {
        if let Err(retry) = owner.cancel_all() {
            fail_closed_http_mutation_drop(format!(
                "HTTP mutation explicit cancellation failed ({error}) and retry failed: {retry}"
            ));
        }
    }
    match owner.take_ledger() {
        Ok(ledger) => ledger,
        Err(error) => fail_closed_http_mutation_drop(format!(
            "HTTP mutation explicit cancellation could not recover its ledger: {error}"
        )),
    }
}

fn cancel_http_upload_mutation_owner(
    owner: &mut PreparedHttpMutationTransaction<'_>,
    preparation: &Option<RemoteProjectExclusivePreparation>,
) -> codefire_core::DecisionWorkLedger {
    if let Err(error) = owner.cancel_all() {
        if let Err(retry) = owner.cancel_all() {
            fail_closed_http_mutation_drop(format!(
                "HTTP upload mutation cancellation failed ({error}) and retry failed: {retry}"
            ));
        }
    }
    let recovered = if matches!(
        preparation,
        Some(RemoteProjectExclusivePreparation::Unpublished(_))
    ) {
        owner.take_ledger_preserving_external_permits()
    } else {
        owner.take_ledger()
    };
    recovered.unwrap_or_else(|error| {
        fail_closed_http_mutation_drop(format!(
            "HTTP upload mutation cancellation could not recover its ledger: {error}"
        ))
    })
}

fn cancel_http_mutation_terminal(
    terminal: &mut PreparedHttpMutationTerminal,
) -> codefire_core::DecisionWorkLedger {
    match terminal.cancel_and_take_ledger() {
        Ok(ledger) => ledger,
        Err(error) => fail_closed_http_mutation_drop(format!(
            "HTTP mutation visibility terminal cancellation could not recover its ledger: {error}"
        )),
    }
}

fn release_http_pending_nonce(
    pending_nonce: &mut Option<crate::signatures::PendingRequestNonce>,
    owner: &mut PreparedHttpMutationTransaction<'_>,
    operation: &str,
) {
    let Some(nonce) = pending_nonce.take() else {
        return;
    };
    if let Err(error) = nonce.release_retained(
        owner
            .ledger
            .as_mut()
            .expect("apply mutation owner retains its ledger"),
    ) {
        fail_closed_http_mutation_drop(format!(
            "{operation} pending nonce cleanup lost exact ownership: {error}"
        ));
    }
}

fn abort_http_validation_session(
    session: &mut codefire_store::SealedCommitValidationSession,
    reason: codefire_core::DecisionWorkTerminalReason,
) -> codefire_core::DecisionWorkLedger {
    match session.abort_and_take_decision_work_ledger(reason) {
        Ok(Some(ledger)) => ledger,
        Ok(None) => fail_closed_http_mutation_drop(
            "apply validation session returned no decision-work ledger".to_string(),
        ),
        Err(error) => fail_closed_http_mutation_drop(format!(
            "apply validation session could not return its ledger: {error}"
        )),
    }
}

fn discard_http_validation_session(
    session: &mut codefire_store::SealedCommitValidationSession,
) -> codefire_core::DecisionWorkLedger {
    match session.discard_and_take_decision_work_ledger() {
        Ok(Some(ledger)) => ledger,
        Ok(None) => fail_closed_http_mutation_drop(
            "discarded apply validation session returned no decision-work ledger".to_string(),
        ),
        Err(error) => fail_closed_http_mutation_drop(format!(
            "discarded apply validation session could not return its ledger: {error}"
        )),
    }
}

fn recover_http_validation_build_failure(
    failure: PreparedHttpMutationValidationBuildFailure,
    preserve_external_retained: bool,
) -> (CliError, codefire_core::DecisionWorkLedger) {
    let recovered = if preserve_external_retained {
        failure.cancel_and_take_unfinished_ledger()
    } else {
        failure.cancel_and_take_ledger(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
    };
    match recovered {
        Ok(recovered) => recovered,
        Err((cause, cancellation)) => {
            let retried = if preserve_external_retained {
                cancellation.retry_unfinished()
            } else {
                cancellation.retry(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
            };
            match retried {
                Ok(ledger) => (cause, ledger),
                Err(error) => fail_closed_http_mutation_drop(format!(
                    "apply bound validation cleanup remained unrecoverable: {error:?}"
                )),
            }
        }
    }
}

fn validate_apply_semantics_with_provisional_subject(
    project_root: &Path,
    source_head: &str,
    target_head_at_request: &str,
    subject: &codefire_store::ProvisionalSealedValidationSubject<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<bool, CliError> {
    let accounted_policy = read_optional_http_json_accounted_with_ledger(
        &project_root.join("server_policy.json"),
        ledger,
        deadline,
        "apply commit-signature policy read",
    )?;
    let (policy, mut policy_retained) = accounted_policy
        .map(AccountedHttpJson::into_parts)
        .map_or((None, 0), |(policy, retained)| (Some(policy), retained));
    let validation = match is_ancestor_in_provisional_subject_with_poll(
        subject,
        ledger,
        target_head_at_request,
        source_head,
        || deadline.check("apply ancestry validation"),
    ) {
        Ok(false) => Ok(false),
        Ok(true) => super::signatures::enforce_remote_commit_signature_policy_in_provisional_subject_with_poll(
                policy.as_ref(),
                subject,
                ledger,
                source_head,
                || deadline.check("apply commit signature validation"),
            )
            .map(|()| true),
        Err(error) => Err(error),
    };
    drop(policy);
    let release = release_apply_owned_amount(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        &mut policy_retained,
        "release apply commit-signature policy",
    );
    match (validation, release) {
        (Ok(valid), Ok(())) => Ok(valid),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_apply_authority_values(
    storage_root: &Path,
    project_root: &Path,
    mr_path: &Path,
    mr_id: &str,
    identity: &ApplyMutationIdentity,
    ledger: &mut codefire_core::DecisionWorkLedger,
    deadline: &HttpRequestDeadline,
) -> Result<ApplyAuthorityPreparation, CliError> {
    let mut accounting = ApplyAuthorityAccounting::default();
    let prepared = (|| -> Result<ApplyAuthorityPreparation, CliError> {
        let accounted_mr = read_optional_http_json_accounted_with_ledger(
            mr_path,
            ledger,
            deadline,
            "apply locked merge-request read",
        )?
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "merge request disappeared while planning apply".to_string(),
            )
        })?;
        let (mut mr, retained) = accounted_mr.into_parts();
        accounting.mr_retained = retained;
        validate_http_merge_request_record_shape(storage_root, &mr)?;
        let source_info = parse_cf_http_url(&required_string(&mr, &["source_url"])?)?;
        let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
        let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
        if target_root != project_root
            || target_info.branch != identity.target_branch
            || required_string(&mr, &["source_head"])? != identity.source_head
            || required_string(&mr, &["target_head_at_request"])? != identity.target_head_at_request
        {
            return Err(CliError::InvalidRepository(
                "merge request identity changed while planning apply".to_string(),
            ));
        }
        let subject_digest = validate_merge_request_subject_for_roots_with_work_and_poll(
            &mr,
            &target_root,
            &source_info.branch,
            &target_info.branch,
            ledger,
            || deadline.check("apply locked subject validation"),
        )?;
        if subject_digest != identity.subject_digest {
            return Err(CliError::InvalidRepository(
                "merge request subject changed while planning apply".to_string(),
            ));
        }
        if mr.get("status").and_then(Value::as_str) != Some("approved") {
            return Ok(ApplyAuthorityPreparation::Rejected(
                409,
                json!({"error": format!("merge request must be approved before apply: {mr_id}")}),
            ));
        }
        let subject_approved =
            merge_request_has_effective_approval_with_poll(&mr, &subject_digest, || {
                deadline.check("apply locked review validation")
            })?;
        if !subject_approved {
            return Ok(ApplyAuthorityPreparation::Rejected(
                409,
                json!({"error": format!("merge request approval is not bound to its current subject: {mr_id}")}),
            ));
        }

        let accounted_branch = read_optional_http_json_accounted_with_ledger(
            &remote_branch_path(project_root, &identity.target_branch),
            ledger,
            deadline,
            "apply locked target branch read",
        )?
        .ok_or_else(|| CliError::Usage("remote target branch not found".to_string()))?;
        let (target_branch, retained) = accounted_branch.into_parts();
        accounting.branch_retained = retained;
        validate_remote_branch_record(&target_branch)?;
        if target_branch.get("head").and_then(Value::as_str)
            != Some(identity.target_head_at_request.as_str())
        {
            return Ok(ApplyAuthorityPreparation::Rejected(
                409,
                json!({"error": format!("merge request is stale: {mr_id}")}),
            ));
        }

        let generation_before = match read_optional_http_json_accounted_with_ledger(
            &project_root.join("gc_state.json"),
            ledger,
            deadline,
            "apply generation authority read",
        )? {
            Some(accounted) => {
                let (generation, retained) = accounted.into_parts();
                accounting.generation_retained = retained;
                Some(generation)
            }
            None => None,
        };
        let generation = generation_before
            .as_ref()
            .and_then(|value| value.get("current_generation"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| CliError::InvalidRepository("remote generation overflow".to_string()))?;

        #[derive(Serialize)]
        struct PlannedApplyBranch<'a> {
            version: u64,
            name: &'a str,
            head: &'a str,
            generation: u64,
            applied_from: &'a str,
            applied_by: &'a str,
            transport: &'a str,
            updated_at: &'a str,
        }
        #[derive(Serialize)]
        struct PlannedGeneration {
            current_generation: u64,
        }

        let applied_from = required_string(&mr, &["source_url"])?;
        let applied_by = identity.actor.as_str();
        let updated_at = now_iso_utc();
        let planned_branch = PlannedApplyBranch {
            version: 1,
            name: &identity.target_branch,
            head: &identity.source_head,
            generation,
            applied_from: &applied_from,
            applied_by,
            transport: "http",
            updated_at: &updated_at,
        };
        accounting.branch_postimage_retained = retain_compact_http_json_with_ledger(
            ledger,
            deadline,
            &planned_branch,
            codefire_core::DecisionWorkPhase::Validation,
            "retain apply branch postimage",
        )?;
        let planned_generation = PlannedGeneration {
            current_generation: generation,
        };
        accounting.generation_postimage_retained = retain_compact_http_json_with_ledger(
            ledger,
            deadline,
            &planned_generation,
            codefire_core::DecisionWorkPhase::Validation,
            "retain apply generation postimage",
        )?;
        let branch_after = json!({
            "version": 1,
            "name": identity.target_branch,
            "head": identity.source_head,
            "generation": generation,
            "applied_from": applied_from,
            "applied_by": applied_by,
            "transport": "http",
            "updated_at": updated_at,
        });
        let generation_after = json!({"current_generation": generation});
        accounting.clone_temporary = reserve_compact_http_temporary_with_ledger(
            ledger,
            deadline,
            &mr,
            codefire_core::DecisionWorkPhase::Validation,
            "reserve apply merge-request clone",
        )?;
        let before_mr = mr.clone();
        let applied_at = now_iso_utc();
        accounting.growth[0] = reserve_http_object_field_growth_with_ledger(
            ledger,
            &mr,
            "status",
            "applied",
            "reserve applied status growth",
        )?;
        for (index, (field, value, operation)) in [
            (
                "applied_at",
                applied_at.as_str(),
                "reserve applied timestamp growth",
            ),
            ("applied_by", applied_by, "reserve applied actor growth"),
            (
                "applied_head",
                identity.source_head.as_str(),
                "reserve applied head growth",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            accounting.growth[index + 1] =
                reserve_http_object_field_growth_with_ledger(ledger, &mr, field, value, operation)?;
        }
        mr["status"] = Value::String("applied".to_string());
        mr["applied_at"] = Value::String(applied_at);
        mr["applied_by"] = Value::String(applied_by.to_string());
        mr["applied_head"] = Value::String(identity.source_head.clone());
        deadline.check("apply publication preflight")?;

        Ok(ApplyAuthorityPreparation::Ready(
            Box::new(ApplyAuthorityValues {
                generation_before,
                generation_after,
                branch_before: target_branch,
                branch_after,
                merge_request_before: before_mr,
                merge_request_after: mr,
            }),
            std::mem::take(&mut accounting),
        ))
    })();

    if !matches!(&prepared, Ok(ApplyAuthorityPreparation::Ready(_, _))) {
        // Every route-owned Value has left scope before its exact scalar is
        // released. Store-bound cancellation therefore always sees current=0.
        accounting.release(ledger)?;
    }
    prepared
}

#[cfg(any())]
fn handle_http_request_apply_legacy(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("apply planning")?;
    let project_root = http_project_root(storage_root, org, app);
    let mr_path = crate::remote::remote_merge_request_path(&project_root, mr_id);
    let actor = request
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let request_target =
        super::signatures::remote_request_target(org, app, "apply", None, Some(mr_id));
    let project_gate =
        acquire_with_http_deadline(request, context.deadline, "apply project gate", |options| {
            RemoteProjectGate::acquire_exclusive(&project_root, options)
        })?;
    let Some(accounted_mr) =
        read_optional_http_json_accounted(&mr_path, context, "apply merge-request read")?
    else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    let (mut mr, apply_mr_retained_bytes) = accounted_mr.into_parts();
    let mut apply_clone_bytes = 0_u64;
    let mut apply_target_branch_retained_bytes = 0_u64;
    let mut apply_generation_retained_bytes = 0_u64;
    let mut apply_growth_bytes = [0_u64; 4];
    let mut apply_branch_postimage_retained_bytes = 0_u64;
    let mut apply_generation_postimage_retained_bytes = 0_u64;
    let result = (|| -> Result<(u16, Value), CliError> {
        validate_http_merge_request_record_with_context(storage_root, &mr, context)?;
        let source_info = parse_cf_http_url(&required_string(&mr, &["source_url"])?)?;
        let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
        let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
        if target_root != project_root || target_info.org != org || target_info.app != app {
            return Ok((
                400,
                json!({"error": "target_url must belong to this project"}),
            ));
        }
        let subject_digest = validate_merge_request_subject_for_roots_with_work_and_poll(
            &mr,
            &target_root,
            &source_info.branch,
            &target_info.branch,
            context
                .ledger
                .as_mut()
                .expect("apply ledger must be present for subject validation"),
            || context.deadline.check("apply subject validation"),
        )?;
        let source_head = required_string(&mr, &["source_head"])?;
        let target_head_at_request = required_string(&mr, &["target_head_at_request"])?;
        let response = apply_result(mr_id, &target_info.branch, &source_head);
        let stable_project_id = remote_project_stable_id(&target_root)?;
        let publication_request = apply_publication(
            actor,
            remote_idempotency_key(request),
            &ApplyPublicationIdentity {
                stable_project_id: &stable_project_id,
                merge_request_id: mr_id,
                subject_digest: &subject_digest,
                target_branch: &target_info.branch,
                source_head: &source_head,
                target_head_at_request: &target_head_at_request,
            },
        );
        if let Some(published) =
            replay_json_publication(&project_gate, &target_root, &publication_request)?
        {
            return Ok((200, published));
        }
        if mr.get("status").and_then(Value::as_str) != Some("approved") {
            return Ok((
                409,
                json!({"error": format!("merge request must be approved before apply: {mr_id}")}),
            ));
        }
        if http_merge_request_is_stale(storage_root, &mr, context)? {
            return Ok((
                409,
                json!({"error": format!("merge request is stale: {mr_id}")}),
            ));
        }
        let accounted_target_branch = read_optional_http_json_accounted(
            &remote_branch_path(&target_root, &target_info.branch),
            context,
            "apply target branch read",
        )?
        .ok_or_else(|| CliError::Usage("remote target branch not found".to_string()))?;
        let (target_branch, target_branch_retained_bytes) = accounted_target_branch.into_parts();
        apply_target_branch_retained_bytes = target_branch_retained_bytes;
        validate_remote_branch_record(&target_branch)?;
        let subject_approved =
            merge_request_has_effective_approval_with_poll(&mr, &subject_digest, || {
                context.deadline.check("apply review validation")
            })?;
        if !subject_approved {
            return Ok((
                409,
                json!({"error": format!("merge request approval is not bound to its current subject: {mr_id}")}),
            ));
        }
        let target_head = required_string(&target_branch, &["head"])?;
        let escrow_objects = remote_dirs(&target_root).objects;
        let deadline = context.deadline;
        let source_is_descendant = with_http_sealed_validation(
            context,
            &escrow_objects,
            &[target_head.as_str(), source_head.as_str()],
            "apply escrow validation",
            |session| {
                let is_descendant = is_ancestor_in_validated_session_with_poll(
                    session,
                    &target_head,
                    &source_head,
                    || deadline.check("apply ancestry validation"),
                )?;
                if !is_descendant {
                    return Ok(false);
                }
                super::signatures::enforce_remote_commit_signature_policy_in_validated_session_with_poll(
                &target_root,
                session,
                &source_head,
                || deadline.check("apply signature validation"),
            )?;
                Ok(is_descendant)
            },
        )?;
        if !source_is_descendant {
            return Ok((
                409,
                json!({"error": "request apply rejected: source is not a fast-forward of target"}),
            ));
        }
        let generation_before = match read_optional_http_json_accounted(
            &target_root.join("gc_state.json"),
            context,
            "apply generation authority read",
        )? {
            Some(accounted_generation) => {
                let (generation, retained_bytes) = accounted_generation.into_parts();
                apply_generation_retained_bytes = retained_bytes;
                Some(generation)
            }
            None => None,
        };
        let generation = generation_before
            .as_ref()
            .and_then(|value| value.get("current_generation"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| CliError::InvalidRepository("remote generation overflow".to_string()))?;
        #[derive(Serialize)]
        struct PlannedApplyBranch<'a> {
            version: u64,
            name: &'a str,
            head: &'a str,
            generation: u64,
            applied_from: &'a str,
            applied_by: &'a str,
            transport: &'a str,
            updated_at: &'a str,
        }
        #[derive(Serialize)]
        struct PlannedGeneration {
            current_generation: u64,
        }
        let applied_from = required_string(&mr, &["source_url"])?;
        let applied_by = request
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("local");
        let updated_at = now_iso_utc();
        let planned_branch = PlannedApplyBranch {
            version: 1,
            name: &target_info.branch,
            head: &source_head,
            generation,
            applied_from: &applied_from,
            applied_by,
            transport: "http",
            updated_at: &updated_at,
        };
        let branch_after_bytes = retain_compact_http_json(
            context,
            &planned_branch,
            codefire_core::DecisionWorkPhase::Validation,
            "retain apply branch postimage",
        )?;
        apply_branch_postimage_retained_bytes = branch_after_bytes;
        let planned_generation = PlannedGeneration {
            current_generation: generation,
        };
        let generation_after_bytes = retain_compact_http_json(
            context,
            &planned_generation,
            codefire_core::DecisionWorkPhase::Validation,
            "retain apply generation postimage",
        )?;
        apply_generation_postimage_retained_bytes = generation_after_bytes;
        let branch_after = json!({
            "version": 1,
            "name": target_info.branch,
            "head": source_head,
            "generation": generation,
            "applied_from": applied_from,
            "applied_by": applied_by,
            "transport": "http",
            "updated_at": updated_at,
        });
        let generation_after = json!({"current_generation": generation});
        apply_clone_bytes = reserve_compact_http_temporary(
            context,
            &mr,
            codefire_core::DecisionWorkPhase::Validation,
            "reserve apply merge-request clone",
        )?;
        let before_mr = mr.clone();
        let applied_at = now_iso_utc();
        let status_growth = reserve_http_object_field_growth(
            context,
            &mr,
            "status",
            "applied",
            "reserve applied status growth",
        )?;
        apply_growth_bytes[0] = status_growth;
        for (index, (field, value, operation)) in [
            (
                "applied_at",
                applied_at.as_str(),
                "reserve applied timestamp growth",
            ),
            ("applied_by", applied_by, "reserve applied actor growth"),
            (
                "applied_head",
                source_head.as_str(),
                "reserve applied head growth",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let growth = reserve_http_object_field_growth(context, &mr, field, value, operation)?;
            apply_growth_bytes[index + 1] = growth;
        }
        mr["status"] = Value::String("applied".to_string());
        mr["applied_at"] = Value::String(applied_at);
        mr["applied_by"] = Value::String(applied_by.to_string());
        mr["applied_head"] = Value::String(source_head.clone());
        context.deadline.check("apply publication preflight")?;
        let published = execute_json_publication(
            &project_gate,
            &target_root,
            &publication_request,
            &apply_writes(ApplyPostimages {
                target_branch: &target_info.branch,
                merge_request_id: mr_id,
                generation_before,
                generation_after,
                branch_before: target_branch,
                branch_after,
                merge_request_before: before_mr,
                merge_request_after: mr,
            }),
            &response,
        )?;
        Ok((200, published))
    })();
    if apply_clone_bytes != 0 {
        release_http_temporary(
            context,
            apply_clone_bytes,
            "release apply merge-request clone",
        )?;
    }
    if apply_generation_retained_bytes != 0 {
        release_http_retained(
            context,
            apply_generation_retained_bytes,
            "release apply generation authority JSON",
        )?;
    }
    if apply_target_branch_retained_bytes != 0 {
        release_http_retained(
            context,
            apply_target_branch_retained_bytes,
            "release apply target branch authority JSON",
        )?;
    }
    for growth in apply_growth_bytes
        .into_iter()
        .rev()
        .filter(|growth| *growth != 0)
    {
        release_http_retained(
            context,
            growth,
            "release apply merge-request authority growth",
        )?;
    }
    if apply_generation_postimage_retained_bytes != 0 {
        release_http_retained(
            context,
            apply_generation_postimage_retained_bytes,
            "release apply generation postimage",
        )?;
    }
    if apply_branch_postimage_retained_bytes != 0 {
        release_http_retained(
            context,
            apply_branch_postimage_retained_bytes,
            "release apply branch postimage",
        )?;
    }
    release_http_retained(
        context,
        apply_mr_retained_bytes,
        "release apply merge-request authority JSON",
    )?;
    result
}

#[cfg(test)]
fn handle_http_request_apply(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    let mut route_intent = None;
    handle_http_request_apply_with_request_source(
        storage_root,
        org,
        app,
        mr_id,
        Some(request),
        &mut route_intent,
        context,
    )
}

fn handle_http_request_apply_with_route_intent(
    storage_root: &Path,
    route_intent: &mut Option<HttpMutationRouteIntent>,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    handle_http_request_apply_with_request_source(
        storage_root,
        "",
        "",
        "",
        None,
        route_intent,
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_http_request_apply_with_request_source(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    borrowed_request: Option<&Value>,
    route_intent: &mut Option<HttpMutationRouteIntent>,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("apply planning")?;
    if let Some(intent) = route_intent.as_mut() {
        intent.prepare_apply_initial_from_routing(
            storage_root,
            context
                .ledger
                .as_mut()
                .expect("apply derived-intent ledger must be present"),
        )?;
    }
    let mut direct_apply = if route_intent.is_none() {
        let request = borrowed_request.expect("direct apply route retains its borrowed request");
        let project_root = http_project_root(storage_root, org, app);
        Some(HttpApplyDerivedIntent {
            mr_path: crate::remote::remote_merge_request_path(&project_root, mr_id),
            project_root,
            merge_request_id: mr_id.to_string(),
            actor: request
                .get("actor")
                .and_then(Value::as_str)
                .unwrap_or("local")
                .to_string(),
            request_target: super::signatures::remote_request_target(
                org,
                app,
                "apply",
                None,
                Some(mr_id),
            ),
            identity: None,
        })
    } else {
        None
    };
    let (
        request,
        apply,
        route_ledger_identity,
        route_request_retained,
        mut route_derived_retained,
        route_routing_released,
    ) = if let Some(intent) = route_intent.as_mut() {
        let routing_released = intent.source_routing.is_none() && intent.routing.is_none();
        let HttpMutationRouteIntent {
            ledger_identity,
            request,
            request_retained_bytes,
            derived_retained_bytes,
            derived,
            ..
        } = intent;
        let HttpMutationDerivedIntent::Apply(apply) = derived else {
            unreachable!("prepared apply route must own apply-derived inputs")
        };
        (
            request
                .as_ref()
                .expect("production apply route retains its canonical request"),
            apply.as_mut(),
            Some(*ledger_identity),
            Some(
                request_retained_bytes.expect("production apply route retains its request scalar"),
            ),
            Some(derived_retained_bytes),
            routing_released,
        )
    } else {
        (
            borrowed_request.expect("direct apply route retains its borrowed request"),
            direct_apply
                .as_mut()
                .expect("direct apply route retains its derived inputs"),
            None,
            None,
            None,
            false,
        )
    };
    if context.prepared.is_some() {
        return Err(CliError::InvalidRepository(
            "apply route started with an existing prepared response".to_string(),
        ));
    }
    let project_root = apply.project_root.as_path();
    let mr_path = apply.mr_path.as_path();
    let mr_id = apply.merge_request_id.as_str();
    let actor = apply.actor.as_str();
    let request_target = apply.request_target.as_str();
    // Authenticate before gate acquisition or merge-request-specific work can
    // become observable. Nonce-cache validation and ownership stay deferred
    // until the proof owner exists, so Store binding still receives a ledger
    // whose current ownership is zero.
    super::signatures::preflight_remote_request_signature_with_work_and_poll(
        project_root,
        "apply",
        actor,
        request_target,
        request,
        request.get("request_signature"),
        context
            .ledger
            .as_mut()
            .expect("apply signature preflight ledger must be present"),
        || context.deadline.check("apply request signature preflight"),
    )?;
    let project_gate =
        acquire_with_http_deadline(request, context.deadline, "apply project gate", |options| {
            RemoteProjectGate::acquire_exclusive(project_root, options)
        })?;

    let Some(accounted_mr) =
        read_optional_http_json_accounted(mr_path, context, "apply merge-request read")?
    else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    let (mr, mr_retained) = accounted_mr.into_parts();
    let identity = (|| -> Result<Result<ApplyMutationIdentity, (u16, Value)>, CliError> {
        validate_http_merge_request_record_shape(storage_root, &mr)?;
        let source_info = parse_cf_http_url(&required_string(&mr, &["source_url"])?)?;
        let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
        let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
        if target_root != project_root {
            return Ok(Err((
                400,
                json!({"error": "target_url must belong to this project"}),
            )));
        }
        let subject_digest = validate_merge_request_subject_for_roots_with_work_and_poll(
            &mr,
            &target_root,
            &source_info.branch,
            &target_info.branch,
            context
                .ledger
                .as_mut()
                .expect("apply ledger must be present for subject validation"),
            || context.deadline.check("apply subject validation"),
        )?;
        let source_head = mr
            .get("source_head")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidMarker("missing source_head".to_string()))?;
        let target_head_at_request = mr
            .get("target_head_at_request")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::InvalidMarker("missing target_head_at_request".to_string()))?;
        let stable_project_id = project_gate.project().stable_id();
        let idempotency_key = request
            .get("idempotency_key")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty());
        let semantic_bytes = account_compact_json(
            context
                .ledger
                .as_mut()
                .expect("apply publication measurement ledger must be present"),
            &AccountedApplySemantic {
                stable_project_id,
                merge_request_id: mr_id,
                subject_digest: &subject_digest,
                target_branch: &target_info.branch,
                source_head,
                target_head_at_request,
            },
            codefire_core::DecisionWorkPhase::PublicationPreflight,
            "measure HTTP apply publication semantic owner",
        )?;
        let response_bytes = account_compact_json(
            context
                .ledger
                .as_mut()
                .expect("apply response measurement ledger must be present"),
            &AccountedApplyResponse {
                id: mr_id,
                target_branch: &target_info.branch,
                head: source_head,
            },
            codefire_core::DecisionWorkPhase::PublicationPreflight,
            "measure HTTP apply response owner",
        )?;
        let root_count = if source_head == target_head_at_request {
            1_u64
        } else {
            2_u64
        };
        let roots_bytes = sum_http_owner_bytes([
            codefire_core::LOGICAL_FRAME_BASE_BYTES,
            logical_string_owner_bytes(source_head)?,
            if root_count == 2 {
                logical_string_owner_bytes(target_head_at_request)?
            } else {
                0
            },
        ])?;
        let publication_bytes = sum_http_owner_bytes([
            logical_string_owner_bytes("remote_request_apply")?,
            logical_string_owner_bytes(actor)?,
            idempotency_key
                .map(logical_string_owner_bytes)
                .transpose()?
                .unwrap_or_default(),
            roots_bytes,
            semantic_bytes,
        ])?;
        let identity_bytes = sum_http_owner_bytes([
            logical_string_owner_bytes(actor)?,
            logical_path_owner_bytes(&target_root)?,
            logical_joined_path_owner_bytes(&target_root, "objects")?,
            logical_string_owner_bytes(&target_info.branch)?,
            logical_string_owner_bytes(source_head)?,
            logical_string_owner_bytes(target_head_at_request)?,
            logical_string_owner_bytes(&subject_digest)?,
            publication_bytes,
            response_bytes,
        ])?;
        if let (Some(ledger_identity), Some(derived_retained_bytes)) =
            (route_ledger_identity, route_derived_retained.as_deref_mut())
        {
            HttpMutationRouteIntent::acquire_derived_retained_parts(
                ledger_identity,
                derived_retained_bytes,
                context
                    .ledger
                    .as_mut()
                    .expect("apply identity-derived ledger must be present"),
                identity_bytes,
                "reserve HTTP apply identity-derived ownership",
            )?;
        }
        let publication_request = apply_publication(
            actor,
            idempotency_key.map(str::to_string),
            &ApplyPublicationIdentity {
                stable_project_id,
                merge_request_id: mr_id,
                subject_digest: &subject_digest,
                target_branch: &target_info.branch,
                source_head,
                target_head_at_request,
            },
        );
        let response = apply_result(mr_id, &target_info.branch, source_head);
        Ok(Ok(ApplyMutationIdentity {
            actor: actor.to_string(),
            objects: target_root.join("objects"),
            target_root,
            target_branch: target_info.branch,
            source_head: source_head.to_string(),
            target_head_at_request: target_head_at_request.to_string(),
            subject_digest,
            publication_request,
            response,
        }))
    })();
    drop(mr);
    release_http_retained(
        context,
        mr_retained,
        "release apply validation merge-request authority JSON",
    )?;
    let identity = match identity? {
        Ok(identity) => identity,
        Err(response) => return Ok(response),
    };
    apply.identity = Some(identity);
    let identity = apply
        .identity
        .as_ref()
        .expect("apply route retains its mutation identity");
    preflight_publication_signature_with_recovery_and_poll(
        &project_gate,
        &identity.target_root,
        &identity.publication_request,
        "apply",
        actor,
        request_target,
        request,
        request.get("request_signature"),
        context
            .ledger
            .as_mut()
            .expect("apply nonce preflight ledger must be present"),
        || context.deadline.check("apply request nonce preflight"),
    )?;
    let validation_ledger = context
        .ledger
        .as_ref()
        .expect("apply validation ledger must be present");
    if let (Some(ledger_identity), Some(request_retained), Some(derived_retained)) = (
        route_ledger_identity,
        route_request_retained,
        route_derived_retained.as_deref().copied(),
    ) {
        let expected = request_retained
            .checked_add(derived_retained)
            .ok_or_else(|| {
                CliError::InvalidRepository("HTTP apply route ownership size overflow".to_string())
            })?;
        let current = &validation_ledger.snapshot().current;
        let retained = current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let structural = apply_store_structure_is_complete(route_routing_released, apply);
        #[cfg(test)]
        record_http_mutation_store_binding(expected, retained, structural);
        let unrelated = current.iter().any(|(dimension, amount)| {
            *dimension != codefire_core::DecisionWorkDimension::RetainedBytes && *amount != 0
        });
        if !validation_ledger.matches_identity(ledger_identity)
            || !structural
            || retained != expected
            || unrelated
            || validation_ledger.has_open_permits()
        {
            return Err(CliError::InvalidRepository(
                "HTTP apply Store binding requires exact route-derived ownership".to_string(),
            ));
        }
    } else if validation_ledger
        .snapshot()
        .current
        .values()
        .any(|amount| *amount != 0)
    {
        return Err(CliError::InvalidRepository(
            "apply Store binding requires all route-owned current memory to be released"
                .to_string(),
        ));
    }

    let owned_ledger = context
        .ledger
        .take()
        .expect("apply validation ledger must be present");
    let mut session = match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger_and_cancellation(
        &identity.objects,
        owned_ledger,
        || context.deadline.is_expired(),
    ) {
        Ok(session) => session,
        Err(error) => {
            let (error, ledger) = error.into_parts();
            *context.ledger = Some(ledger);
            if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                return Err(context.deadline.error("apply validation session begin"));
            }
            return Err(http_store_error(error));
        }
    };
    for root in &identity.publication_request.object_roots {
        if let Err(error) =
            session.validate_commit_with_cancellation(root, || context.deadline.is_expired())
        {
            let ledger = if route_intent.is_some() {
                discard_http_validation_session(&mut session)
            } else {
                abort_http_validation_session(&mut session, http_store_terminal_reason(&error))
            };
            *context.ledger = Some(ledger);
            if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                return Err(context.deadline.error("apply publication-root validation"));
            }
            return Err(http_store_error(error));
        }
    }
    let bound =
        match session.bind_prepublication_validation(&identity.publication_request.object_roots) {
            Ok(bound) => bound,
            Err(error) => {
                let (error, mut session) = error.into_parts();
                let ledger = discard_http_validation_session(&mut session);
                *context.ledger = Some(ledger);
                return Err(http_store_error(error));
            }
        };
    let mut owner = match PreparedHttpMutationTransaction::from_bound_validation(bound) {
        Ok(owner) => owner,
        Err(failure) => {
            let (error, mut ledger) =
                recover_http_validation_build_failure(failure, route_intent.is_some());
            if let Some(mut intent) = route_intent.take() {
                if let Err(cleanup) = intent.release(&mut ledger) {
                    fail_closed_http_mutation_drop(format!(
                        "apply validation-owner construction failed ({error}) and route-intent cleanup failed: {cleanup}"
                    ));
                }
            }
            *context.ledger = Some(ledger);
            return Err(error);
        }
    };

    let pending_nonce = validate_publication_signature_with_recovery_and_poll(
        &project_gate,
        &identity.target_root,
        &identity.publication_request,
        "apply",
        actor,
        request_target,
        request,
        request.get("request_signature"),
        owner
            .ledger
            .as_mut()
            .expect("apply mutation owner retains its ledger"),
        || context.deadline.check("apply request signature validation"),
    );
    if route_intent.is_some() {
        if let Err(error) = owner.attach_route_intent(route_intent) {
            if let Some(mut intent) = route_intent.take() {
                if let Err(cleanup) = intent.release(
                    owner
                        .ledger
                        .as_mut()
                        .expect("apply rejected route intent retains its ledger"),
                ) {
                    fail_closed_http_mutation_drop(format!(
                        "apply route-intent attachment failed ({error}) and cleanup failed: {cleanup}"
                    ));
                }
            }
            let ledger = cancel_http_mutation_owner(&mut owner);
            *context.ledger = Some(ledger);
            return Err(error);
        }
    }
    let mut pending_nonce = match pending_nonce {
        Ok(nonce) => Some(nonce),
        Err(error) => {
            let ledger = cancel_http_mutation_owner(&mut owner);
            *context.ledger = Some(ledger);
            return Err(error);
        }
    };
    let mut authority_accounting = None;
    let mut fresh_rejection = None;
    let publication_plan = {
        let planning_route_intent = owner.route_intent.take();
        let planning_apply = planning_route_intent
            .as_ref()
            .map(HttpMutationRouteIntent::apply)
            .or(direct_apply.as_ref())
            .expect("apply planning retains its typed derived owner");
        let identity = planning_apply
            .identity
            .as_ref()
            .expect("apply planning retains its mutation identity");
        let merge_request_id = planning_apply.merge_request_id.as_str();
        let result = owner.plan_publication(
            &project_gate,
            &identity.target_root,
            &identity.publication_request,
            &mut pending_nonce,
            &identity.response,
            |subject, ledger| {
                if !validate_apply_semantics_with_provisional_subject(
                    &identity.target_root,
                    &identity.source_head,
                    &identity.target_head_at_request,
                    subject,
                    ledger,
                    context.deadline,
                )? {
                    fresh_rejection = Some((
                        409,
                        json!({"error": "request apply rejected: source is not a fast-forward of target"}),
                    ));
                    return Err(CliError::InvalidRepository(
                        APPLY_FRESH_REJECTION_SENTINEL.to_string(),
                    ));
                }
                match prepare_apply_authority_values(
                    storage_root,
                    &planning_apply.project_root,
                    &planning_apply.mr_path,
                    merge_request_id,
                    identity,
                    ledger,
                    context.deadline,
                )? {
                    ApplyAuthorityPreparation::Ready(values, accounting) => {
                        authority_accounting = Some(accounting);
                        Ok((*values).into_writes(
                            &identity.target_branch,
                            merge_request_id,
                        ))
                    }
                    ApplyAuthorityPreparation::Rejected(status, payload) => {
                        fresh_rejection = Some((status, payload));
                        Err(CliError::InvalidRepository(
                            APPLY_FRESH_REJECTION_SENTINEL.to_string(),
                        ))
                    }
                }
            },
        );
        owner.route_intent = planning_route_intent;
        result
    };
    let release = authority_accounting.map_or(Ok(()), |mut accounting| {
        accounting.release(
            owner
                .ledger
                .as_mut()
                .expect("apply mutation owner retains its ledger"),
        )
    });
    release_http_pending_nonce(&mut pending_nonce, &mut owner, "apply");
    if let Err(error) = release {
        let ledger = cancel_http_mutation_owner(&mut owner);
        *context.ledger = Some(ledger);
        return Err(error);
    }
    match publication_plan {
        Ok(()) => debug_assert!(fresh_rejection.is_none()),
        Err(CliError::InvalidRepository(message))
            if message == APPLY_FRESH_REJECTION_SENTINEL && fresh_rejection.is_some() =>
        {
            let response = fresh_rejection
                .take()
                .expect("apply fresh rejection was matched by its private sentinel");
            let ledger = cancel_http_mutation_owner(&mut owner);
            *context.ledger = Some(ledger);
            return Ok(response);
        }
        Err(error) => {
            let ledger = cancel_http_mutation_owner(&mut owner);
            *context.ledger = Some(ledger);
            return Err(error);
        }
    }
    let response_preparation = if owner.route_intent.is_some() {
        owner.prepare_and_attach_responses_from_apply_route(200, context.deadline)
    } else {
        owner.prepare_and_attach_responses(
            200,
            &direct_apply
                .as_ref()
                .and_then(|apply| apply.identity.as_ref())
                .expect("direct apply retains its mutation identity")
                .response,
            context.deadline,
        )
    };
    if let Err(error) = response_preparation {
        let ledger = cancel_http_mutation_owner(&mut owner);
        *context.ledger = Some(ledger);
        return Err(error);
    }
    if let Err(error) = owner.release_route_intent() {
        let ledger = cancel_http_mutation_owner(&mut owner);
        *context.ledger = Some(ledger);
        return Err(error);
    }

    if let Err(error) = owner.revalidate_and_seal(|| context.deadline.is_expired()) {
        let error = if matches!(
            &error,
            CliError::Store(store)
                if matches!(store.as_ref(), codefire_store::StoreError::ValidationCancelled)
        ) {
            context.deadline.error("apply prepublication revalidation")
        } else {
            error
        };
        let ledger = cancel_http_mutation_owner(&mut owner);
        *context.ledger = Some(ledger);
        return Err(error);
    }
    let status = match owner.commit_and_select_success() {
        Ok(()) => 200,
        Err(_) => {
            if let Err(error) = owner.select_durable_after_commit_error() {
                let ledger = cancel_http_mutation_owner(&mut owner);
                *context.ledger = Some(ledger);
                return Err(error);
            }
            503
        }
    };
    if let Err(error) = owner.validate_http_handoff() {
        let ledger = cancel_http_mutation_owner(&mut owner);
        *context.ledger = Some(ledger);
        return Err(error);
    }
    let (ledger, prepared) = owner.into_http_parts_after_validation();
    debug_assert_eq!(prepared.status, status);
    *context.ledger = Some(ledger);
    *context.prepared = Some(prepared);
    Ok((status, Value::Null))
}

fn validate_http_merge_request_record_with_context(
    storage_root: &Path,
    mr: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(), CliError> {
    validate_http_merge_request_record_shape(storage_root, mr)?;
    let source_info = parse_cf_http_url(&required_string(mr, &["source_url"])?)?;
    let source_root = http_project_root(storage_root, &source_info.org, &source_info.app);
    let target_info = parse_cf_http_url(&required_string(mr, &["target_url"])?)?;
    let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
    let source_head = required_string(mr, &["source_head"])?;
    let target_head = required_string(mr, &["target_head_at_request"])?;
    let applied_head = mr
        .get("applied_head")
        .and_then(Value::as_str)
        .map(str::to_string);
    let source_objects = if mr.get("subject").is_some() {
        remote_dirs(&target_root).objects
    } else {
        remote_dirs(&source_root).objects
    };
    let target_objects = remote_dirs(&target_root).objects;
    if source_objects == target_objects {
        let mut commits = vec![source_head.as_str(), target_head.as_str()];
        if let Some(applied_head) = applied_head.as_deref() {
            commits.push(applied_head);
        }
        with_http_sealed_validation(
            context,
            &target_objects,
            &commits,
            "merge-request sealed validation",
            |_| Ok(()),
        )?;
    } else {
        with_http_sealed_validation(
            context,
            &source_objects,
            &[source_head.as_str()],
            "merge-request source validation",
            |_| Ok(()),
        )?;
        let mut commits = vec![target_head.as_str()];
        if let Some(applied_head) = applied_head.as_deref() {
            commits.push(applied_head);
        }
        with_http_sealed_validation(
            context,
            &target_objects,
            &commits,
            "merge-request target validation",
            |_| Ok(()),
        )?;
    }
    if mr.get("subject").is_some() {
        validate_merge_request_subject_for_roots_with_work_and_poll(
            mr,
            &target_root,
            &source_info.branch,
            &target_info.branch,
            context
                .ledger
                .as_mut()
                .expect("merge-request ledger must return after sealed validation"),
            || context.deadline.check("merge-request subject validation"),
        )?;
    }
    Ok(())
}

fn validate_http_merge_request_record_shape(
    _storage_root: &Path,
    mr: &Value,
) -> Result<(), CliError> {
    validate_record_version(mr, "remote_merge_request")?;
    validate_remote_component(&required_string(mr, &["id"])?, "merge-request ID")?;
    let source_info = parse_cf_http_url(&required_string(mr, &["source_url"])?)?;
    let target_info = parse_cf_http_url(&required_string(mr, &["target_url"])?)?;
    if source_info.endpoint != target_info.endpoint {
        return Err(CliError::Usage(
            "HTTP merge requests across different remote endpoints are not supported".to_string(),
        ));
    }
    required_string(mr, &["source_head"])?;
    required_string(mr, &["target_head_at_request"])?;
    Ok(())
}

fn http_merge_request_is_stale(
    storage_root: &Path,
    mr: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<bool, CliError> {
    let target = parse_cf_http_url(&required_string(mr, &["target_url"])?)?;
    let target_root = http_project_root(storage_root, &target.org, &target.app);
    let current_target = read_optional_http_json_accounted(
        &remote_branch_path(&target_root, &target.branch),
        context,
        "merge-request stale target branch read",
    )?;
    let Some(accounted_target) = current_target else {
        return Ok(false);
    };
    let (branch, retained_bytes) = accounted_target.into_parts();
    let stale = (|| -> Result<bool, CliError> {
        validate_remote_branch_record(&branch)?;
        Ok(branch.get("head").and_then(Value::as_str)
            != mr.get("target_head_at_request").and_then(Value::as_str))
    })();
    release_http_retained(
        context,
        retained_bytes,
        "release stale target branch authority JSON",
    )?;
    stale
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, ErrorKind, Read, Write};
    use std::net::TcpStream;
    use std::sync::Mutex;
    use std::thread;
    use std::time::{Duration, Instant};

    static HTTP_TIMEOUT_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn upload_bootstrap_footprint_counts_object_bytes_without_materialization() {
        let record = json!({
            "id": "CF-COMMIT-fixture",
            "type": "commit",
            "payload": {"message": "multibyte-火", "parents": []},
        });
        let rendered =
            u64::try_from(serde_json::to_vec_pretty(&record).unwrap().len()).unwrap() + 1;
        let request = json!({"idempotency_key": "footprint"});
        let empty = http_upload_bootstrap_footprint(
            &request,
            "main",
            "local-main",
            "uploader",
            "CF-COMMIT-fixture",
            &[],
        )
        .unwrap();
        let records = [record.clone(), record];
        let populated = http_upload_bootstrap_footprint(
            &request,
            "main",
            "local-main",
            "uploader",
            "CF-COMMIT-fixture",
            &records,
        )
        .unwrap();

        assert_eq!(populated.entries - empty.entries, 4);
        assert_eq!(populated.payload_bytes - empty.payload_bytes, rendered * 4);
        let without_receipt = http_upload_bootstrap_footprint(
            &json!({}),
            "main",
            "local-main",
            "uploader",
            "CF-COMMIT-fixture",
            &[],
        )
        .unwrap();
        assert_eq!(empty.entries - without_receipt.entries, 3);
        assert_eq!(
            empty.payload_bytes - without_receipt.payload_bytes,
            1024 * 1024
        );
    }

    #[test]
    fn upload_bootstrap_entry_and_payload_caps_have_exact_boundaries() {
        let max_entries = BootstrapStageFootprint::maximum_entries();
        assert!(BootstrapStageFootprint::maximum_payload_bytes(max_entries - 1).is_some());
        assert!(BootstrapStageFootprint::maximum_payload_bytes(max_entries).is_some());
        assert!(BootstrapStageFootprint::maximum_payload_bytes(max_entries + 1).is_none());
        let entry_error =
            http_upload_bootstrap_budget_error(BootstrapStageFootprint::new(max_entries + 1, 0));
        assert!(matches!(
            entry_error,
            CliError::ResourceBudget(ref exhaustion)
                if exhaustion.dimension == codefire_core::DecisionWorkDimension::GraphNodes
                    && exhaustion.observed == max_entries + 1
                    && exhaustion.limit == max_entries
        ));

        let entries = 13;
        let max_payload = BootstrapStageFootprint::maximum_payload_bytes(entries).unwrap();
        for payload in [max_payload - 1, max_payload] {
            assert!(payload <= BootstrapStageFootprint::maximum_payload_bytes(entries).unwrap());
        }
        let payload_error = http_upload_bootstrap_budget_error(BootstrapStageFootprint::new(
            entries,
            max_payload + 1,
        ));
        assert!(matches!(
            payload_error,
            CliError::ResourceBudget(ref exhaustion)
                if exhaustion.dimension == codefire_core::DecisionWorkDimension::AggregateBytes
                    && exhaustion.observed == max_payload + 1
                    && exhaustion.limit == max_payload
        ));
    }

    type ReviewBoundaryRunResult = (
        Result<(u16, Value), CliError>,
        Option<codefire_core::DecisionWorkLedger>,
        PathBuf,
        Value,
    );

    fn preflight_http_upload_for_test(
        existing_objects: &Path,
        head: &str,
        records: &[Value],
    ) -> Result<HttpUploadPreflight, CliError> {
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());
        preflight_http_upload(existing_objects, head, records, &mut ledger, &deadline)
    }

    fn upload_preflight_ledger() -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload preflight cleanup",
        )
        .unwrap()
    }

    fn assert_upload_preflight_current_zero(ledger: &codefire_core::DecisionWorkLedger) {
        for dimension in [
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            codefire_core::DecisionWorkDimension::RetainedBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0,
                "{dimension:?} must be released after upload preflight"
            );
        }
    }

    fn test_tree_snapshot(root: &Path) -> (bool, BTreeMap<PathBuf, Option<Vec<u8>>>) {
        fn visit(root: &Path, current: &Path, snapshot: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
            if !current.exists() {
                return;
            }
            for entry in fs::read_dir(current).unwrap() {
                let path = entry.unwrap().path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                if path.is_dir() {
                    snapshot.insert(relative, None);
                    visit(root, &path, snapshot);
                } else {
                    snapshot.insert(relative, Some(fs::read(path).unwrap()));
                }
            }
        }

        let mut snapshot = BTreeMap::new();
        visit(root, root, &mut snapshot);
        (root.exists(), snapshot)
    }

    fn write_http_test_commit(objects: &Path) -> String {
        let roots = [
            (
                "content_manifest",
                "content_manifest",
                json!({"type": "content_manifest", "version": 1, "entries": []}),
            ),
            (
                "atom_index",
                "atom_index",
                json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
            ),
            (
                "trace_graph",
                "trace_graph",
                json!({"type": "trace_graph", "version": 1, "links": []}),
            ),
            (
                "fire_delta",
                "fire_ledger",
                json!({"type": "fire_ledger", "version": 1, "fires": []}),
            ),
            (
                "resolution_ledger",
                "resolution_ledger",
                json!({"type": "resolution_ledger", "version": 1, "resolutions": []}),
            ),
            (
                "verification",
                "verification",
                json!({
                    "type": "verification",
                    "version": 1,
                    "result": "passed",
                    "trace_completeness_required": false,
                    "open_required_fires": 0,
                    "failed_checks": [],
                    "missing_required_links": [],
                    "stale_resolutions": [],
                    "missing_evidence_refs": [],
                    "duplicate_atom_ids": [],
                    "verified_at": "2026-07-20T00:00:00Z"
                }),
            ),
            (
                "policy",
                "policy",
                json!({
                    "type": "policy",
                    "version": 1,
                    "policy": {
                        "require_no_required_fires": true,
                        "require_no_stale_resolutions": true,
                        "require_trace_completeness": false,
                        "require_verification_success": true,
                        "reject_duplicate_atom_ids": true,
                        "no_change_required_requires_rationale": true,
                        "required_links": {},
                        "verification": []
                    }
                }),
            ),
        ]
        .into_iter()
        .map(|(root, kind, payload)| {
            let object_id = codefire_store::store_object(objects, kind, payload).unwrap();
            (root.to_string(), Value::String(object_id))
        })
        .collect::<serde_json::Map<_, _>>();
        let certificate = [
            ("result", json!("consistent")),
            ("open_required_fires", json!(0)),
            ("failed_checks", json!(0)),
            ("missing_required_links", json!(0)),
            ("stale_resolutions", json!(0)),
            ("missing_evidence_refs", json!(0)),
            ("duplicate_atom_ids", json!(0)),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect::<serde_json::Map<_, _>>();
        codefire_store::store_object(
            objects,
            "commit",
            codefire_store::commit_payload(Vec::new(), roots, certificate),
        )
        .unwrap()
    }

    fn commit_id_between(left: &str, right: &str) -> String {
        fn hex_successor(byte: u8) -> Option<u8> {
            match byte {
                b'0'..=b'8' | b'a'..=b'e' => Some(byte + 1),
                b'9' => Some(b'a'),
                _ => None,
            }
        }

        let (low, high) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        let prefix = "CF-COMMIT-";
        let low_suffix = low.strip_prefix(prefix).unwrap().as_bytes();
        let high_suffix = high.strip_prefix(prefix).unwrap().as_bytes();
        let mut candidate = low_suffix.to_vec();
        for index in (0..candidate.len()).rev() {
            let Some(next) = hex_successor(candidate[index]) else {
                continue;
            };
            candidate[index] = next;
            candidate[index + 1..].fill(b'0');
            if candidate.as_slice() < high_suffix {
                return format!("{prefix}{}", String::from_utf8(candidate).unwrap());
            }
            candidate.copy_from_slice(low_suffix);
        }
        panic!("test commit IDs have no representable ID between them");
    }

    fn http_route_context_for_test<'a>(
        ledger: &'a mut Option<codefire_core::DecisionWorkLedger>,
        prepared: &'a mut Option<PreparedHttpResponse>,
        deadline: &'a HttpRequestDeadline,
    ) -> HttpRouteContext<'a> {
        HttpRouteContext {
            ledger,
            prepared,
            deadline,
            payload_owner: None,
        }
    }

    fn write_unmaterialized_review_fixture(storage_root: &Path, mr_id: &str) -> (PathBuf, Value) {
        let project_root = http_project_root(storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let mr = json!({
            "version": 1,
            "id": mr_id,
            "source_url": "cf+http://local/org/app/main",
            "source_head": head,
            "target_url": "cf+http://local/org/app/main",
            "target_head_at_request": head,
            "status": "open",
            "created_by": "test",
            "created_at": "2026-07-20T00:00:00Z",
            "reviews": [{
                "reviewer": "legacy",
                "decision": "approve",
                "comment": "unbound",
                "reviewed_at": "2026-07-19T00:00:00Z"
            }],
        });
        fs::write(
            remote_merge_request_path(&project_root, mr_id),
            codefire_store::canonical_json(&mr).unwrap(),
        )
        .unwrap();
        (project_root, mr)
    }

    fn write_approved_apply_fixture(storage_root: &Path, mr_id: &str) -> PathBuf {
        let project_root = http_project_root(storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        let branch = json!({"version": 1, "name": "main", "head": head});
        fs::write(
            remote_branch_path(&project_root, "main"),
            codefire_store::canonical_json(&branch).unwrap(),
        )
        .unwrap();
        let records = collect_object_records(&objects, &head).unwrap();
        let object_set_digest = source_object_set_digest(&records).unwrap();
        let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
            mr_id,
            source_project_root: &project_root,
            target_project_root: &project_root,
            source_branch: "main",
            target_branch: "main",
            source_head: &head,
            target_head: &head,
            source_object_set_digest: &object_set_digest,
        })
        .unwrap();
        let subject_digest = merge_request_subject_digest(&subject).unwrap();
        let mr = json!({
            "version": 1,
            "id": mr_id,
            "source_url": "cf+http://local/org/app/main",
            "source_head": head,
            "target_url": "cf+http://local/org/app/main",
            "target_head_at_request": head,
            "status": "approved",
            "created_by": "test",
            "created_at": "2026-07-20T00:00:00Z",
            "subject": subject,
            "subject_digest": subject_digest,
            "reviews": [{
                "reviewer": "reviewer",
                "decision": "approve",
                "comment": "approved",
                "subject_digest": subject_digest,
                "reviewed_at": "2026-07-20T00:00:00Z"
            }],
        });
        fs::write(
            remote_merge_request_path(&project_root, mr_id),
            codefire_store::canonical_json(&mr).unwrap(),
        )
        .unwrap();
        project_root
    }

    fn require_apply_request_signatures(project_root: &Path) {
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "nonce_ttl_seconds": 300,
                    "max_nonce_cache_entries": 32,
                    "keys": {
                        "reviewer-request": {
                            "secret": "reviewer-request-key",
                            "signers": ["reviewer"]
                        }
                    }
                }
            }),
        )
        .unwrap();
    }

    fn sign_apply_request(mr_id: &str, request: &mut Value) {
        request.as_object_mut().unwrap().remove("request_signature");
        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "apply",
            None,
            Some(mr_id),
        );
        request["request_signature"] = super::super::signatures::sign_remote_request_with_key(
            "apply",
            "reviewer",
            &target,
            request,
            "reviewer-request",
            "reviewer-request-key",
        )
        .unwrap();
    }

    fn write_upload_request_signature_policy(project_root: &Path, require_commit: bool) {
        let mut policy = json!({
            "request_signatures": {
                "required": true,
                "nonce_ttl_seconds": 300,
                "max_nonce_cache_entries": 32,
                "keys": {
                    "uploader-request": {
                        "secret": "uploader-request-key",
                        "signers": ["uploader"]
                    }
                }
            }
        });
        if require_commit {
            policy["commit_signatures"] = json!({
                "required": true,
                "keys": {"uploader": "commit-key"}
            });
        }
        write_json_atomic(&project_root.join("server_policy.json"), &policy).unwrap();
    }

    fn sign_upload_request(request: &mut Value) {
        request.as_object_mut().unwrap().remove("request_signature");
        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "upload",
            Some("main"),
            None,
        );
        request["request_signature"] = super::super::signatures::sign_remote_request_with_key(
            "upload",
            "uploader",
            &target,
            request,
            "uploader-request",
            "uploader-request-key",
        )
        .unwrap();
    }

    fn upload_request_fixture(storage_root: &Path, idempotency_key: &str) -> (PathBuf, Value) {
        let project_root = http_project_root(storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let incoming_objects = storage_root.join("incoming-objects");
        let head = write_http_test_commit(&incoming_objects);
        let records = collect_object_records(&incoming_objects, &head).unwrap();
        (
            project_root,
            json!({
                "branch": "local-main",
                "head": head,
                "objects": records,
                "actor": "uploader",
                "idempotency_key": idempotency_key,
            }),
        )
    }

    type AccountedUploadResult = (
        Result<(u16, Value), CliError>,
        Option<codefire_core::DecisionWorkLedger>,
        Option<PreparedHttpResponse>,
    );

    fn run_accounted_upload(storage_root: &Path, request: &Value) -> AccountedUploadResult {
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "accounted HTTP upload fixture",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let result = handle_http_upload_with_ledger(
            storage_root,
            "org",
            "app",
            "main",
            request,
            &mut http_route_context_for_test(&mut ledger, &mut prepared, &deadline),
        );
        (result, ledger, prepared)
    }

    fn release_prepared_upload_response(
        ledger: &mut Option<codefire_core::DecisionWorkLedger>,
        prepared: &mut Option<PreparedHttpResponse>,
    ) -> Value {
        let response = prepared
            .take()
            .expect("upload produced a prepared response");
        let payload = serde_json::from_slice(&response.body).unwrap();
        release_prepared_http_response(
            ledger.as_mut().expect("upload returned its ledger"),
            response,
            "release upload test response",
        )
        .unwrap();
        payload
    }

    fn make_apply_fixture_non_fast_forward(project_root: &Path, mr_id: &str) {
        let objects = remote_dirs(project_root).objects;
        let source_head = read_json(&remote_branch_path(project_root, "main")).unwrap()["head"]
            .as_str()
            .unwrap()
            .to_string();
        let source_record = codefire_store::read_object_record(&objects, &source_head).unwrap();
        let target_head = codefire_store::store_object(
            &objects,
            "commit",
            codefire_store::commit_payload(
                vec![source_head.clone()],
                source_record.payload["roots"].as_object().unwrap().clone(),
                source_record.payload["certificate"]
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .unwrap();
        write_json_atomic(
            &remote_branch_path(project_root, "main"),
            &json!({"version": 1, "name": "main", "head": target_head}),
        )
        .unwrap();
        write_json_atomic(
            &remote_branch_path(project_root, "feature"),
            &json!({"version": 1, "name": "feature", "head": source_head}),
        )
        .unwrap();
        let records = collect_object_records(&objects, &source_head).unwrap();
        let object_set_digest = source_object_set_digest(&records).unwrap();
        let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
            mr_id,
            source_project_root: project_root,
            target_project_root: project_root,
            source_branch: "feature",
            target_branch: "main",
            source_head: &source_head,
            target_head: &target_head,
            source_object_set_digest: &object_set_digest,
        })
        .unwrap();
        let subject_digest = merge_request_subject_digest(&subject).unwrap();
        let mr_path = remote_merge_request_path(project_root, mr_id);
        let mut mr = read_json(&mr_path).unwrap();
        mr["source_url"] = Value::String("cf+http://local/org/app/feature".to_string());
        mr["source_head"] = Value::String(source_head);
        mr["target_head_at_request"] = Value::String(target_head);
        mr["subject"] = subject;
        mr["subject_digest"] = Value::String(subject_digest.clone());
        mr["reviews"][0]["subject_digest"] = Value::String(subject_digest);
        write_json_atomic(&mr_path, &mr).unwrap();
    }

    fn make_apply_fixture_invalid_commit_signature(project_root: &Path, mr_id: &str) {
        let objects = remote_dirs(project_root).objects;
        let original_head = read_json(&remote_branch_path(project_root, "main")).unwrap()["head"]
            .as_str()
            .unwrap()
            .to_string();
        let mut payload = codefire_store::read_object_record(&objects, &original_head)
            .unwrap()
            .payload;
        payload["signature"] = json!({
            "type": "commit_signature",
            "algorithm": "hmac-sha256",
            "signer": "reviewer",
            "signature": "0".repeat(64),
        });
        let signed_head = codefire_store::store_object(&objects, "commit", payload).unwrap();
        write_json_atomic(
            &remote_branch_path(project_root, "main"),
            &json!({"version": 1, "name": "main", "head": signed_head}),
        )
        .unwrap();
        let records = collect_object_records(&objects, &signed_head).unwrap();
        let object_set_digest = source_object_set_digest(&records).unwrap();
        let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
            mr_id,
            source_project_root: project_root,
            target_project_root: project_root,
            source_branch: "main",
            target_branch: "main",
            source_head: &signed_head,
            target_head: &signed_head,
            source_object_set_digest: &object_set_digest,
        })
        .unwrap();
        let subject_digest = merge_request_subject_digest(&subject).unwrap();
        let mr_path = remote_merge_request_path(project_root, mr_id);
        let mut mr = read_json(&mr_path).unwrap();
        mr["source_head"] = Value::String(signed_head.clone());
        mr["target_head_at_request"] = Value::String(signed_head);
        mr["subject"] = subject;
        mr["subject_digest"] = Value::String(subject_digest.clone());
        mr["reviews"][0]["subject_digest"] = Value::String(subject_digest);
        write_json_atomic(&mr_path, &mr).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "nonce_ttl_seconds": 300,
                    "max_nonce_cache_entries": 32,
                    "keys": {
                        "reviewer-request": {
                            "secret": "reviewer-request-key",
                            "signers": ["reviewer"]
                        }
                    }
                },
                "commit_signatures": {
                    "required": true,
                    "keys": {"reviewer": "commit-key"}
                }
            }),
        )
        .unwrap();
    }

    fn release_prepared_apply_response(
        ledger: &mut Option<codefire_core::DecisionWorkLedger>,
        prepared: &mut Option<PreparedHttpResponse>,
    ) -> Value {
        let response = prepared.take().expect("apply produced a prepared response");
        let payload = serde_json::from_slice(&response.body).unwrap();
        release_prepared_http_response(
            ledger.as_mut().expect("apply returned its ledger"),
            response,
            "release apply test response",
        )
        .unwrap();
        payload
    }

    #[test]
    fn http_remote_doctor_endpoint_returns_typed_missing_report() {
        let storage_root = test_storage_root("doctor-missing");
        let body_ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "doctor request body",
        )
        .unwrap();
        let mut request = HttpRequest {
            routing: HttpRequestRoutingOwnership {
                ledger_identity: body_ledger.identity(),
                method: Some("GET".to_string()),
                path: Some("/v1/projects/org/app/doctor?quick=1".to_string()),
                retained_bytes: None,
            },
            body: HttpRequestBodyOwnership::new(json!({}), 0, &body_ledger),
        };

        let (status, payload) = dispatch_http(&storage_root, &mut request).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["schema"], "codefire.remote_doctor.v1");
        assert_eq!(payload["transport"], "http");
        assert_eq!(payload["project_exists"], false);
        assert_eq!(payload["quick"], true);
        assert_eq!(payload["issues"][0]["kind"], "remote_project_missing");
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn branch_list_transfers_authority_ownership_to_exact_projected_payload() {
        let storage_root = test_storage_root("branch-list-accounting");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "branch list accounting",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) =
            handle_http_list(&storage_root, "org", "app", &mut context).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["branches"][0]["name"], "main");
        let expected = codefire_store::canonical_json(&payload).unwrap().len() as u64;
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            expected
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn merge_request_list_releases_each_authority_and_target_branch_owner() {
        let storage_root = test_storage_root("merge-request-list-accounting");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let mr_id = "MR-accounting";
        let records = collect_object_records(&objects, &head).unwrap();
        let object_set_digest = source_object_set_digest(&records).unwrap();
        let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
            mr_id,
            source_project_root: &project_root,
            target_project_root: &project_root,
            source_branch: "main",
            target_branch: "main",
            source_head: &head,
            target_head: &head,
            source_object_set_digest: &object_set_digest,
        })
        .unwrap();
        let subject_digest = merge_request_subject_digest(&subject).unwrap();
        write_json_atomic(
            &remote_merge_request_path(&project_root, mr_id),
            &json!({
                "version": 1,
                "id": mr_id,
                "source_url": "cf+http://local/org/app/main",
                "source_head": head,
                "target_url": "cf+http://local/org/app/main",
                "target_head_at_request": head,
                "status": "open",
                "created_by": "test",
                "created_at": "2026-07-20T00:00:00Z",
                "reviews": [],
                "subject": subject,
                "subject_digest": subject_digest,
            }),
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "merge-request list accounting",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) =
            handle_http_request_list(&storage_root, "org", "app", &mut context).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["merge_requests"][0]["id"], mr_id);
        let expected = codefire_store::canonical_json(&payload).unwrap().len() as u64;
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            expected
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn merge_request_list_reuses_one_sealed_session_for_shared_dag() {
        let storage_root = test_storage_root("merge-request-list-shared-session");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        for mr_id in ["MR-shared-a", "MR-shared-b"] {
            write_json_atomic(
                &remote_merge_request_path(&project_root, mr_id),
                &json!({
                    "version": 1,
                    "id": mr_id,
                    "source_url": "cf+http://local/org/app/main",
                    "source_head": head,
                    "target_url": "cf+http://local/org/app/main",
                    "target_head_at_request": head,
                    "status": "open",
                    "created_by": "test",
                    "created_at": "2026-07-20T00:00:00Z",
                    "reviews": [],
                }),
            )
            .unwrap();
        }
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "merge-request list shared session",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) =
            handle_http_request_list(&storage_root, "org", "app", &mut context).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["merge_requests"].as_array().unwrap().len(), 2);
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::ObjectRecords],
            8,
            "the commit plus its seven roots must be read once across both merge requests"
        );
        let expected = codefire_store::canonical_json(&payload).unwrap().len() as u64;
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            expected
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn merge_request_list_reuses_one_subject_graph_digest_for_shared_materialized_dag() {
        let storage_root = test_storage_root("merge-request-list-shared-subject-digest");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let records = collect_object_records(&objects, &head).unwrap();
        let object_set_digest = source_object_set_digest(&records).unwrap();
        for mr_id in ["MR-subject-a", "MR-subject-b"] {
            let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
                mr_id,
                source_project_root: &project_root,
                target_project_root: &project_root,
                source_branch: "main",
                target_branch: "main",
                source_head: &head,
                target_head: &head,
                source_object_set_digest: &object_set_digest,
            })
            .unwrap();
            let subject_digest = merge_request_subject_digest(&subject).unwrap();
            write_json_atomic(
                &remote_merge_request_path(&project_root, mr_id),
                &json!({
                    "version": 1,
                    "id": mr_id,
                    "source_url": "cf+http://local/org/app/main",
                    "source_head": head,
                    "target_url": "cf+http://local/org/app/main",
                    "target_head_at_request": head,
                    "status": "open",
                    "created_by": "test",
                    "created_at": "2026-07-20T00:00:00Z",
                    "reviews": [],
                    "subject": subject,
                    "subject_digest": subject_digest,
                }),
            )
            .unwrap();
        }
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "merge-request list shared subject digest",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) =
            handle_http_request_list(&storage_root, "org", "app", &mut context).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["merge_requests"].as_array().unwrap().len(), 2);
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::ObjectRecords],
            16,
            "sealed validation and subject graph discovery must each scan the shared DAG once"
        );
        let expected = codefire_store::canonical_json(&payload).unwrap().len() as u64;
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            expected
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn merge_request_subject_digest_cache_limit_minus_exact_plus_one_is_pre_effect() {
        let source_head = format!("CF-COMMIT-{}", "a".repeat(64));
        let digest = format!("sha256:{}", "b".repeat(64));
        let frame_bytes = codefire_core::logical_retained_frame_bytes(&source_head)
            .unwrap()
            .checked_add(codefire_core::logical_retained_frame_bytes(&digest).unwrap())
            .unwrap();
        let limit = codefire_core::DecisionWorkProfile::http_v1()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let exact_filler = limit.checked_sub(frame_bytes).unwrap();

        for (suffix, adjustment, succeeds) in [
            ("minus-one", -1_i64, true),
            ("exact", 0_i64, true),
            ("plus-one", 1_i64, false),
        ] {
            let filler = if adjustment < 0 {
                exact_filler - adjustment.unsigned_abs()
            } else {
                exact_filler + adjustment as u64
            };
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::http_v1(),
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                format!("merge-request subject digest cache {suffix}"),
            )
            .unwrap();
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, filler),
                )
                .unwrap();

            let result =
                retain_merge_request_subject_digest_cache_entry(&mut ledger, &source_head, &digest);
            if succeeds {
                assert_eq!(result.unwrap(), frame_bytes);
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    filler + frame_bytes
                );
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        frame_bytes,
                    )
                    .unwrap();
            } else {
                let error = result.unwrap_err();
                assert!(
                    matches!(error, CliError::ResourceBudget(_)),
                    "unexpected boundary error: {error:?}"
                );
                let diagnostic = error.diagnostic();
                assert_eq!(diagnostic["dimension"], "retained_bytes");
                assert_eq!(diagnostic["observed"], limit + 1);
                assert_eq!(diagnostic["limit"], limit);
                assert_eq!(diagnostic["phase"], "graph_discovery");
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    filler,
                    "limit+1 must fail before retaining the cache entry"
                );
            }
            ledger
                .release(codefire_core::DecisionWorkDimension::RetainedBytes, filler)
                .unwrap();
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                0
            );
        }
    }

    #[test]
    fn merge_request_subject_digest_cache_handoff_cleans_reserve_and_duplicate_failures() {
        let records = vec![json!({"object_id": "CF-POLICY-a", "payload": {"x": 1}})];
        let source_head = format!("CF-COMMIT-{}", "a".repeat(64));
        let mut exhausted = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "merge-request digest cache reserve failure",
        )
        .unwrap();
        let digest = source_object_set_digest_ordered(&records, &[0], &mut exhausted).unwrap();
        let limit = exhausted
            .profile()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let filler = limit - digest.retained_bytes();
        exhausted
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, filler),
            )
            .unwrap();
        let mut cache = BTreeMap::new();

        let error = insert_merge_request_subject_digest_cache_entry(
            &mut exhausted,
            &mut cache,
            &source_head,
            digest,
        )
        .unwrap_err();

        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert!(cache.is_empty());
        assert_eq!(
            exhausted.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            filler
        );
        exhausted
            .release(codefire_core::DecisionWorkDimension::RetainedBytes, filler)
            .unwrap();
        assert_eq!(
            exhausted.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(exhausted.open_permit_count(), 0);

        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "merge-request digest cache duplicate",
        )
        .unwrap();
        let first = source_object_set_digest_ordered(&records, &[0], &mut ledger).unwrap();
        insert_merge_request_subject_digest_cache_entry(
            &mut ledger,
            &mut cache,
            &source_head,
            first,
        )
        .unwrap();
        let retained_after_first =
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
        let duplicate = source_object_set_digest_ordered(&records, &[0], &mut ledger).unwrap();

        let error = insert_merge_request_subject_digest_cache_entry(
            &mut ledger,
            &mut cache,
            &source_head,
            duplicate,
        )
        .unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            retained_after_first
        );
        remove_merge_request_subject_digest_cache_entry(&mut ledger, &mut cache, &source_head)
            .unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(ledger.open_permit_count(), 0);
    }

    #[test]
    fn http_digest_destination_handoff_reserves_before_copy_and_is_retryable() {
        let records = vec![json!({"object_id": "CF-POLICY-a", "payload": {"x": 1}})];
        let mut exhausted = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "HTTP digest destination reserve failure",
        )
        .unwrap();
        let mut digest = source_object_set_digest_ordered(&records, &[0], &mut exhausted).unwrap();
        let source_bytes = digest.retained_bytes();
        let limit = exhausted
            .profile()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let filler = limit - source_bytes;
        exhausted
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, filler),
            )
            .unwrap();

        let error = HttpDigestDestinationOwnership::acquire(
            &mut exhausted,
            &digest,
            2,
            codefire_core::DecisionWorkPhase::Validation,
            "test HTTP digest destination reserve failure",
        )
        .unwrap_err();

        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert_eq!(
            exhausted.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            limit
        );
        digest.release(&mut exhausted).unwrap();
        exhausted
            .release(codefire_core::DecisionWorkDimension::RetainedBytes, filler)
            .unwrap();
        assert_eq!(
            exhausted.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );

        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "HTTP digest destination handoff",
        )
        .unwrap();
        let mut digest = source_object_set_digest_ordered(&records, &[0], &mut ledger).unwrap();
        let mut destination = HttpDigestDestinationOwnership::acquire(
            &mut ledger,
            &digest,
            2,
            codefire_core::DecisionWorkPhase::Validation,
            "test HTTP digest destination handoff",
        )
        .unwrap();
        let first = digest.as_str().to_string();
        let second = digest.as_str().to_string();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            source_bytes * 3
        );
        digest.release(&mut ledger).unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            source_bytes * 2
        );
        drop(first);
        drop(second);
        destination.release(&mut ledger).unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(ledger.open_permit_count(), 0);
    }

    #[test]
    fn merge_request_list_retained_limit_minus_exact_plus_one_is_balanced() {
        fn run_list(
            label: &str,
            case_index: u8,
            filler: u64,
        ) -> (
            Result<(u16, Value), CliError>,
            Option<codefire_core::DecisionWorkLedger>,
            PathBuf,
        ) {
            let storage_root = std::env::temp_dir().join(format!(
                "codefire-{label}-{}-{case_index:03}",
                std::process::id()
            ));
            let project_root = http_project_root(&storage_root, "org", "app");
            ensure_remote_project_layout(&project_root).unwrap();
            let objects = remote_dirs(&project_root).objects;
            let head = write_http_test_commit(&objects);
            write_json_atomic(
                &remote_branch_path(&project_root, "main"),
                &json!({"version": 1, "name": "main", "head": head}),
            )
            .unwrap();
            let records = collect_object_records(&objects, &head).unwrap();
            let object_set_digest = source_object_set_digest(&records).unwrap();
            for mr_id in ["MR-cache-a", "MR-cache-b"] {
                let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
                    mr_id,
                    source_project_root: &project_root,
                    target_project_root: &project_root,
                    source_branch: "main",
                    target_branch: "main",
                    source_head: &head,
                    target_head: &head,
                    source_object_set_digest: &object_set_digest,
                })
                .unwrap();
                let subject_digest = merge_request_subject_digest(&subject).unwrap();
                write_json_atomic(
                    &remote_merge_request_path(&project_root, mr_id),
                    &json!({
                        "version": 1,
                        "id": mr_id,
                        "source_url": "cf+http://local/org/app/main",
                        "source_head": head,
                        "target_url": "cf+http://local/org/app/main",
                        "target_head_at_request": head,
                        "status": "open",
                        "created_by": "test",
                        "created_at": "2026-07-20T00:00:00Z",
                        "reviews": [],
                        "subject": subject,
                        "subject_digest": subject_digest,
                    }),
                )
                .unwrap();
            }
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let deadline = HttpRequestDeadline::from_profile(&profile);
            let mut request_ledger = codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "merge-request list subject digest cache boundary",
            )
            .unwrap();
            if filler != 0 {
                request_ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Validation,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, filler),
                    )
                    .unwrap();
            }
            let mut ledger = Some(request_ledger);
            let mut prepared = None;
            let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);
            let result = handle_http_request_list(&storage_root, "org", "app", &mut context);
            (result, ledger, storage_root)
        }

        // Keep every run's ambient path shape identical. Snapshot ownership
        // includes path materialization, so changing the label length between
        // the baseline and a boundary probe would move the measured peak.
        const BOUNDARY_LABEL: &str = "merge-request-list-cache-boundary";
        let (baseline, baseline_ledger, baseline_root) = run_list(BOUNDARY_LABEL, 0, 0);
        assert_eq!(baseline.unwrap().0, 200);
        let baseline_peak = baseline_ledger.unwrap().snapshot().peak
            [&codefire_core::DecisionWorkDimension::RetainedBytes];
        let limit = codefire_core::DecisionWorkProfile::http_v1()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let exact_filler = limit.checked_sub(baseline_peak).unwrap();
        let _ = fs::remove_dir_all(baseline_root);

        for (case_index, suffix, adjustment, succeeds) in [
            (1_u8, "minus-one", -1_i64, true),
            (2_u8, "exact", 0_i64, true),
            (3_u8, "plus-one", 1_i64, false),
        ] {
            let filler = if adjustment < 0 {
                exact_filler - adjustment.unsigned_abs()
            } else {
                exact_filler + adjustment as u64
            };
            let (result, mut ledger, storage_root) = run_list(BOUNDARY_LABEL, case_index, filler);
            if succeeds {
                assert_eq!(result.unwrap().0, 200, "unexpected {suffix} result");
            } else {
                let error = result.unwrap_err();
                assert!(
                    matches!(error, CliError::ResourceBudget(_)),
                    "unexpected list boundary error: {error:?}"
                );
                let diagnostic = error.diagnostic();
                assert_eq!(diagnostic["dimension"], "retained_bytes");
                assert_eq!(diagnostic["observed"], limit + 1);
                assert_eq!(diagnostic["limit"], limit);
                assert_eq!(diagnostic["phase"], "snapshot_revalidation");
            }
            let ledger = ledger.as_mut().unwrap();
            let current =
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
            assert!(current >= filler);
            ledger
                .release(codefire_core::DecisionWorkDimension::RetainedBytes, current)
                .unwrap();
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                0
            );
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
                0
            );
            let _ = fs::remove_dir_all(storage_root);
        }
    }

    #[test]
    fn merge_request_list_preserves_per_record_sealed_then_subject_error_precedence() {
        let storage_root = test_storage_root("merge-request-list-error-precedence");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let records = collect_object_records(&objects, &head).unwrap();
        let object_set_digest = source_object_set_digest(&records).unwrap();
        let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
            mr_id: "MR-a-subject-corrupt",
            source_project_root: &project_root,
            target_project_root: &project_root,
            source_branch: "main",
            target_branch: "main",
            source_head: &head,
            target_head: &head,
            source_object_set_digest: &object_set_digest,
        })
        .unwrap();
        write_json_atomic(
            &remote_merge_request_path(&project_root, "MR-a-subject-corrupt"),
            &json!({
                "version": 1,
                "id": "MR-a-subject-corrupt",
                "source_url": "cf+http://local/org/app/main",
                "source_head": head,
                "target_url": "cf+http://local/org/app/main",
                "target_head_at_request": head,
                "status": "open",
                "reviews": [],
                "subject": subject,
                "subject_digest": "sha256:corrupt",
            }),
        )
        .unwrap();
        write_json_atomic(
            &remote_merge_request_path(&project_root, "MR-b-commit-corrupt"),
            &json!({
                "version": 1,
                "id": "MR-b-commit-corrupt",
                "source_url": "cf+http://local/org/app/main",
                "source_head": format!("CF-COMMIT-{}", "f".repeat(64)),
                "target_url": "cf+http://local/org/app/main",
                "target_head_at_request": head,
                "status": "open",
                "reviews": [],
            }),
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "merge-request list precedence",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let error =
            handle_http_request_list(&storage_root, "org", "app", &mut context).unwrap_err();

        assert!(matches!(
            error,
            CliError::InvalidRepository(message)
                if message == "merge request subject digest diverged"
        ));
        let snapshot = ledger.as_ref().unwrap().snapshot();
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn rejected_review_target_releases_initial_authority_owner() {
        let storage_root = test_storage_root("review-target-owner");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_merge_request_path(&project_root, "MR-wrong-target"),
            &json!({
                "version": 1,
                "id": "MR-wrong-target",
                "source_url": "cf+http://local/org/app/main",
                "source_head": head,
                "target_url": "cf+http://local/org/other/main",
                "target_head_at_request": head,
                "status": "open",
                "reviews": [],
            }),
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "review rejected target owner",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, _) = handle_http_request_review(
            &storage_root,
            "org",
            "app",
            "MR-wrong-target",
            &json!({"decision": "approve", "reviewer": "reviewer"}),
            &mut context,
        )
        .unwrap();

        assert_eq!(status, 400);
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn stale_unmaterialized_review_releases_locked_authority_owner() {
        let storage_root = test_storage_root("review-stale-owner");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let requested_head = write_http_test_commit(&objects);
        let requested_commit = codefire_store::read_object(&objects, &requested_head).unwrap();
        let current_head = codefire_store::store_object(
            &objects,
            "commit",
            codefire_store::commit_payload(
                vec![requested_head.clone()],
                requested_commit["roots"].as_object().unwrap().clone(),
                requested_commit["certificate"].as_object().unwrap().clone(),
            ),
        )
        .unwrap();
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": current_head}),
        )
        .unwrap();
        let mr_path = remote_merge_request_path(&project_root, "MR-stale");
        let mr = json!({
            "version": 1,
            "id": "MR-stale",
            "source_url": "cf+http://local/org/app/main",
            "source_head": requested_head,
            "target_url": "cf+http://local/org/app/main",
            "target_head_at_request": requested_head,
            "status": "open",
            "reviews": [],
        });
        fs::write(&mr_path, codefire_store::canonical_json(&mr).unwrap()).unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "review stale owner",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, _) = handle_http_request_review(
            &storage_root,
            "org",
            "app",
            "MR-stale",
            &json!({"decision": "approve", "reviewer": "reviewer"}),
            &mut context,
        )
        .unwrap();

        assert_eq!(status, 409);
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn unmaterialized_review_accounts_subject_history_before_mutation() {
        let storage_root = test_storage_root("review-subject-accounting");
        let (project_root, _) = write_unmaterialized_review_fixture(&storage_root, "MR-accounted");
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "review subject accounting",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, _) = handle_http_request_review(
            &storage_root,
            "org",
            "app",
            "MR-accounted",
            &json!({"decision": "approve", "reviewer": "reviewer"}),
            &mut context,
        )
        .unwrap();

        assert_eq!(status, 200);
        let materialized =
            read_json(&remote_merge_request_path(&project_root, "MR-accounted")).unwrap();
        assert!(materialized.get("subject").is_some());
        assert!(materialized.get("subject_digest").is_some());
        assert_eq!(
            materialized["legacy_review_history"]["reviews"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(materialized["reviews"].as_array().unwrap().len(), 1);
        assert_eq!(materialized["status"], "approved");
        let snapshot = ledger.as_ref().unwrap().snapshot();
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        assert!(
            snapshot.peak[&codefire_core::DecisionWorkDimension::TemporaryBytes] > 0,
            "subject/history planning must own its temporary materialization"
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn subject_materialization_deadline_releases_owners_without_mutation() {
        use std::cell::Cell;

        let storage_root = test_storage_root("review-subject-deadline");
        let (project_root, before) =
            write_unmaterialized_review_fixture(&storage_root, "MR-deadline");
        let mut mr = before.clone();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "review subject deadline",
        )
        .unwrap();
        let reserved = Cell::new(false);

        let error = materialize_merge_request_subject_for_roots_with_work_and_poll(
            &mut mr,
            MergeRequestSubjectRoots::new(&project_root, &project_root, "main", "main"),
            &mut ledger,
            || {
                if reserved.get() {
                    Err(CliError::RequestDeadlineExceeded {
                        phase: "review subject materialization",
                        limit_ms: 30_000,
                    })
                } else {
                    Ok(())
                }
            },
            |ledger, mr, plan, history| {
                let ownership =
                    reserve_http_merge_request_materialization(ledger, mr, plan, history)?;
                reserved.set(true);
                Ok(ownership)
            },
        )
        .unwrap_err();

        assert!(matches!(error, CliError::RequestDeadlineExceeded { .. }));
        assert_eq!(mr, before);
        let persisted =
            read_json(&remote_merge_request_path(&project_root, "MR-deadline")).unwrap();
        assert_eq!(persisted, before);
        let snapshot = ledger.snapshot();
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn subject_materialization_reserve_failure_releases_digest_and_record_owners() {
        let storage_root = test_storage_root("review-subject-reserve-failure");
        let (project_root, before) =
            write_unmaterialized_review_fixture(&storage_root, "MR-reserve-failure");
        let mut mr = before.clone();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "review subject reserve failure",
        )
        .unwrap();

        let error = materialize_merge_request_subject_for_roots_with_work_and_poll(
            &mut mr,
            MergeRequestSubjectRoots::new(&project_root, &project_root, "main", "main"),
            &mut ledger,
            || Ok(()),
            |_, _, _, _| Err(CliError::Usage("destination reserve failed".to_string())),
        )
        .unwrap_err();

        assert!(
            matches!(error, CliError::Usage(message) if message == "destination reserve failed")
        );
        assert_eq!(mr, before);
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        assert_eq!(ledger.open_permit_count(), 0);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn subject_materialization_subject_build_failure_releases_digest_and_record_owners() {
        let storage_root = test_storage_root("review-subject-build-failure");
        let (source_root, before) =
            write_unmaterialized_review_fixture(&storage_root, "MR-build-failure");
        let missing_target = storage_root.join("missing-target");
        let mut mr = before.clone();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "review subject build failure",
        )
        .unwrap();

        let error = materialize_merge_request_subject_for_roots_with_work_and_poll(
            &mut mr,
            MergeRequestSubjectRoots::new(&source_root, &missing_target, "main", "main"),
            &mut ledger,
            || Ok(()),
            |_, _, _, _| panic!("destination reservation must not run after subject failure"),
        )
        .unwrap_err();

        assert!(!error.to_string().is_empty());
        assert_eq!(mr, before);
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        assert_eq!(ledger.open_permit_count(), 0);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn subject_materialization_plan_has_no_object_store_effect() {
        let storage_root = test_storage_root("review-subject-pure-plan");
        let source_root = http_project_root(&storage_root, "org", "source");
        let target_root = http_project_root(&storage_root, "org", "target");
        ensure_remote_project_layout(&source_root).unwrap();
        ensure_remote_project_layout(&target_root).unwrap();
        let source_objects = remote_dirs(&source_root).objects;
        let target_objects = remote_dirs(&target_root).objects;
        let head = write_http_test_commit(&source_objects);
        assert!(codefire_store::object_record_path(&target_objects, &head).is_none());
        let mut mr = json!({
            "version": 1,
            "id": "MR-pure-plan",
            "source_url": "cf+http://local/org/source/main",
            "source_head": head,
            "target_url": "cf+http://local/org/target/main",
            "target_head_at_request": head,
            "status": "open",
            "reviews": [],
        });
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "review subject pure plan",
        )
        .unwrap();

        let planned = plan_merge_request_subject_materialization_for_roots_with_work_and_poll(
            &mut mr,
            MergeRequestSubjectRoots::new(&source_root, &target_root, "main", "main"),
            &mut ledger,
            || Ok(()),
            reserve_http_merge_request_materialization,
        )
        .unwrap();

        assert!(mr.get("subject").is_some());
        assert!(
            codefire_store::object_record_path(&target_objects, &head).is_none(),
            "planning must not publish object records"
        );
        let (accounted, object_records) = planned.into_parts();
        let (_, retained_bytes) = accounted.into_parts();
        object_records.unwrap().release(&mut ledger).unwrap();
        drop(mr);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn subject_materialization_publish_failure_restores_local_postimage_and_owners() {
        let storage_root = test_storage_root("review-subject-publish-rollback");
        let source_root = http_project_root(&storage_root, "org", "source");
        let target_root = http_project_root(&storage_root, "org", "target");
        ensure_remote_project_layout(&source_root).unwrap();
        ensure_remote_project_layout(&target_root).unwrap();
        let source_objects = remote_dirs(&source_root).objects;
        let target_objects = remote_dirs(&target_root).objects;
        let head = write_http_test_commit(&source_objects);
        for mut record in collect_object_records(&source_objects, &head).unwrap() {
            record["legacy_test_extension"] = Value::Bool(true);
            let document = codefire_store::object_record_document_from_value(record).unwrap();
            codefire_store::store_object_record_document(&target_objects, document).unwrap();
        }
        let before = json!({
            "version": 1,
            "id": "MR-publish-rollback",
            "source_url": "cf+http://local/org/source/main",
            "source_head": head,
            "target_url": "cf+http://local/org/target/main",
            "target_head_at_request": head,
            "status": "open",
            "reviews": [{
                "reviewer": "legacy",
                "decision": "approve",
                "comment": "unbound",
                "reviewed_at": "2026-07-19T00:00:00Z"
            }],
        });
        let mut mr = before.clone();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "review subject publish rollback",
        )
        .unwrap();

        let error = materialize_merge_request_subject_for_roots_with_work_and_poll(
            &mut mr,
            MergeRequestSubjectRoots::new(&source_root, &target_root, "main", "main"),
            &mut ledger,
            || Ok(()),
            reserve_http_merge_request_materialization,
        )
        .unwrap_err();

        assert!(
            matches!(
                &error,
                CliError::Store(error)
                    if matches!(
                        error.as_ref(),
                        codefire_store::StoreError::InvalidSealedCommit(message)
                            if message.starts_with("immutable object extension fields diverge at ")
                    )
            ),
            "unexpected publication failure: {error:?}"
        );
        assert_eq!(mr, before);
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn review_retained_limit_minus_exact_plus_one_is_effect_safe() {
        fn run_review(label: &str, filler: u64) -> ReviewBoundaryRunResult {
            let storage_root = test_storage_root(label);
            let (project_root, before) =
                write_unmaterialized_review_fixture(&storage_root, "MR-boundary");
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let deadline = HttpRequestDeadline::from_profile(&profile);
            let mut request_ledger = codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "review retained boundary",
            )
            .unwrap();
            if filler != 0 {
                request_ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Validation,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, filler),
                    )
                    .unwrap();
            }
            let mut ledger = Some(request_ledger);
            let mut prepared = None;
            let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);
            let result = handle_http_request_review(
                &storage_root,
                "org",
                "app",
                "MR-boundary",
                &json!({"decision": "approve", "reviewer": "reviewer"}),
                &mut context,
            );
            (result, ledger, project_root, before)
        }

        // Keep every fixture root label the same length: the absolute root is
        // part of retained-work accounting, so varying labels would shift the
        // boundary that this test is intended to isolate.
        let (baseline, baseline_ledger, baseline_root, _) =
            run_review("review-boundary-baseline-", 0);
        assert_eq!(baseline.unwrap().0, 200);
        let baseline_peak = baseline_ledger.unwrap().snapshot().peak
            [&codefire_core::DecisionWorkDimension::RetainedBytes];
        let limit = codefire_core::DecisionWorkProfile::http_v1()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let exact_filler = limit.checked_sub(baseline_peak).unwrap();
        let _ = fs::remove_dir_all(
            baseline_root
                .parent()
                .and_then(Path::parent)
                .expect("test project has a storage root"),
        );

        for (suffix, fixture_suffix, adjustment, succeeds) in [
            ("minus-one", "minus-one", -1_i64, true),
            ("exact", "exact-pad", 0_i64, true),
            ("plus-one", "plus-one-", 1_i64, false),
        ] {
            let filler = if adjustment < 0 {
                exact_filler - adjustment.unsigned_abs()
            } else {
                exact_filler + adjustment as u64
            };
            let (result, mut ledger, project_root, before) =
                run_review(&format!("review-boundary-{fixture_suffix}"), filler);
            if succeeds {
                assert_eq!(result.unwrap().0, 200, "unexpected {suffix} result");
            } else {
                let error = result.unwrap_err();
                assert!(matches!(error, CliError::ResourceBudget(_)));
                let diagnostic = error.diagnostic();
                assert_eq!(diagnostic["dimension"], "retained_bytes");
                assert_eq!(diagnostic["observed"], limit + 1);
                assert_eq!(diagnostic["limit"], limit);
                assert_eq!(
                    read_json(&remote_merge_request_path(&project_root, "MR-boundary")).unwrap(),
                    before,
                    "limit+1 must not publish subject, history, or review authority"
                );
            }
            let ledger = ledger.as_mut().unwrap();
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                filler
            );
            ledger
                .release(codefire_core::DecisionWorkDimension::RetainedBytes, filler)
                .unwrap();
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                0
            );
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
                0
            );
            let storage_root = project_root
                .parent()
                .and_then(Path::parent)
                .expect("test project has a storage root");
            let _ = fs::remove_dir_all(storage_root);
        }
    }

    #[test]
    fn merge_request_list_corrupt_second_record_releases_every_owner() {
        let storage_root = test_storage_root("merge-request-list-corrupt-second");
        let (project_root, _) = write_unmaterialized_review_fixture(&storage_root, "MR-a-valid");
        fs::write(
            remote_merge_request_path(&project_root, "MR-z-corrupt"),
            b"{not-json",
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "merge-request list corrupt second",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let error =
            handle_http_request_list(&storage_root, "org", "app", &mut context).unwrap_err();

        assert!(matches!(error, CliError::Json(_)));
        let snapshot = ledger.as_ref().unwrap().snapshot();
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn apply_owns_branch_and_generation_postimages_until_publication() {
        let storage_root = test_storage_root("apply-postimage-accounting");
        let project_root = write_approved_apply_fixture(&storage_root, "MR-apply");
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "apply postimage accounting",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) = handle_http_request_apply(
            &storage_root,
            "org",
            "app",
            "MR-apply",
            &json!({"actor": "reviewer"}),
            &mut context,
        )
        .unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload, Value::Null);
        assert_eq!(
            release_prepared_apply_response(&mut ledger, &mut prepared)["id"],
            "MR-apply"
        );
        assert_eq!(
            read_json(&project_root.join("gc_state.json")).unwrap()["current_generation"],
            1
        );
        assert_eq!(
            read_json(&remote_merge_request_path(&project_root, "MR-apply")).unwrap()["status"],
            "applied"
        );
        let snapshot = ledger.as_ref().unwrap().snapshot();
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn full_wire_apply_preserves_authority_at_parser_store_handoff_boundary() {
        let storage_root = test_storage_root("apply-parser-store-handoff");
        let project_root = write_approved_apply_fixture(&storage_root, "MR-wire-apply");
        require_apply_request_signatures(&project_root);
        let branch_path = remote_branch_path(&project_root, "main");
        let mr_path = remote_merge_request_path(&project_root, "MR-wire-apply");
        let source_head = read_json(&mr_path).unwrap()["source_head"]
            .as_str()
            .unwrap()
            .to_string();
        let mut request = json!({
            "actor": "reviewer",
            "idempotency_key": format!("wire-{}", "i".repeat(1024 - "wire-".len()))
        });
        sign_apply_request("MR-wire-apply", &mut request);
        let body = serde_json::to_vec(&request).unwrap();
        let apply_path = format!(
            "/v1/projects/org/app/merge-requests/MR-wire-apply/apply?routing={}",
            "q".repeat(4 * 1024)
        );
        let mut wire = format!(
            "POST {apply_path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        wire.extend_from_slice(&body);
        let request_len = wire.len();
        let mut stream = Cursor::new(wire);
        let _ = take_http_mutation_store_binding();

        handle_http_stream(&storage_root, &mut stream).unwrap();

        let (binding_expected, binding_current, binding_structural) =
            take_http_mutation_store_binding();
        assert_eq!(binding_current, binding_expected);
        assert!(binding_structural);
        assert!(binding_expected > u64::try_from(body.len()).unwrap());

        let wire = stream.into_inner();
        let response = std::str::from_utf8(&wire[request_len..]).unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"id\":\"MR-wire-apply\""));
        assert_eq!(read_json(&branch_path).unwrap()["head"], source_head);
        assert_eq!(read_json(&mr_path).unwrap()["status"], "applied");
        assert_eq!(
            read_json(&project_root.join("gc_state.json")).unwrap()["current_generation"],
            1
        );
        assert!(project_root.join("request_nonce_cache.json").exists());
        assert_eq!(
            fs::read_dir(project_root.join("idempotency/v2/remote_request_apply"))
                .unwrap()
                .count(),
            1
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn full_wire_upload_preserves_authentication_and_non_fast_forward_precedence() {
        fn send(storage_root: &Path, request: &Value) -> String {
            let body = serde_json::to_vec(request).unwrap();
            let mut wire = format!(
                "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .into_bytes();
            wire.extend_from_slice(&body);
            let request_len = wire.len();
            let mut stream = Cursor::new(wire);
            handle_http_stream(storage_root, &mut stream).unwrap();
            String::from_utf8(stream.into_inner()[request_len..].to_vec()).unwrap()
        }

        let storage_root = test_storage_root("full-wire-upload-precedence");
        let (project_root, mut signed) =
            upload_request_fixture(&storage_root, "full-wire-upload-precedence");
        let objects = remote_dirs(&project_root).objects;
        let original = write_http_test_commit(&objects);
        let mut unrelated_payload = codefire_store::read_object_record(&objects, &original)
            .unwrap()
            .payload;
        unrelated_payload["message"] = Value::String("unrelated branch head".to_string());
        let unrelated =
            codefire_store::store_object(&objects, "commit", unrelated_payload).unwrap();
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": unrelated}),
        )
        .unwrap();
        write_upload_request_signature_policy(&project_root, false);
        sign_upload_request(&mut signed);

        let mut invalid = signed.clone();
        invalid["request_signature"]["signature"] = Value::String("0".repeat(64));
        let response = send(&storage_root, &invalid);
        assert!(response.starts_with("HTTP/1.1 400 Bad Request"));
        assert!(response.contains("signature validation failed"));
        assert!(!project_root.join("request_nonce_cache.json").exists());

        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "upload",
            Some("main"),
            None,
        );
        super::super::signatures::validate_remote_request_signature(
            &project_root,
            "upload",
            "uploader",
            &target,
            &signed,
            signed.get("request_signature"),
        )
        .unwrap()
        .publish(&project_root)
        .unwrap();
        let nonce_before = fs::read(project_root.join("request_nonce_cache.json")).unwrap();
        let response = send(&storage_root, &signed);
        assert!(response.starts_with("HTTP/1.1 400 Bad Request"));
        assert!(response.contains("nonce has already been used"));

        sign_upload_request(&mut signed);
        let response = send(&storage_root, &signed);
        assert!(response.starts_with("HTTP/1.1 409 Conflict"));
        assert!(
            response.contains("remote branch is not an ancestor of local branch head"),
            "unexpected conflict response: {response}"
        );
        assert_eq!(
            fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            nonce_before,
            "semantic rejection must cancel the fresh full-wire nonce owner"
        );
        assert!(!project_root.join("gc_state.json").exists());
        assert!(!project_root.join("idempotency/v2/remote_upload").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn full_wire_upload_transfers_intent_into_store_publication() {
        let storage_root = test_storage_root("upload-parser-store-handoff");
        let (project_root, mut request) =
            upload_request_fixture(&storage_root, "wire-upload-success");
        let large_local_branch = format!("local-{}", "b".repeat(1024 * 1024));
        request["branch"] = Value::String(large_local_branch.clone());
        write_upload_request_signature_policy(&project_root, false);
        sign_upload_request(&mut request);
        let head = request["head"].as_str().unwrap().to_string();
        let object_ids = request["objects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["object_id"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        let body = serde_json::to_vec(&request).unwrap();
        let upload_path = format!(
            "/v1/projects/org/app/branches/main/upload?routing={}",
            "q".repeat(4 * 1024)
        );
        let mut wire = format!(
            "POST {upload_path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        wire.extend_from_slice(&body);
        let request_len = wire.len();
        let mut stream = Cursor::new(wire);
        let _ = take_http_mutation_store_binding();

        handle_http_stream(&storage_root, &mut stream).unwrap();

        let (binding_expected, binding_current, binding_structural) =
            take_http_mutation_store_binding();
        assert_eq!(binding_current, binding_expected);
        assert!(binding_structural);
        assert!(binding_expected > u64::try_from(body.len()).unwrap());

        let wire = stream.into_inner();
        let response = std::str::from_utf8(&wire[request_len..]).unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains(&format!("\"head\":\"{head}\"")));
        assert_eq!(
            read_json(&remote_branch_path(&project_root, "main")).unwrap()["head"],
            head
        );
        assert_eq!(
            read_json(&remote_branch_path(&project_root, "main")).unwrap()["uploaded_from"],
            large_local_branch
        );
        for object_id in object_ids {
            let path =
                codefire_store::object_record_path(&remote_dirs(&project_root).objects, &object_id)
                    .expect("uploaded object ID must map to its canonical path");
            assert!(
                path.is_file(),
                "uploaded object must be durable: {object_id}"
            );
            let record = read_json(&path).unwrap();
            assert_eq!(record["object_id"], object_id);
        }
        assert!(project_root.join("request_nonce_cache.json").exists());
        assert_eq!(
            fs::read_dir(project_root.join("idempotency/v2/remote_upload"))
                .unwrap()
                .count(),
            1
        );
        let nonce_before = fs::read(project_root.join("request_nonce_cache.json")).unwrap();
        sign_upload_request(&mut request);
        let replay_body = serde_json::to_vec(&request).unwrap();
        let mut replay_wire = format!(
            "POST {upload_path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            replay_body.len()
        )
        .into_bytes();
        replay_wire.extend_from_slice(&replay_body);
        let replay_request_len = replay_wire.len();
        let mut replay_stream = Cursor::new(replay_wire);

        handle_http_stream(&storage_root, &mut replay_stream).unwrap();

        let replay_wire = replay_stream.into_inner();
        let replay_response = std::str::from_utf8(&replay_wire[replay_request_len..]).unwrap();
        assert!(replay_response.starts_with("HTTP/1.1 200 OK"));
        assert!(replay_response.contains(&format!("\"head\":\"{head}\"")));
        assert_ne!(
            fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            nonce_before,
            "completed production replay must publish its fresh nonce"
        );
        assert_eq!(
            fs::read_dir(project_root.join("idempotency/v2/remote_upload"))
                .unwrap()
                .count(),
            1,
            "completed production replay must reuse its receipt"
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn production_mutation_routing_accepts_exact_and_rejects_query_plus_one_pre_effect() {
        const RETAINED_LIMIT: u64 = 32 * 1024 * 1024;

        fn fixture(label: &str) -> (PathBuf, PathBuf, String, Vec<u8>, u64) {
            let storage_root = test_storage_root(label);
            let (project_root, mut request) =
                upload_request_fixture(&storage_root, "routing-boundary");
            write_upload_request_signature_policy(&project_root, false);
            sign_upload_request(&mut request);
            let path = format!(
                "/v1/projects/org/app/branches/main/upload?routing={}",
                "q".repeat(8 * 1024)
            );
            let body = serde_json::to_vec(&request).unwrap();
            let source_routing = logical_string_owner_bytes("POST").unwrap()
                + logical_string_owner_bytes(&path).unwrap();
            let filler_bytes = RETAINED_LIMIT - source_routing - u64::try_from(body.len()).unwrap();
            (storage_root, project_root, path, body, filler_bytes)
        }

        let (storage_root, project_root, path, body, filler_bytes) =
            fixture("mutation-routing-exact");
        let source_routing = logical_string_owner_bytes("POST").unwrap()
            + logical_string_owner_bytes(&path).unwrap();
        assert_eq!(
            u64::try_from(body.len()).unwrap() + source_routing + filler_bytes,
            RETAINED_LIMIT
        );
        let mut wire = format!(
            "POST {path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        assert!(wire.len() <= MAX_HTTP_HEADER_BYTES);
        wire.extend_from_slice(&body);
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut exact_ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "production mutation routing exact",
        )
        .unwrap();
        exact_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestHeaders,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    filler_bytes,
                ),
            )
            .unwrap();
        let filler = "f".repeat(
            usize::try_from(filler_bytes - codefire_core::LOGICAL_FRAME_BASE_BYTES).unwrap(),
        );
        assert_eq!(logical_string_owner_bytes(&filler).unwrap(), filler_bytes);
        let mut parsed =
            read_http_request_with_ledger(&mut Cursor::new(wire), &mut exact_ledger, &deadline)
                .unwrap();
        drop(filler);
        exact_ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                filler_bytes,
            )
            .unwrap();
        let mut exact_ledger = Some(exact_ledger);
        let mut exact_prepared = None;
        let response = dispatch_http_with_ledger(
            &storage_root,
            &mut parsed,
            &mut exact_ledger,
            &mut exact_prepared,
            &deadline,
        )
        .unwrap();
        assert_eq!(response.status, 200);
        drop(response.payload);
        parsed.release_body(exact_ledger.as_mut().unwrap()).unwrap();
        parsed
            .release_routing(exact_ledger.as_mut().unwrap())
            .unwrap();
        release_prepared_http_response(
            exact_ledger.as_mut().unwrap(),
            exact_prepared.take().unwrap(),
            "release exact routing response",
        )
        .unwrap();
        assert!(exact_ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(remote_branch_path(&project_root, "main").exists());
        let _ = fs::remove_dir_all(storage_root);

        let (storage_root, project_root, path, body, filler_bytes) =
            fixture("mutation-routing-plus-one");
        let plus_one_path = format!("{path}q");
        let source_routing = logical_string_owner_bytes("POST").unwrap()
            + logical_string_owner_bytes(&plus_one_path).unwrap();
        assert_eq!(
            u64::try_from(body.len()).unwrap() + source_routing + filler_bytes,
            RETAINED_LIMIT + 1
        );
        let mut wire = format!(
            "POST {plus_one_path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        assert!(wire.len() <= MAX_HTTP_HEADER_BYTES);
        wire.extend_from_slice(&body);
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "production mutation routing plus one",
        )
        .unwrap();
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestHeaders,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    filler_bytes,
                ),
            )
            .unwrap();
        let filler = "f".repeat(
            usize::try_from(filler_bytes - codefire_core::LOGICAL_FRAME_BASE_BYTES).unwrap(),
        );
        let error = read_http_request_with_ledger(&mut Cursor::new(wire), &mut ledger, &deadline)
            .unwrap_err();
        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert_eq!(http_error_response(&error).0, 413);
        drop(filler);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                filler_bytes,
            )
            .unwrap();
        terminate_http_request(&mut ledger, http_request_terminal_reason(&error)).unwrap();
        let snapshot = ledger.snapshot();
        assert!(snapshot.current.values().all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
        assert!(!remote_branch_path(&project_root, "main").exists());
        assert!(!project_root.join("gc_state.json").exists());
        assert!(!project_root.join("request_nonce_cache.json").exists());
        assert!(!project_root.join("idempotency/v2/remote_upload").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn production_upload_derived_owner_overflow_is_413_and_pre_effect() {
        let storage_root = test_storage_root("upload-derived-owner-overflow");
        let (project_root, mut request) =
            upload_request_fixture(&storage_root, "derived-owner-overflow");
        request["branch"] = Value::String("b".repeat(12 * 1024 * 1024));
        write_upload_request_signature_policy(&project_root, false);
        sign_upload_request(&mut request);
        let body = serde_json::to_vec(&request).unwrap();
        assert!(body.len() < 16 * 1024 * 1024);
        let mut wire = format!(
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        wire.extend_from_slice(&body);
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "upload derived owner overflow",
        )
        .unwrap();
        let mut parsed =
            read_http_request_with_ledger(&mut Cursor::new(wire), &mut ledger, &deadline).unwrap();
        let mut ledger = Some(ledger);
        let mut prepared = None;

        let error = match dispatch_http_with_ledger(
            &storage_root,
            &mut parsed,
            &mut ledger,
            &mut prepared,
            &deadline,
        ) {
            Err(error) => error,
            Ok(_) => panic!("derived-owner overflow must fail before dispatch success"),
        };

        let (status, payload) = http_error_response(&error);
        assert_eq!(status, 413);
        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert!(payload["error"]
            .as_str()
            .unwrap()
            .contains("resource budget exceeded"));
        let snapshot = ledger.as_ref().unwrap().snapshot();
        assert!(snapshot.current.values().all(|amount| *amount == 0));
        assert!(!ledger.as_ref().unwrap().has_open_permits());
        assert!(prepared.is_none());
        assert!(!remote_branch_path(&project_root, "main").exists());
        assert!(!project_root.join("gc_state.json").exists());
        assert!(!project_root.join("request_nonce_cache.json").exists());
        assert!(!project_root.join("idempotency/v2/remote_upload").exists());
        for entry in fs::read_dir(remote_dirs(&project_root).objects).unwrap() {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_dir());
            assert_eq!(fs::read_dir(entry.path()).unwrap().count(), 0);
        }
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn apply_authenticates_missing_invalid_and_reused_nonce_before_non_fast_forward_rejection() {
        let storage_root = test_storage_root("apply-auth-before-non-ff");
        let project_root = write_approved_apply_fixture(&storage_root, "MR-auth");
        make_apply_fixture_non_fast_forward(&project_root, "MR-auth");
        require_apply_request_signatures(&project_root);
        let branch_path = remote_branch_path(&project_root, "main");
        let mr_path = remote_merge_request_path(&project_root, "MR-auth");
        let branch_before = fs::read(&branch_path).unwrap();
        let mr_before = fs::read(&mr_path).unwrap();

        let mut signed = json!({
            "actor": "reviewer",
            "idempotency_key": "apply-auth-precedence"
        });
        sign_apply_request("MR-auth", &mut signed);
        let mut invalid = signed.clone();
        invalid["request_signature"]["signature"] = Value::String("0".repeat(64));
        for (label, request) in [
            ("missing", json!({"actor": "reviewer"})),
            ("invalid", invalid),
        ] {
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let deadline = HttpRequestDeadline::from_profile(&profile);
            let mut ledger = Some(
                codefire_core::DecisionWorkLedger::new(
                    profile,
                    codefire_core::DecisionWorkCommandKind::HttpRequest,
                    format!("apply {label} signature precedence"),
                )
                .unwrap(),
            );
            let mut prepared = None;
            let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

            let error = handle_http_request_apply(
                &storage_root,
                "org",
                "app",
                "MR-auth",
                &request,
                &mut context,
            )
            .unwrap_err();

            assert!(matches!(error, CliError::AuthenticationOrSignature(_)));
            assert!(prepared.is_none());
            assert_eq!(
                ledger
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                    .copied()
                    .unwrap_or_default(),
                0
            );
            assert_eq!(
                ledger
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied()
                    .unwrap_or_default(),
                0
            );
            assert_eq!(fs::read(&branch_path).unwrap(), branch_before);
            assert_eq!(fs::read(&mr_path).unwrap(), mr_before);
            assert!(!project_root.join("gc_state.json").exists());
            assert!(!project_root.join("request_nonce_cache.json").exists());
        }

        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "apply",
            None,
            Some("MR-auth"),
        );
        super::super::signatures::validate_remote_request_signature(
            &project_root,
            "apply",
            "reviewer",
            &target,
            &signed,
            signed.get("request_signature"),
        )
        .unwrap()
        .publish(&project_root)
        .unwrap();
        let nonce_before = fs::read(project_root.join("request_nonce_cache.json")).unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "apply reused nonce precedence",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let error = handle_http_request_apply(
            &storage_root,
            "org",
            "app",
            "MR-auth",
            &signed,
            &mut context,
        )
        .unwrap_err();

        assert!(matches!(error, CliError::AuthenticationOrSignature(_)));
        assert!(error.to_string().contains("nonce has already been used"));
        assert!(prepared.is_none());
        assert_eq!(
            ledger
                .as_ref()
                .unwrap()
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert_eq!(
            ledger
                .as_ref()
                .unwrap()
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert_eq!(fs::read(&branch_path).unwrap(), branch_before);
        assert_eq!(fs::read(&mr_path).unwrap(), mr_before);
        assert_eq!(
            fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            nonce_before
        );
        assert!(!project_root.join("gc_state.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn apply_invalid_commit_signature_releases_policy_query_and_nonce_owners() {
        let storage_root = test_storage_root("apply-invalid-commit-signature");
        let project_root = write_approved_apply_fixture(&storage_root, "MR-bad-signature");
        make_apply_fixture_invalid_commit_signature(&project_root, "MR-bad-signature");
        let branch_path = remote_branch_path(&project_root, "main");
        let mr_path = remote_merge_request_path(&project_root, "MR-bad-signature");
        let branch_before = fs::read(&branch_path).unwrap();
        let mr_before = fs::read(&mr_path).unwrap();
        let mut request = json!({
            "actor": "reviewer",
            "idempotency_key": "invalid-commit-signature"
        });
        sign_apply_request("MR-bad-signature", &mut request);
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "apply invalid commit signature",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let error = handle_http_request_apply(
            &storage_root,
            "org",
            "app",
            "MR-bad-signature",
            &request,
            &mut context,
        )
        .unwrap_err();

        assert!(matches!(error, CliError::AuthenticationOrSignature(_)));
        assert!(error.to_string().contains("invalid signature"));
        assert!(prepared.is_none());
        let snapshot = ledger.as_ref().unwrap().snapshot();
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                snapshot
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0
            );
        }
        assert!(
            snapshot
                .counters
                .get(&codefire_core::DecisionWorkCounter::BytesHashed)
                .copied()
                .unwrap_or_default()
                > 0
        );
        assert_eq!(fs::read(&branch_path).unwrap(), branch_before);
        assert_eq!(fs::read(&mr_path).unwrap(), mr_before);
        assert!(!project_root.join("gc_state.json").exists());
        assert!(!project_root.join("request_nonce_cache.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn apply_replays_completed_receipt_with_fresh_nonce_without_rebuilding_authority_postimages() {
        let storage_root = test_storage_root("apply-fresh-nonce-replay");
        let project_root = write_approved_apply_fixture(&storage_root, "MR-replay");
        require_apply_request_signatures(&project_root);
        let mut request = json!({
            "actor": "reviewer",
            "idempotency_key": "stable-apply-request"
        });

        sign_apply_request("MR-replay", &mut request);
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "apply initial signed publication",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);
        let (status, payload) = handle_http_request_apply(
            &storage_root,
            "org",
            "app",
            "MR-replay",
            &request,
            &mut context,
        )
        .unwrap();
        assert_eq!((status, payload), (200, Value::Null));
        let first_response = release_prepared_apply_response(&mut ledger, &mut prepared);
        assert_eq!(first_response["id"], "MR-replay");

        let branch_path = remote_branch_path(&project_root, "main");
        let mr_path = remote_merge_request_path(&project_root, "MR-replay");
        let generation_path = project_root.join("gc_state.json");
        let receipt_path = fs::read_dir(project_root.join("idempotency/v2/remote_request_apply"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let branch_after_first = fs::read(&branch_path).unwrap();
        let mr_after_first = fs::read(&mr_path).unwrap();
        let generation_after_first = fs::read(&generation_path).unwrap();
        let receipt_after_first = fs::read(&receipt_path).unwrap();
        let nonce_after_first = fs::read(project_root.join("request_nonce_cache.json")).unwrap();

        // A completed receipt must win before the Fresh-only commit policy
        // and authority builder. This policy would reject the unsigned source
        // commit if the replay incorrectly re-entered Fresh planning.
        let mut changed_policy = read_json(&project_root.join("server_policy.json")).unwrap();
        changed_policy["commit_signatures"] = json!({
            "required": true,
            "keys": {"reviewer": "commit-key"}
        });
        write_json_atomic(&project_root.join("server_policy.json"), &changed_policy).unwrap();

        sign_apply_request("MR-replay", &mut request);
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut replay_ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "apply signed replay publication",
            )
            .unwrap(),
        );
        let mut replay_prepared = None;
        let mut replay_context =
            http_route_context_for_test(&mut replay_ledger, &mut replay_prepared, &deadline);
        let (status, payload) = handle_http_request_apply(
            &storage_root,
            "org",
            "app",
            "MR-replay",
            &request,
            &mut replay_context,
        )
        .unwrap();
        assert_eq!((status, payload), (200, Value::Null));
        let replay_response =
            release_prepared_apply_response(&mut replay_ledger, &mut replay_prepared);

        assert_eq!(replay_response, first_response);
        assert_eq!(fs::read(&branch_path).unwrap(), branch_after_first);
        assert_eq!(fs::read(&mr_path).unwrap(), mr_after_first);
        assert_eq!(fs::read(&generation_path).unwrap(), generation_after_first);
        assert_eq!(fs::read(&receipt_path).unwrap(), receipt_after_first);
        assert_ne!(
            fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            nonce_after_first
        );
        for ledger in [&ledger, &replay_ledger] {
            let snapshot = ledger.as_ref().unwrap().snapshot();
            assert_eq!(
                snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                0
            );
            assert_eq!(
                snapshot.current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
                0
            );
        }
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn full_http_doctor_reuses_one_ledger_and_releases_scan_ownership() {
        let storage_root = test_storage_root("doctor-accounting");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "doctor accounting",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) =
            handle_http_doctor(&storage_root, "org", "app", false, &mut context).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["ok"], true);
        assert_eq!(payload["branch_count"], 1);
        let expected = codefire_store::canonical_json(&payload).unwrap().len() as u64;
        assert_eq!(
            ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            expected
        );
        assert_eq!(
            ledger
                .as_ref()
                .unwrap()
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        let snapshot = ledger.as_ref().unwrap().snapshot();
        let projection_peak = snapshot.peak[&codefire_core::DecisionWorkDimension::RetainedBytes]
            .checked_add(snapshot.peak[&codefire_core::DecisionWorkDimension::TemporaryBytes])
            .unwrap();
        assert!(
            projection_peak >= expected * 2,
            "report and projected Value must have a measured simultaneous 2L ownership window"
        );
        assert!(
            ledger.as_ref().unwrap().snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::ObjectRecords]
                >= 9
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_doctor_report_growth_exhaustion_is_a_typed_413() {
        let storage_root = test_storage_root("doctor-report-budget");
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let retained_limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut request_ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "doctor report budget",
        )
        .unwrap();
        request_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Encoding,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_limit - 1,
                ),
            )
            .unwrap();
        let mut ledger = Some(request_ledger);
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let error =
            handle_http_doctor(&storage_root, "org", "app", true, &mut context).unwrap_err();

        assert!(
            matches!(error, CliError::ResourceBudget(_)),
            "unexpected doctor report-budget error: {error:?}"
        );
        assert_eq!(http_error_response(&error).0, 413);
        assert_eq!(error.diagnostic()["kind"], "resource_budget_exceeded");
        assert_eq!(error.diagnostic()["dimension"], "retained_bytes");
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_doctor_root_validation_budget_exhaustion_is_not_a_corruption_report() {
        let storage_root = test_storage_root("doctor-root-budget");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": head}),
        )
        .unwrap();
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let graph_limit = profile
            .limit(codefire_core::DecisionWorkDimension::GraphNodes)
            .unwrap()
            .limit;
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut request_ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "doctor root budget",
        )
        .unwrap();
        request_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::GraphDiscovery,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::GraphNodes,
                    graph_limit,
                ),
            )
            .unwrap();
        let mut ledger = Some(request_ledger);
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let error =
            handle_http_doctor(&storage_root, "org", "app", false, &mut context).unwrap_err();

        assert!(
            matches!(error, CliError::ResourceBudget(_)),
            "unexpected doctor root-budget error: {error:?}"
        );
        assert_eq!(http_error_response(&error).0, 413);
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["kind"], "resource_budget_exceeded");
        assert_eq!(diagnostic["exit_code"], 60);
        assert_eq!(diagnostic["profile"], "codefire.decision-work.v1");
        assert_eq!(diagnostic["dimension"], "graph_nodes");
        assert_eq!(diagnostic["observed"], graph_limit + 1);
        assert_eq!(diagnostic["limit"], graph_limit);
        assert_eq!(diagnostic["phase"], "graph_discovery");
        assert_eq!(diagnostic["command"], "doctor root budget");
        assert_eq!(diagnostic["continuation_supported"], false);
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0,
                "typed doctor budget failure leaked {dimension:?}"
            );
        }
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn remote_doctor_cancellation_releases_every_local_owner() {
        use std::cell::Cell;

        let storage_root = test_storage_root("doctor-cancel-cleanup");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let head = write_http_test_commit(&objects);
        for branch in ["alpha", "beta", "gamma"] {
            write_json_atomic(
                &remote_branch_path(&project_root, branch),
                &json!({"version": 1, "name": branch, "head": head}),
            )
            .unwrap();
        }

        let total_polls = Cell::new(0_u64);
        let mut baseline_ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::http_v1(),
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "doctor cancellation baseline",
            )
            .unwrap(),
        );
        diagnose_remote_project_with_work_and_poll(
            &project_root,
            "cf+http://local/org/app",
            "http",
            false,
            &mut baseline_ledger,
            || {
                total_polls.set(total_polls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        let total_polls = total_polls.get();
        assert!(total_polls > 8);

        let mut saw_cancel_after_record_work = false;
        for cutoff in [2, total_polls / 4, total_polls / 2, total_polls - 1] {
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let mut ledger = Some(
                codefire_core::DecisionWorkLedger::new(
                    profile,
                    codefire_core::DecisionWorkCommandKind::HttpRequest,
                    "doctor cancellation cleanup",
                )
                .unwrap(),
            );
            let polls = Cell::new(0_u64);
            let result = diagnose_remote_project_with_work_and_poll(
                &project_root,
                "cf+http://local/org/app",
                "http",
                false,
                &mut ledger,
                || {
                    polls.set(polls.get() + 1);
                    if polls.get() == cutoff {
                        Err(CliError::RequestDeadlineExceeded {
                            phase: "doctor cancellation test",
                            limit_ms: 30_000,
                        })
                    } else {
                        Ok(())
                    }
                },
            );
            match result {
                Err(CliError::RequestDeadlineExceeded { .. }) => {
                    let snapshot = ledger.as_ref().unwrap().snapshot();
                    for dimension in [
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                    ] {
                        assert_eq!(
                            snapshot
                                .current
                                .get(&dimension)
                                .copied()
                                .unwrap_or_default(),
                            0,
                            "doctor leaked {dimension:?} at cancellation poll {cutoff}"
                        );
                    }
                    saw_cancel_after_record_work |= snapshot
                        .counters
                        .get(&codefire_core::DecisionWorkCounter::RecordReads)
                        .copied()
                        .unwrap_or_default()
                        > 0;
                }
                Ok(_) => panic!("doctor unexpectedly completed at cancellation poll {cutoff}"),
                Err(error) => panic!("unexpected doctor cancellation result: {error:?}"),
            }
        }
        assert!(saw_cancel_after_record_work);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn doctor_missing_root_between_shared_dag_roots_does_not_reset_validation_cache() {
        let storage_root = test_storage_root("doctor-missing-root-shared-cache");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let base = write_http_test_commit(&objects);
        let base_record = codefire_store::read_object_record(&objects, &base).unwrap();
        let child = codefire_store::store_object(
            &objects,
            "commit",
            codefire_store::commit_payload(
                vec![base.clone()],
                base_record.payload["roots"].as_object().unwrap().clone(),
                base_record.payload["certificate"]
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .unwrap();
        for (branch, head) in [("base", &base), ("child", &child)] {
            write_json_atomic(
                &remote_branch_path(&project_root, branch),
                &json!({"version": 1, "name": branch, "head": head}),
            )
            .unwrap();
        }

        let run = || {
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let deadline = HttpRequestDeadline::from_profile(&profile);
            let mut ledger = Some(
                codefire_core::DecisionWorkLedger::new(
                    profile,
                    codefire_core::DecisionWorkCommandKind::HttpRequest,
                    "doctor shared cache",
                )
                .unwrap(),
            );
            let mut prepared = None;
            let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);
            let response = handle_http_doctor(&storage_root, "org", "app", false, &mut context);
            let object_records = ledger.as_ref().unwrap().snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::ObjectRecords];
            (response, object_records)
        };

        let (baseline, baseline_work) = run();
        assert_eq!(baseline.unwrap().1["invalid_sealed_commit_references"], 0);
        let missing = commit_id_between(&base, &child);
        let (low, high) = if base < child {
            (&base, &child)
        } else {
            (&child, &base)
        };
        assert!(low < &missing && &missing < high);
        write_json_atomic(
            &remote_branch_path(&project_root, "missing"),
            &json!({"version": 1, "name": "missing", "head": missing}),
        )
        .unwrap();

        let (diagnosed, diagnosed_work) = run();
        let (_, payload) = diagnosed.unwrap();
        assert_eq!(payload["invalid_sealed_commit_references"], 1);
        assert_eq!(payload["issues"][0]["subject"], missing);
        assert_eq!(
            diagnosed_work,
            baseline_work + 1,
            "the missing interleaved root may add its branch read but must not reset shared-DAG validation"
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn doctor_corrupt_root_before_shared_valid_root_preserves_cache_order_and_ownership() {
        let storage_root = test_storage_root("doctor-corrupt-root-shared-cache");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let base = write_http_test_commit(&objects);
        let base_record = codefire_store::read_object_record(&objects, &base).unwrap();
        let child = codefire_store::store_object(
            &objects,
            "commit",
            codefire_store::commit_payload(
                vec![base.clone()],
                base_record.payload["roots"].as_object().unwrap().clone(),
                base_record.payload["certificate"]
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .unwrap();
        for (branch, head) in [("base", &base), ("child", &child)] {
            write_json_atomic(
                &remote_branch_path(&project_root, branch),
                &json!({"version": 1, "name": branch, "head": head}),
            )
            .unwrap();
        }

        let run = || {
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let deadline = HttpRequestDeadline::from_profile(&profile);
            let mut ledger = Some(
                codefire_core::DecisionWorkLedger::new(
                    profile,
                    codefire_core::DecisionWorkCommandKind::HttpRequest,
                    "doctor corrupt shared cache",
                )
                .unwrap(),
            );
            let mut prepared = None;
            let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);
            let response = handle_http_doctor(&storage_root, "org", "app", false, &mut context);
            let snapshot = ledger.as_ref().unwrap().snapshot();
            let object_records =
                snapshot.cumulative[&codefire_core::DecisionWorkDimension::ObjectRecords];
            let retained = snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes];
            let temporary = snapshot
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default();
            (response, object_records, retained, temporary)
        };

        let (baseline, baseline_work, _, _) = run();
        assert_eq!(baseline.unwrap().1["invalid_sealed_commit_references"], 0);

        let corrupt = commit_id_between(&base, &child);
        let (low, high) = if base < child {
            (&base, &child)
        } else {
            (&child, &base)
        };
        assert!(low < &corrupt && &corrupt < high);
        fs::copy(
            objects.join("commits").join(format!("{base}.json")),
            objects.join("commits").join(format!("{corrupt}.json")),
        )
        .unwrap();
        write_json_atomic(
            &remote_branch_path(&project_root, "corrupt"),
            &json!({"version": 1, "name": "corrupt", "head": corrupt}),
        )
        .unwrap();

        let (diagnosed, diagnosed_work, retained, temporary) = run();
        let (_, payload) = diagnosed.unwrap();
        assert_eq!(payload["invalid_sealed_commit_references"], 1);
        let subjects = payload["issues"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|issue| issue["kind"] == "remote_sealed_commit_reference_invalid")
            .map(|issue| issue["subject"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(subjects, vec![corrupt.as_str()]);
        assert_eq!(
            diagnosed_work,
            baseline_work + 3,
            "the corrupt root adds one branch read, one scan read, and one validation read without resetting shared-DAG validation"
        );
        assert_eq!(
            retained,
            u64::try_from(codefire_store::canonical_json(&payload).unwrap().len()).unwrap()
        );
        assert_eq!(temporary, 0);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn full_http_doctor_reports_each_corrupt_root_in_stable_order() {
        let storage_root = test_storage_root("doctor-multiple-corrupt-roots");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let first = format!("CF-COMMIT-{}", "a".repeat(64));
        let second = format!("CF-COMMIT-{}", "b".repeat(64));
        for (branch, head) in [("alpha", &first), ("beta", &second)] {
            write_json_atomic(
                &remote_branch_path(&project_root, branch),
                &json!({"version": 1, "name": branch, "head": head}),
            )
            .unwrap();
        }
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = Some(
            codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "doctor corrupt roots",
            )
            .unwrap(),
        );
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);

        let (status, payload) =
            handle_http_doctor(&storage_root, "org", "app", false, &mut context).unwrap();

        assert_eq!(status, 200);
        assert_eq!(payload["invalid_sealed_commit_references"], 2);
        let subjects = payload["issues"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|issue| issue["kind"] == "remote_sealed_commit_reference_invalid")
            .map(|issue| issue["subject"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(subjects, vec![first.as_str(), second.as_str()]);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_existing_project_commits_one_owner_and_hands_off_null_payload() {
        let storage_root = test_storage_root("upload-owner-fresh");
        let (project_root, request) = upload_request_fixture(&storage_root, "upload-fresh");

        let (result, mut ledger, mut prepared) = run_accounted_upload(&storage_root, &request);

        assert_eq!(result.unwrap(), (200, Value::Null));
        let response = release_prepared_upload_response(&mut ledger, &mut prepared);
        assert_eq!(
            response,
            upload_result("main", request["head"].as_str().unwrap())
        );
        assert_eq!(
            read_json(&remote_branch_path(&project_root, "main")).unwrap()["head"],
            request["head"]
        );
        assert_eq!(
            read_json(&project_root.join("gc_state.json")).unwrap()["current_generation"],
            1
        );
        assert!(codefire_store::object_record_path(
            &remote_dirs(&project_root).objects,
            request["head"].as_str().unwrap(),
        )
        .is_some());
        assert!(ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_completed_replay_with_fresh_nonce_skips_changed_business_authority() {
        let storage_root = test_storage_root("upload-owner-replay");
        let (project_root, mut request) = upload_request_fixture(&storage_root, "upload-replay");
        write_upload_request_signature_policy(&project_root, false);
        sign_upload_request(&mut request);

        let (first, mut first_ledger, mut first_prepared) =
            run_accounted_upload(&storage_root, &request);
        assert_eq!(first.unwrap(), (200, Value::Null));
        let first_response =
            release_prepared_upload_response(&mut first_ledger, &mut first_prepared);
        let generation_before = fs::read(project_root.join("gc_state.json")).unwrap();
        let nonce_before = fs::read(project_root.join("request_nonce_cache.json")).unwrap();

        let objects = remote_dirs(&project_root).objects;
        let original = write_http_test_commit(&objects);
        let mut unrelated_payload = codefire_store::read_object_record(&objects, &original)
            .unwrap()
            .payload;
        unrelated_payload["message"] = Value::String("changed after completion".to_string());
        let unrelated =
            codefire_store::store_object(&objects, "commit", unrelated_payload).unwrap();
        let changed_branch = json!({
            "version": 1,
            "name": "main",
            "head": unrelated,
            "generation": 99,
        });
        write_json_atomic(&remote_branch_path(&project_root, "main"), &changed_branch).unwrap();
        write_upload_request_signature_policy(&project_root, true);
        sign_upload_request(&mut request);

        let (replay, mut replay_ledger, mut replay_prepared) =
            run_accounted_upload(&storage_root, &request);

        assert_eq!(replay.unwrap(), (200, Value::Null));
        assert_eq!(
            release_prepared_upload_response(&mut replay_ledger, &mut replay_prepared),
            first_response
        );
        assert_eq!(
            read_json(&remote_branch_path(&project_root, "main")).unwrap(),
            changed_branch,
            "completed replay must not rebuild or overwrite branch authority"
        );
        assert_eq!(
            fs::read(project_root.join("gc_state.json")).unwrap(),
            generation_before,
            "completed replay must not rebuild generation authority"
        );
        assert_ne!(
            fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            nonce_before,
            "fresh replay nonce must still join the atomic publication"
        );
        assert!(replay_ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_authenticates_nonce_before_non_fast_forward_semantics() {
        let storage_root = test_storage_root("upload-auth-before-non-ff");
        let (project_root, mut signed) =
            upload_request_fixture(&storage_root, "upload-auth-precedence");
        let objects = remote_dirs(&project_root).objects;
        let original = write_http_test_commit(&objects);
        let mut unrelated_payload = codefire_store::read_object_record(&objects, &original)
            .unwrap()
            .payload;
        unrelated_payload["message"] = Value::String("unrelated branch head".to_string());
        let unrelated =
            codefire_store::store_object(&objects, "commit", unrelated_payload).unwrap();
        write_json_atomic(
            &remote_branch_path(&project_root, "main"),
            &json!({"version": 1, "name": "main", "head": unrelated}),
        )
        .unwrap();
        write_upload_request_signature_policy(&project_root, false);
        sign_upload_request(&mut signed);
        let mut invalid = signed.clone();
        invalid["request_signature"]["signature"] = Value::String("0".repeat(64));

        for request in [
            {
                let mut missing = signed.clone();
                missing.as_object_mut().unwrap().remove("request_signature");
                missing
            },
            invalid,
        ] {
            let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);
            assert!(matches!(
                result.unwrap_err(),
                CliError::AuthenticationOrSignature(_)
            ));
            assert!(prepared.is_none());
            assert!(ledger
                .as_ref()
                .unwrap()
                .snapshot()
                .current
                .values()
                .all(|amount| *amount == 0));
        }

        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "upload",
            Some("main"),
            None,
        );
        super::super::signatures::validate_remote_request_signature(
            &project_root,
            "upload",
            "uploader",
            &target,
            &signed,
            signed.get("request_signature"),
        )
        .unwrap()
        .publish(&project_root)
        .unwrap();
        let nonce_before = fs::read(project_root.join("request_nonce_cache.json")).unwrap();
        let (reused, reused_ledger, reused_prepared) = run_accounted_upload(&storage_root, &signed);
        let error = reused.unwrap_err();
        assert!(matches!(error, CliError::AuthenticationOrSignature(_)));
        assert!(error.to_string().contains("nonce has already been used"));
        assert!(reused_prepared.is_none());
        assert!(reused_ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));

        sign_upload_request(&mut signed);
        let (non_ff, non_ff_ledger, non_ff_prepared) = run_accounted_upload(&storage_root, &signed);
        let non_ff_error = non_ff.unwrap_err();
        assert!(
            matches!(non_ff_error, CliError::IdempotencyConflict(_)),
            "unexpected non-fast-forward diagnostic: {non_ff_error:?}"
        );
        assert!(non_ff_prepared.is_none());
        assert_eq!(
            fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            nonce_before,
            "non-fast-forward rejection must cancel fresh nonce ownership"
        );
        assert!(non_ff_ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_missing_project_publishes_complete_project_with_response() {
        let storage_root = test_storage_root("upload-missing-project");
        let (project_root, request) = upload_request_fixture(&storage_root, "missing-project");
        fs::remove_dir_all(&project_root).unwrap();
        reset_ledger_bootstrap_rename_attempts();
        reset_http_upload_bootstrap_handoff_validations();

        let (result, mut ledger, mut prepared) = run_accounted_upload(&storage_root, &request);

        assert_eq!(result.unwrap(), (200, Value::Null));
        assert_eq!(
            release_prepared_upload_response(&mut ledger, &mut prepared),
            upload_result("main", request["head"].as_str().unwrap())
        );
        assert_eq!(
            read_json(&remote_branch_path(&project_root, "main")).unwrap()["head"],
            request["head"]
        );
        assert_eq!(
            read_json(&project_root.join("gc_state.json")).unwrap()["current_generation"],
            1
        );
        assert!(ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert_eq!(ledger_bootstrap_rename_attempts(), 1);
        assert_eq!(
            http_upload_bootstrap_handoff_validations(),
            1,
            "bootstrap handoff validation must run only in the pre-rename callback"
        );

        let (replay, mut replay_ledger, mut replay_prepared) =
            run_accounted_upload(&storage_root, &request);
        assert_eq!(replay.unwrap(), (200, Value::Null));
        release_prepared_upload_response(&mut replay_ledger, &mut replay_prepared);
        assert_eq!(
            ledger_bootstrap_rename_attempts(),
            1,
            "completed replay on an existing target must not rename the project again"
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_missing_project_preseal_failure_cleans_hidden_owner() {
        let storage_root = test_storage_root("upload-missing-preseal-failure");
        let project_root = http_project_root(&storage_root, "org", "app");
        let request = json!({
            "branch": "local-main",
            "head": format!("CF-COMMIT-{}", "a".repeat(64)),
            "objects": [],
            "actor": "uploader",
            "idempotency_key": "missing-preseal-failure",
        });

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(result.is_err());
        assert!(prepared.is_none());
        assert!(!project_root.exists());
        let org_root = storage_root.join("projects/org");
        let hidden_stages = fs::read_dir(&org_root).map_or(0, |entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.starts_with(".bootstrap-"))
                })
                .count()
        });
        assert_eq!(hidden_stages, 0);
        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_bootstrap_collision_preserves_foreign_target_and_recovers_hidden_owner() {
        let storage_root = test_storage_root("upload-bootstrap-collision");
        let (project_root, request) = upload_request_fixture(&storage_root, "bootstrap-collision");
        fs::remove_dir_all(&project_root).unwrap();
        INJECT_HTTP_UPLOAD_BOOTSTRAP_CALLBACK.set(1);

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(matches!(
            result.unwrap_err(),
            CliError::Io(ref error) if error.kind() == std::io::ErrorKind::AlreadyExists
        ));
        assert!(prepared.is_none());
        assert!(project_root.is_dir());
        assert!(!project_root.join("remote_project.json").exists());
        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let org_root = project_root.parent().unwrap();
        assert_eq!(
            fs::read_dir(org_root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".bootstrap-")))
                .count(),
            1,
            "rename collision must retain exactly one recoverable hidden owner"
        );

        fs::remove_dir(&project_root).unwrap();
        let (retry, mut retry_ledger, mut retry_prepared) =
            run_accounted_upload(&storage_root, &request);
        assert_eq!(retry.unwrap(), (200, Value::Null));
        release_prepared_upload_response(&mut retry_ledger, &mut retry_prepared);
        assert!(project_root.join("remote_project.json").is_file());
        assert_eq!(
            fs::read_dir(org_root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".bootstrap-")))
                .count(),
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_bootstrap_callback_failure_keeps_final_absent_and_stage_recoverable() {
        let storage_root = test_storage_root("upload-bootstrap-callback-failure");
        let (project_root, request) =
            upload_request_fixture(&storage_root, "bootstrap-callback-failure");
        fs::remove_dir_all(&project_root).unwrap();
        INJECT_HTTP_UPLOAD_BOOTSTRAP_CALLBACK.set(2);

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(matches!(
            result.unwrap_err(),
            CliError::RequestDeadlineExceeded {
                phase: "injected upload bootstrap callback",
                ..
            }
        ));
        assert!(prepared.is_none());
        assert!(!project_root.exists());
        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let org_root = project_root.parent().unwrap();
        assert_eq!(
            fs::read_dir(org_root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".bootstrap-")))
                .count(),
            1
        );

        let (retry, mut retry_ledger, mut retry_prepared) =
            run_accounted_upload(&storage_root, &request);
        assert_eq!(retry.unwrap(), (200, Value::Null));
        release_prepared_upload_response(&mut retry_ledger, &mut retry_prepared);
        assert!(project_root.join("remote_project.json").is_file());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_bootstrap_rename_failure_keeps_final_absent_and_stage_recoverable() {
        let storage_root = test_storage_root("upload-bootstrap-rename-failure");
        let (project_root, request) =
            upload_request_fixture(&storage_root, "bootstrap-rename-failure");
        fs::remove_dir_all(&project_root).unwrap();
        reset_http_upload_bootstrap_handoff_validations();
        inject_ledger_bootstrap_rename_failure_once();

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(matches!(
            result.unwrap_err(),
            CliError::Io(ref error) if error.kind() == std::io::ErrorKind::PermissionDenied
        ));
        assert!(prepared.is_none());
        assert!(!project_root.exists());
        assert_eq!(
            http_upload_bootstrap_handoff_validations(),
            1,
            "rename failure must occur after the sole handoff validation"
        );
        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let org_root = project_root.parent().unwrap();
        assert_eq!(
            fs::read_dir(org_root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".bootstrap-")))
                .count(),
            1
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_existing_project_does_not_apply_bootstrap_entry_limit() {
        let storage_root = test_storage_root("upload-existing-bootstrap-limit");
        let (_project_root, mut request) =
            upload_request_fixture(&storage_root, "existing-bootstrap-limit");
        request["objects"] = Value::Array(vec![json!({}); 2_032]);

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        let error = result.unwrap_err();
        assert!(!error.to_string().contains("bounded recovery limits"));
        assert!(prepared.is_none());
        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_missing_project_rejects_entry_overflow_before_effects() {
        let storage_root = test_storage_root("upload-missing-entry-limit");
        let project_root = http_project_root(&storage_root, "org", "app");
        let request = json!({
            "branch": "local-main",
            "head": format!("CF-COMMIT-{}", "a".repeat(64)),
            "objects": vec![json!({}); 2_032],
            "actor": "uploader",
            "idempotency_key": "missing-entry-limit",
        });

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(matches!(result.unwrap_err(), CliError::ResourceBudget(_)));
        assert!(prepared.is_none());
        assert!(!project_root.exists());
        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_bootstrap_payload_limit_is_missing_only_and_pre_effect() {
        let missing_storage = test_storage_root("upload-missing-payload-limit");
        let (missing_root, mut missing_request) =
            upload_request_fixture(&missing_storage, "missing-payload-limit");
        missing_request["objects"][0]["bootstrap_payload"] =
            Value::String("x".repeat(7 * 1024 * 1024));
        fs::remove_dir_all(&missing_root).unwrap();

        let (missing, missing_ledger, missing_prepared) =
            run_accounted_upload(&missing_storage, &missing_request);
        assert!(matches!(missing.unwrap_err(), CliError::ResourceBudget(_)));
        assert!(missing_prepared.is_none());
        assert!(!missing_root.exists());
        let missing_ledger = missing_ledger.as_ref().unwrap();
        assert!(!missing_ledger.has_open_permits());
        assert!(missing_ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));

        let existing_storage = test_storage_root("upload-existing-payload-limit");
        let (_existing_root, mut existing_request) =
            upload_request_fixture(&existing_storage, "existing-payload-limit");
        existing_request["objects"][0]["bootstrap_payload"] =
            Value::String("x".repeat(7 * 1024 * 1024));
        let (existing, mut existing_ledger, mut existing_prepared) =
            run_accounted_upload(&existing_storage, &existing_request);
        assert_eq!(existing.unwrap(), (200, Value::Null));
        assert_eq!(
            release_prepared_upload_response(&mut existing_ledger, &mut existing_prepared),
            upload_result("main", existing_request["head"].as_str().unwrap())
        );
        let existing_ledger = existing_ledger.as_ref().unwrap();
        assert!(!existing_ledger.has_open_permits());
        assert!(existing_ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(missing_storage);
        let _ = fs::remove_dir_all(existing_storage);
    }

    #[test]
    fn missing_upload_generic_body_limit_dominates_before_bootstrap_dispatch() {
        fn body_with_exact_size(mut request: Value, target: usize) -> Vec<u8> {
            request["transport_padding"] = Value::String(String::new());
            let base = serde_json::to_vec(&request).unwrap().len();
            assert!(base <= target);
            request["transport_padding"] = Value::String("x".repeat(target - base));
            let body = serde_json::to_vec(&request).unwrap();
            assert_eq!(body.len(), target);
            body
        }

        for body_size in [MAX_HTTP_BODY_BYTES - 1, MAX_HTTP_BODY_BYTES] {
            let storage_root = test_storage_root(&format!("generic-body-{body_size}"));
            let (project_root, request) =
                upload_request_fixture(&storage_root, &format!("generic-body-{body_size}"));
            fs::remove_dir_all(&project_root).unwrap();
            let body = body_with_exact_size(request, body_size);
            let mut wire = format!(
                "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .into_bytes();
            wire.extend_from_slice(&body);
            let request_len = wire.len();
            let mut stream = Cursor::new(wire);

            handle_http_stream(&storage_root, &mut stream).unwrap();

            let wire = stream.into_inner();
            let response = std::str::from_utf8(&wire[request_len..]).unwrap();
            assert!(response.starts_with("HTTP/1.1 200 OK"));
            assert!(!response.contains("HTTP/1.1 413"));
            assert!(!response.contains("HTTP/1.1 409"));
            assert!(project_root.exists());
            let _ = fs::remove_dir_all(storage_root);
        }

        let storage_root = test_storage_root("generic-body-plus-one");
        let project_root = http_project_root(&storage_root, "org", "app");
        let request = format!(
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_HTTP_BODY_BYTES + 1
        );
        let request_len = request.len();
        let mut stream = Cursor::new(request.into_bytes());

        handle_http_stream(&storage_root, &mut stream).unwrap();

        let wire = stream.into_inner();
        let response = std::str::from_utf8(&wire[request_len..]).unwrap();
        assert!(response.starts_with("HTTP/1.1 413 Payload Too Large"));
        assert!(!response.contains("HTTP/1.1 200"));
        assert!(!response.contains("HTTP/1.1 409"));
        assert!(response.contains("\"dimension\":\"request_body_bytes\""));
        assert!(response.contains("\"phase\":\"request_body\""));
        assert!(response.contains("\"observed\":16777217"));
        assert!(response.contains("\"limit\":16777216"));
        assert!(response.contains("\"exit_code\":60"));
        assert!(!project_root.exists());
        assert!(!storage_root.join(".codefire-server").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn missing_upload_bootstrap_payload_quantized_boundary_follows_parser_accounting() {
        fn boundary_request(delta: i64) -> (Value, BootstrapStageFootprint, u64) {
            let mut request = json!({
                "branch": "local-main",
                "head": format!("CF-COMMIT-{}", "a".repeat(64)),
                "objects": [{"bootstrap_payload": ""}],
                "actor": "u",
                "idempotency_key": "bootstrap-payload-boundary",
            });
            let base = http_upload_bootstrap_footprint(
                &request,
                "main",
                "local-main",
                "u",
                request["head"].as_str().unwrap(),
                request["objects"].as_array().unwrap(),
            )
            .unwrap();
            let limit = BootstrapStageFootprint::maximum_payload_bytes(base.entries).unwrap();
            let target = if delta < 0 {
                limit - delta.unsigned_abs()
            } else {
                limit + delta as u64
            };
            let growth = target - base.payload_bytes;
            request["objects"][0]["bootstrap_payload"] =
                Value::String("x".repeat(usize::try_from(growth / 2).unwrap()));
            assert_eq!(growth % 2, 0, "bootstrap payload grows in two-byte quanta");
            let footprint = http_upload_bootstrap_footprint(
                &request,
                "main",
                "local-main",
                request["actor"].as_str().unwrap(),
                request["head"].as_str().unwrap(),
                request["objects"].as_array().unwrap(),
            )
            .unwrap();
            assert_eq!(footprint.payload_bytes, target);
            (request, footprint, limit)
        }

        // Every variable staged payload component is retained both as its
        // unpublished file and as rollback input, so this route grows in
        // two-byte quanta. The direct gate contract separately covers the
        // mathematical limit +/- 1; HTTP exercises exact and the first
        // realizable rejected value.
        for delta in [0_i64, 2] {
            let storage_root = test_storage_root(&format!("bootstrap-wire-{delta}"));
            let project_root = http_project_root(&storage_root, "org", "app");
            let (body_value, footprint, limit) = boundary_request(delta);
            let body = serde_json::to_vec(&body_value).unwrap();
            assert!(body.len() < MAX_HTTP_BODY_BYTES);
            let mut wire = format!(
                "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .into_bytes();
            wire.extend_from_slice(&body);
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let deadline = HttpRequestDeadline::from_profile(&profile);
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "bootstrap parser handoff",
            )
            .unwrap();
            let mut request =
                read_http_request_with_ledger(&mut Cursor::new(wire), &mut ledger, &deadline)
                    .unwrap();
            assert_eq!(
                ledger.snapshot().cumulative
                    [&codefire_core::DecisionWorkDimension::RequestBodyBytes],
                u64::try_from(body.len()).unwrap()
            );
            let mut owned_ledger = Some(ledger);
            let mut prepared = None;
            let result = handle_http_upload_with_ledger(
                &storage_root,
                "org",
                "app",
                "main",
                request.body(),
                &mut http_route_context_for_test(&mut owned_ledger, &mut prepared, &deadline),
            );
            request
                .release_body(owned_ledger.as_mut().unwrap())
                .unwrap();

            let error = result.unwrap_err();
            let (status, payload) = http_error_response(&error);
            if delta == 0 {
                assert_ne!(
                    status, 413,
                    "in-bound footprint must reach later validation"
                );
                assert_ne!(status, 200);
                assert_ne!(status, 409);
            } else {
                assert_eq!(status, 413);
                assert_eq!(payload["exit_code"], 60);
                assert_eq!(payload["diagnostics"][0]["dimension"], "aggregate_bytes");
                assert_eq!(payload["diagnostics"][0]["phase"], "staging");
                assert_eq!(payload["diagnostics"][0]["observed"], limit + 2);
                assert_eq!(payload["diagnostics"][0]["limit"], limit);
                assert_eq!(footprint.payload_bytes, limit + 2);
                assert!(!storage_root.join(".codefire-server").exists());
            }
            terminate_http_request(
                owned_ledger.as_mut().unwrap(),
                http_request_terminal_reason(&error),
            )
            .unwrap();
            let ledger = owned_ledger.as_ref().unwrap();
            assert!(!ledger.has_open_permits());
            assert!(ledger
                .snapshot()
                .current
                .values()
                .all(|amount| *amount == 0));
            assert!(prepared.is_none());
            assert!(!project_root.exists());
            let _ = fs::remove_dir_all(storage_root);
        }
    }

    #[test]
    fn http_upload_preseal_commit_signature_failure_cancels_nonce_proof_and_batch() {
        let storage_root = test_storage_root("upload-preseal-cleanup");
        let (project_root, mut request) =
            upload_request_fixture(&storage_root, "upload-preseal-cleanup");
        let incoming_objects = storage_root.join("incoming-objects");
        let original_head = request["head"].as_str().unwrap().to_string();
        let mut payload = codefire_store::read_object_record(&incoming_objects, &original_head)
            .unwrap()
            .payload;
        payload["signature"] = json!({
            "type": "commit_signature",
            "algorithm": "hmac-sha256",
            "signer": "uploader",
            "signature": "0".repeat(64),
        });
        let bad_head = codefire_store::store_object(&incoming_objects, "commit", payload).unwrap();
        request["head"] = Value::String(bad_head.clone());
        request["objects"] =
            Value::Array(collect_object_records(&incoming_objects, &bad_head).unwrap());
        write_upload_request_signature_policy(&project_root, true);
        sign_upload_request(&mut request);

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(matches!(
            result.unwrap_err(),
            CliError::AuthenticationOrSignature(_)
        ));
        assert!(prepared.is_none());
        assert!(!project_root.join("request_nonce_cache.json").exists());
        assert!(!remote_branch_path(&project_root, "main").exists());
        assert!(!project_root.join("gc_state.json").exists());
        assert!(
            codefire_store::object_record_path(&remote_dirs(&project_root).objects, &bad_head,)
                .is_none()
        );
        assert!(ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_upload_rejects_payload_tamper_before_nonce_or_remote_write() {
        let storage_root = test_storage_root("payload-tamper");
        let project_root = http_project_root(&storage_root, "org", "app");
        fs::create_dir_all(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "keys": {
                        "alice-request": {
                            "secret": "alice-request-key",
                            "signers": ["alice"]
                        }
                    }
                }
            }),
        )
        .unwrap();
        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "upload",
            Some("main"),
            None,
        );
        let original = json!({
            "branch": "main",
            "head": "CF-COMMIT-original",
            "objects": [],
            "actor": "alice",
            "idempotency_key": "request-1",
            "lock": {},
        });
        let signature = super::super::signatures::sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &original,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        let mut tampered = original;
        tampered["head"] = Value::String("CF-COMMIT-tampered".to_string());
        tampered["request_signature"] = signature;

        let error = handle_http_upload(&storage_root, "org", "app", "main", &tampered).unwrap_err();

        assert_eq!(
            error.exit_code(),
            ExitCode::AuthenticationOrSignatureFailure.code()
        );
        assert!(error.to_string().contains("payload digest mismatch"));
        assert!(!project_root.join("request_nonce_cache.json").exists());
        assert!(!remote_branch_path(&project_root, "main").exists());
        assert!(!project_root.join("objects").exists());
        assert!(!project_root.join("locks").exists());
    }

    #[test]
    fn upload_staging_ignores_unreachable_existing_sentinels() {
        let storage_root = test_storage_root("upload-closure-only");
        let existing_objects = storage_root.join("existing");
        let staging_objects = storage_root.join("staging");
        let reachable_id = codefire_store::store_object(
            &existing_objects,
            "blob",
            json!({"type": "blob", "version": 1, "encoding": "base64", "content": "eA=="}),
        )
        .unwrap();
        let mut sentinel_ids = Vec::new();
        for index in 0..512_u64 {
            sentinel_ids.push(
                codefire_store::store_object(
                    &existing_objects,
                    "blob",
                    json!({
                        "type": "blob",
                        "version": 1,
                        "encoding": "base64",
                        "content": format!("sentinel-{index}")
                    }),
                )
                .unwrap(),
            );
        }
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let mut policy_value = serde_json::to_value(&policy).unwrap();
        policy_value["extensions"] = json!([{
            "namespace": "example.test",
            "name": "reachable-existing",
            "version": 1,
            "critical": false,
            "references": [&reachable_id]
        }]);
        let records = vec![policy_value];

        let preflight =
            preflight_http_upload_for_test(&existing_objects, &policy.object_id, &records).unwrap();
        assert_eq!(preflight.incoming_closure, vec![0]);
        assert_eq!(preflight.existing_closure, vec![reachable_id.clone()]);
        assert_eq!(
            preflight.object_set_digest.as_str(),
            source_object_set_digest(&records).unwrap()
        );

        stage_http_upload_closure(&existing_objects, &staging_objects, &records, &preflight)
            .unwrap();

        assert!(
            codefire_store::object_record_path(&staging_objects, &policy.object_id)
                .unwrap()
                .exists()
        );
        assert!(
            codefire_store::object_record_path(&staging_objects, &reachable_id)
                .unwrap()
                .exists()
        );
        for sentinel_id in sentinel_ids {
            assert!(codefire_store::object_record_path(&staging_objects, &sentinel_id).is_none());
        }
        assert_eq!(count_regular_files(&staging_objects), 2);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_budget_failure_is_typed_413_and_has_no_effects() {
        let storage_root = test_storage_root("upload-budget-no-effects");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "keys": {
                        "alice-request": {
                            "secret": "alice-request-key",
                            "signers": ["alice"]
                        }
                    }
                }
            }),
        )
        .unwrap();
        let oversized_commit = codefire_store::object_record(
            "commit",
            json!({
                "type": "commit",
                "version": 1,
                "parents": [],
                "roots": {},
                "padding": "x".repeat(MAX_HTTP_BODY_BYTES + 1024)
            }),
        )
        .unwrap();
        let oversized_head = oversized_commit.object_id.clone();
        let mut request = json!({
            "branch": "main",
            "head": oversized_head,
            "objects": [serde_json::to_value(&oversized_commit).unwrap()],
            "actor": "alice"
        });
        let target = super::super::signatures::remote_request_target(
            "org",
            "app",
            "upload",
            Some("main"),
            None,
        );
        request["request_signature"] = super::super::signatures::sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &request,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();

        let error = handle_http_upload(&storage_root, "org", "app", "main", &request).unwrap_err();

        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert_eq!(http_error_response(&error).0, 413);
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["kind"], "resource_budget_exceeded");
        assert_eq!(diagnostic["dimension"], "temporary_bytes");
        assert_eq!(diagnostic["continuation_supported"], false);
        assert!(!project_root.join("request_nonce_cache.json").exists());
        assert!(!remote_branch_path(&project_root, "main").exists());
        assert!(!project_root.join("gc_state.json").exists());
        assert!(codefire_store::object_record_path(
            &remote_dirs(&project_root).objects,
            &oversized_head,
        )
        .is_none());
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_record_precharge_accepts_count_boundary_and_rejects_next() {
        for count in [99_999_u64, 100_000] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::http_v1(),
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                "http upload",
            )
            .unwrap();
            precharge_upload_record_count(&mut ledger, count).unwrap();
            assert_eq!(
                ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::ObjectRecords],
                count
            );
            assert!(!ledger
                .snapshot()
                .current
                .contains_key(&codefire_core::DecisionWorkDimension::RetainedBytes));
            assert!(!ledger
                .snapshot()
                .peak
                .contains_key(&codefire_core::DecisionWorkDimension::RetainedBytes));
        }

        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        let error = precharge_upload_record_count(&mut ledger, 100_001).unwrap_err();
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["dimension"], "object_records");
        assert_eq!(diagnostic["observed"], 100_001);
        assert_eq!(diagnostic["limit"], 100_000);
        assert_eq!(diagnostic["continuation_supported"], false);
        assert!(ledger.snapshot().cumulative.is_empty());
    }

    #[test]
    fn upload_preflight_success_releases_auxiliary_and_traversal_ownership_exactly() {
        let storage_root = test_storage_root("upload-preflight-success-ownership");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let record = serde_json::to_value(&policy).unwrap();
        let logical_bytes = compact_json_size(&record).unwrap();
        let frame_bytes = codefire_core::logical_retained_frame_bytes(&policy.object_id).unwrap();
        let digest_frame_bytes =
            codefire_core::logical_retained_frame_bytes(HTTP_OBJECT_SET_DIGEST_TEMPLATE).unwrap();
        let path_frame_bytes = upload_object_path_frame_bytes(&objects, &policy.object_id).unwrap();
        let before = test_tree_snapshot(&storage_root);
        let mut ledger = upload_preflight_ledger();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());

        let preflight = preflight_http_upload(
            &objects,
            &policy.object_id,
            &[record],
            &mut ledger,
            &deadline,
        )
        .unwrap();

        assert_eq!(preflight.incoming_closure, vec![0]);
        assert_upload_preflight_current_zero(&ledger);
        let snapshot = ledger.snapshot();
        assert_eq!(
            snapshot.peak[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            logical_bytes
        );
        assert_eq!(
            snapshot.peak[&codefire_core::DecisionWorkDimension::RetainedBytes],
            (u64::try_from(std::mem::size_of::<u64>()).unwrap()
                + codefire_core::LOGICAL_FRAME_BASE_BYTES
                + digest_frame_bytes)
                .max(
                    u64::try_from(std::mem::size_of::<u64>()).unwrap()
                        + codefire_core::LOGICAL_FRAME_BASE_BYTES
                        + path_frame_bytes,
                )
                .max(codefire_core::LOGICAL_FRAME_BASE_BYTES + frame_bytes * 2)
        );
        assert_eq!(
            snapshot.cumulative[&codefire_core::DecisionWorkDimension::ObjectRecords],
            1
        );
        assert_eq!(
            snapshot.cumulative[&codefire_core::DecisionWorkDimension::GraphNodes],
            1
        );
        assert_eq!(
            snapshot.counters[&codefire_core::DecisionWorkCounter::ObjectRecords],
            1
        );
        assert_eq!(
            snapshot.counters[&codefire_core::DecisionWorkCounter::UniqueNodesDiscovered],
            1
        );
        assert_eq!(
            snapshot.counters[&codefire_core::DecisionWorkCounter::FrontierInsertionAttempts],
            1
        );
        assert_eq!(before, test_tree_snapshot(&storage_root));
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_preflight_digest_failure_releases_owned_string_without_effects() {
        let storage_root = test_storage_root("upload-preflight-digest-cleanup");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let record = serde_json::to_value(&policy).unwrap();
        let digest_frame_bytes =
            codefire_core::logical_retained_frame_bytes(HTTP_OBJECT_SET_DIGEST_TEMPLATE).unwrap();
        let before = test_tree_snapshot(&storage_root);
        let mut ledger = upload_preflight_ledger();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());
        let mut digest_polls = 0_u64;

        let error = preflight_http_upload_with_polls(
            UploadPreflightInput {
                existing_objects: &objects,
                head: &policy.object_id,
                records: &[record],
                deadline: &deadline,
            },
            &mut ledger,
            UploadPreflightPolls {
                digest: || Ok(()),
                digest_result: || {
                    digest_polls += 1;
                    Err(deadline.error("upload object-set digest"))
                },
                traversal: || Ok(()),
            },
        )
        .unwrap_err();

        assert!(matches!(error, CliError::RequestDeadlineExceeded { .. }));
        assert_eq!(digest_polls, 1);
        assert_upload_preflight_current_zero(&ledger);
        assert_eq!(
            ledger.snapshot().peak[&codefire_core::DecisionWorkDimension::RetainedBytes],
            u64::try_from(std::mem::size_of::<u64>()).unwrap()
                + codefire_core::LOGICAL_FRAME_BASE_BYTES
                + digest_frame_bytes
        );
        assert_eq!(before, test_tree_snapshot(&storage_root));
        assert!(!project_root.join("request_nonce_cache.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_preflight_malformed_incoming_releases_work_without_effects() {
        let storage_root = test_storage_root("upload-preflight-malformed-cleanup");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let mut malformed = serde_json::to_value(&policy).unwrap();
        malformed["payload"]["policy"]["portable"] = Value::Bool(false);
        let before = test_tree_snapshot(&storage_root);
        let mut ledger = upload_preflight_ledger();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());

        let error = preflight_http_upload(
            &objects,
            &policy.object_id,
            &[malformed],
            &mut ledger,
            &deadline,
        )
        .unwrap_err();

        assert!(
            matches!(
                &error,
                CliError::Store(store_error)
                    if matches!(
                        store_error.as_ref(),
                        codefire_store::StoreError::ObjectIdMismatch { .. }
                    )
            ),
            "unexpected malformed incoming error: {error:?}"
        );
        assert_upload_preflight_current_zero(&ledger);
        assert_eq!(before, test_tree_snapshot(&storage_root));
        assert!(!project_root.join("request_nonce_cache.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_preflight_deadline_with_frontier_releases_work_without_effects() {
        let storage_root = test_storage_root("upload-preflight-frontier-deadline-cleanup");
        let source_objects = storage_root.join("source-objects");
        let head = write_http_test_commit(&source_objects);
        let records = collect_object_records(&source_objects, &head).unwrap();
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let before = test_tree_snapshot(&storage_root);
        let mut ledger = upload_preflight_ledger();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());
        let mut polls = 0_u64;

        let error = preflight_http_upload_with_traversal_poll(
            &objects,
            &head,
            &records,
            &mut ledger,
            &deadline,
            || {
                polls += 1;
                if polls == 2 {
                    Err(deadline.error("upload closure validation"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();

        assert!(matches!(error, CliError::RequestDeadlineExceeded { .. }));
        assert_eq!(polls, 2);
        assert_upload_preflight_current_zero(&ledger);
        assert!(
            ledger.snapshot().maximum[&codefire_core::DecisionWorkDimension::MaximumFrontier] > 1,
            "the injected deadline must fire after the root populated the frontier"
        );
        assert_eq!(before, test_tree_snapshot(&storage_root));
        assert!(!project_root.join("request_nonce_cache.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_preflight_corrupt_existing_releases_work_without_effects() {
        let storage_root = test_storage_root("upload-preflight-corrupt-existing-cleanup");
        let source_objects = storage_root.join("source-objects");
        let head = write_http_test_commit(&source_objects);
        let mut records = collect_object_records(&source_objects, &head).unwrap();
        let commit = records
            .iter()
            .find(|record| record["object_id"] == head)
            .unwrap();
        let corrupt_id = commit["payload"]["roots"]["policy"]
            .as_str()
            .unwrap()
            .to_string();
        records.retain(|record| record["object_id"].as_str() != Some(&corrupt_id));
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        fs::write(
            objects.join("policies").join(format!("{corrupt_id}.json")),
            b"{",
        )
        .unwrap();
        let before = test_tree_snapshot(&storage_root);
        let mut ledger = upload_preflight_ledger();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());

        let error =
            preflight_http_upload(&objects, &head, &records, &mut ledger, &deadline).unwrap_err();

        assert!(
            matches!(
                &error,
                CliError::Store(store_error)
                    if matches!(store_error.as_ref(), codefire_store::StoreError::Json(_))
            ),
            "unexpected corrupt existing error: {error:?}"
        );
        assert_upload_preflight_current_zero(&ledger);
        assert_eq!(before, test_tree_snapshot(&storage_root));
        assert!(!project_root.join("request_nonce_cache.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_preflight_collision_mismatch_releases_work_without_effects() {
        let storage_root = test_storage_root("upload-preflight-collision-cleanup");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let requested = serde_json::to_value(&policy).unwrap();
        let mut existing = requested.clone();
        existing["legacy_test_extension"] = Value::Bool(true);
        let existing = codefire_store::object_record_document_from_value(existing).unwrap();
        codefire_store::store_object_record_document(&objects, existing).unwrap();
        let before = test_tree_snapshot(&storage_root);
        let mut ledger = upload_preflight_ledger();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());

        let error = preflight_http_upload(
            &objects,
            &policy.object_id,
            &[requested],
            &mut ledger,
            &deadline,
        )
        .unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(error
            .to_string()
            .contains("immutable object document diverges"));
        assert_upload_preflight_current_zero(&ledger);
        assert_eq!(before, test_tree_snapshot(&storage_root));
        assert!(!project_root.join("request_nonce_cache.json").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_compact_measurement_does_not_retain_nested_request_values_twice() {
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        let value = json!({"nested": ["alpha", "beta"]});
        let logical_bytes = account_compact_json(
            &mut ledger,
            &value,
            codefire_core::DecisionWorkPhase::Validation,
            "test compact JSON measurement",
        )
        .unwrap();

        assert_eq!(logical_bytes, compact_json_size(&value).unwrap());
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            logical_bytes
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    #[test]
    fn parsed_document_transfer_acquires_logical_retained_before_releasing_raw_scratch() {
        let record = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let mut value = serde_json::to_value(&record).unwrap();
        value["extensions"] = json!([{
            "namespace": "example.test",
            "name": "transfer-timeline",
            "version": 1,
            "critical": false,
            "references": ["CF-BLOB-test"]
        }]);
        let document = codefire_store::object_record_document_from_value(value.clone()).unwrap();
        let logical_bytes = compact_json_size(&value).unwrap();
        let physical_bytes = MAX_HTTP_BODY_BYTES as u64;
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    physical_bytes,
                ),
            )
            .unwrap();

        assert_eq!(
            transfer_parsed_object_document(
                &mut ledger,
                &document,
                physical_bytes,
                codefire_core::DecisionWorkPhase::RecordRead,
                "test parsed document transfer",
            )
            .unwrap(),
            logical_bytes
        );
        let snapshot = ledger.snapshot();
        assert_eq!(
            snapshot.peak[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            physical_bytes
        );
        assert_eq!(
            snapshot
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert_eq!(
            snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            logical_bytes
        );
        assert_eq!(
            snapshot.peak[&codefire_core::DecisionWorkDimension::RetainedBytes],
            logical_bytes
        );
    }

    #[test]
    fn upload_byte_units_use_compact_logical_and_pretty_physical_json() {
        let record = json!({
            "object_id": "CF-POLICY-test",
            "payload": {"items": [1, 2, 3, 4]}
        });
        let mut compact = JsonByteCounter::default();
        serde_json::to_writer(&mut compact, &record).unwrap();

        let physical = rendered_object_record_size(&record).unwrap();

        assert_eq!(
            compact.bytes,
            codefire_store::canonical_json(&record).unwrap().len() as u64
        );
        assert!(physical > compact.bytes);
    }

    #[test]
    fn upload_existing_closure_reserves_raw_bytes_before_parse() {
        let storage_root = test_storage_root("upload-existing-raw-precharge");
        let objects = storage_root.join("objects");
        ensure_object_dirs(&objects).unwrap();
        let existing_id = format!("CF-BLOB-{}", "a".repeat(64));
        let existing_path = objects.join("blobs").join(format!("{existing_id}.json"));
        let file = fs::File::create(&existing_path).unwrap();
        file.set_len(MAX_HTTP_BODY_BYTES as u64 + 1).unwrap();
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let mut policy_value = serde_json::to_value(&policy).unwrap();
        policy_value["extensions"] = json!([{
            "namespace": "example.test",
            "name": "oversized-existing",
            "version": 1,
            "critical": false,
            "references": [existing_id]
        }]);

        let error = preflight_http_upload_for_test(&objects, &policy.object_id, &[policy_value])
            .unwrap_err();

        assert!(matches!(error, CliError::ResourceBudget(_)));
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["dimension"], "temporary_bytes");
        assert_eq!(diagnostic["continuation_supported"], false);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_collision_accepts_large_physical_record_with_small_logical_value() {
        let storage_root = test_storage_root("upload-large-physical-small-logical");
        let objects = storage_root.join("objects");
        ensure_object_dirs(&objects).unwrap();
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let record = serde_json::to_value(&policy).unwrap();
        let rendered = serde_json::to_vec_pretty(&record).unwrap();
        assert!(rendered.len() < MAX_HTTP_BODY_BYTES);
        let mut physical = vec![b' '; MAX_HTTP_BODY_BYTES - rendered.len()];
        physical.extend_from_slice(&rendered);
        let path = objects
            .join("policies")
            .join(format!("{}.json", policy.object_id));
        fs::write(&path, physical).unwrap();

        let preflight =
            preflight_http_upload_for_test(&objects, &policy.object_id, &[record]).unwrap();

        assert_eq!(preflight.incoming_closure, vec![0]);
        assert!(preflight.existing_closure.is_empty());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn upload_stream_digest_accounts_every_hashed_byte() {
        let records = vec![json!({"object_id": "CF-POLICY-a", "payload": {"x": 1}})];
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();

        let mut digest = source_object_set_digest_ordered(&records, &[0], &mut ledger).unwrap();

        assert_eq!(digest.as_str(), source_object_set_digest(&records).unwrap());
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed],
            codefire_store::canonical_json(&Value::Array(records))
                .unwrap()
                .len() as u64
        );
        digest.release(&mut ledger).unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
    }

    #[test]
    fn upload_stream_digest_polls_inside_record_serialization() {
        let records = vec![json!({
            "object_id": "CF-POLICY-a",
            "payload": {"large": vec!["value"; 64]}
        })];
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        let mut polls = 0_u64;

        let error = source_object_set_digest_ordered_with_poll(&records, [0], &mut ledger, || {
            polls += 1;
            if polls >= 3 {
                Err(CliError::Usage("digest cancelled".to_string()))
            } else {
                Ok(())
            }
        })
        .unwrap_err();

        assert!(matches!(error, CliError::Usage(message) if message == "digest cancelled"));
        assert_eq!(polls, 3);
    }

    #[test]
    fn http_request_rejects_truncated_body() {
        let request =
            b"POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: 7\r\n\r\n{}";
        let error = read_http_request(&mut Cursor::new(request.to_vec())).unwrap_err();
        assert!(error.to_string().contains("incomplete HTTP request body"));
    }

    #[test]
    fn request_body_owner_rejects_foreign_ledger_without_consuming_payload() {
        let mut owner_ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "request body owner",
        )
        .unwrap();
        let mut foreign_ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "foreign request body owner",
        )
        .unwrap();
        let body = json!({"actor": "alice", "nested": {"value": 1}});
        let retained = compact_json_size(&body).unwrap();
        owner_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestBody,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();
        let mut owner = HttpRequestBodyOwnership::new(body, retained, &owner_ledger);

        let error = owner.release(&mut foreign_ledger).unwrap_err();

        assert!(error.to_string().contains("different decision-work ledger"));
        assert_eq!(owner.value()["actor"], "alice");
        assert_eq!(retained_http_bytes(&owner_ledger), retained);
        assert_eq!(retained_http_bytes(&foreign_ledger), 0);
        assert!(!owner_ledger.has_open_permits());
        assert!(!foreign_ledger.has_open_permits());

        owner.release(&mut owner_ledger).unwrap();
        assert!(owner.body.is_none());
        assert!(owner.retained_bytes.is_none());
        assert_eq!(retained_http_bytes(&owner_ledger), 0);
    }

    #[test]
    fn request_body_owner_release_failure_keeps_scalar_for_retry_after_payload_drop() {
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "request body release retry",
        )
        .unwrap();
        let body = json!({"actor": "alice"});
        let retained = compact_json_size(&body).unwrap();
        let mut owner = HttpRequestBodyOwnership::new(body, retained, &ledger);

        let error = owner.release(&mut ledger).unwrap_err();

        assert!(error.to_string().contains("release HTTP request body"));
        assert!(owner.body.is_none());
        assert_eq!(owner.retained_bytes, Some(retained));
        assert_eq!(retained_http_bytes(&ledger), 0);
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestBody,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();

        owner.release(&mut ledger).unwrap();
        owner.release(&mut ledger).unwrap();
        assert!(owner.retained_bytes.is_none());
        assert_eq!(retained_http_bytes(&ledger), 0);
        assert!(!ledger.has_open_permits());
    }

    #[test]
    fn mutation_intent_foreign_release_is_no_change_and_release_retries() {
        let body = json!({"actor": "alice"});
        let retained = compact_json_size(&body).unwrap();
        let mut owner_ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "mutation intent owner",
        )
        .unwrap();
        owner_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();
        let mut foreign = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "foreign mutation intent",
        )
        .unwrap();
        let mut intent = HttpMutationRouteIntent {
            ledger_identity: owner_ledger.identity(),
            source_routing: None,
            routing: None,
            request: Some(body.clone()),
            request_retained_bytes: Some(retained),
            derived_retained_bytes: 0,
            derived: HttpMutationDerivedIntent::None,
        };

        let error = intent.release(&mut foreign).unwrap_err();
        assert!(error.to_string().contains("different decision-work ledger"));
        assert_eq!(intent.request(), &body);
        assert_eq!(retained_http_bytes(&owner_ledger), retained);
        assert_eq!(retained_http_bytes(&foreign), 0);

        owner_ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            )
            .unwrap();
        let error = intent.release(&mut owner_ledger).unwrap_err();
        assert!(error
            .to_string()
            .contains("release HTTP mutation route intent"));
        assert!(intent.request.is_none());
        assert_eq!(intent.request_retained_bytes, Some(retained));
        owner_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();
        intent.release(&mut owner_ledger).unwrap();
        intent.release(&mut owner_ledger).unwrap();
        assert_eq!(retained_http_bytes(&owner_ledger), 0);
    }

    fn upload_overlay_handoff_fixture(
        label: &str,
    ) -> (
        HttpMutationRouteIntent,
        codefire_core::DecisionWorkLedger,
        Value,
        u64,
        u64,
    ) {
        let document = codefire_store::ObjectRecordDocument {
            record: codefire_store::object_record("policy", json!({"version": 1, "policy": {}}))
                .unwrap(),
            extensions: serde_json::Map::new(),
        };
        let record = codefire_store::object_record_document_value(&document).unwrap();
        let document_bytes =
            codefire_store::compact_object_record_document_size(&document).unwrap();
        let request = json!({
            "actor": "uploader",
            "branch": "local-main",
            "head": "CF-COMMIT-fixture",
            "idempotency_key": label,
            "objects": [record]
        });
        let request_bytes = compact_json_size(&request).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            label,
        )
        .unwrap();
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RequestBody,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    request_bytes,
                ),
            )
            .unwrap();
        let intent = HttpMutationRouteIntent {
            ledger_identity: ledger.identity(),
            source_routing: None,
            routing: None,
            request: Some(request.clone()),
            request_retained_bytes: Some(request_bytes),
            derived_retained_bytes: 0,
            derived: HttpMutationDerivedIntent::None,
        };
        (intent, ledger, request, request_bytes, document_bytes)
    }

    #[test]
    fn upload_overlay_move_reserves_destination_slots_at_exact_temporary_boundary() {
        let slot_bytes =
            u64::try_from(std::mem::size_of::<codefire_store::ObjectRecordDocument>()).unwrap();
        for delta in [0_u64, 1] {
            let (mut intent, mut ledger, original, request_bytes, document_bytes) =
                upload_overlay_handoff_fixture(&format!("upload overlay slots +{delta}"));
            let temporary_limit = ledger
                .profile()
                .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
                .unwrap()
                .limit;
            let filler = temporary_limit - slot_bytes + delta;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, filler),
                )
                .unwrap();
            let deadline = HttpRequestDeadline::from_profile(ledger.profile());
            let result = intent.prepare_upload_overlay_input(ledger, document_bytes, &deadline);
            let mut ledger = if delta == 0 {
                let input = match result {
                    Ok(input) => input,
                    Err(_) => panic!("the exact destination reservation must succeed"),
                };
                assert!(intent
                    .request()
                    .get("objects")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty));
                let temp = test_storage_root("upload-overlay-input-recovery");
                fs::create_dir_all(&temp).unwrap();
                let not_a_directory = temp.join("not-a-directory");
                fs::write(&not_a_directory, b"file").unwrap();
                let error = match codefire_store::begin_sealed_commit_overlay_validation_session_with_recoverable_ledger(
                        &not_a_directory,
                        input,
                    ) {
                    Ok(_) => panic!("a regular file cannot bind an overlay validation session"),
                    Err(error) => error,
                };
                let (_, recovered) = error.discard_documents_and_recover_ledger();
                let _ = fs::remove_dir_all(temp);
                assert_eq!(
                    retained_http_bytes(&recovered),
                    request_bytes - document_bytes
                );
                recovered
            } else {
                let failure = result.unwrap_err();
                let (error, ledger) = failure.into_parts();
                assert!(matches!(error, CliError::ResourceBudget(_)));
                assert_eq!(intent.request(), &original);
                assert_eq!(retained_http_bytes(&ledger), request_bytes);
                ledger
            };
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
                filler
            );
            intent.release(&mut ledger).unwrap();
            ledger
                .release(codefire_core::DecisionWorkDimension::TemporaryBytes, filler)
                .unwrap();
            assert!(ledger
                .snapshot()
                .current
                .values()
                .all(|amount| *amount == 0));
        }
    }

    #[test]
    fn upload_overlay_move_rejects_foreign_and_expired_handoffs_pre_effect() {
        let (mut intent, mut owner, original, request_bytes, document_bytes) =
            upload_overlay_handoff_fixture("foreign upload overlay handoff");
        let foreign = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "foreign upload overlay handoff ledger",
        )
        .unwrap();
        let foreign_before = foreign.snapshot().clone();
        let deadline = HttpRequestDeadline::from_profile(foreign.profile());

        let failure = intent
            .prepare_upload_overlay_input(foreign, document_bytes, &deadline)
            .unwrap_err();
        let (error, foreign) = failure.into_parts();
        assert!(error.to_string().contains("different decision-work ledger"));
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(intent.request(), &original);
        assert_eq!(retained_http_bytes(&owner), request_bytes);

        let expired = HttpRequestDeadline {
            started_at: Instant::now(),
            limit: Duration::ZERO,
            limit_ms: 0,
        };
        let owner_before = owner.snapshot().clone();
        let failure = intent
            .prepare_upload_overlay_input(owner, document_bytes, &expired)
            .unwrap_err();
        let (error, recovered) = failure.into_parts();
        assert!(matches!(
            error,
            CliError::RequestDeadlineExceeded {
                phase: "upload overlay destination reservation",
                ..
            }
        ));
        owner = recovered;
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(intent.request(), &original);
        intent.release(&mut owner).unwrap();
        assert_eq!(retained_http_bytes(&owner), 0);
    }

    #[test]
    fn upload_derived_intent_accepts_exact_and_rejects_first_overflow() {
        const RETAINED_LIMIT: u64 = 32 * 1024 * 1024;
        let storage_root = PathBuf::from("/tmp/codefire-derived-boundary");
        let body = json!({
            "head": "CF-COMMIT-fixture",
            "branch": "large-local-branch",
            "actor": "uploader",
        });
        let request_retained = compact_json_size(&body).unwrap();
        let derived_retained = sum_http_owner_bytes([
            http_project_root_logical_bytes(&storage_root, "org", "app").unwrap(),
            logical_string_owner_bytes("main").unwrap(),
            logical_string_owner_bytes("CF-COMMIT-fixture").unwrap(),
            logical_string_owner_bytes("large-local-branch").unwrap(),
            logical_string_owner_bytes("uploader").unwrap(),
            http_request_target_logical_bytes("org", "app", "upload", Some("main"), None).unwrap(),
        ])
        .unwrap();
        for delta in [0_u64, 1] {
            let profile = codefire_core::DecisionWorkProfile::http_v1();
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile,
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                format!("upload derived boundary +{delta}"),
            )
            .unwrap();
            let filler = RETAINED_LIMIT
                .checked_sub(request_retained + derived_retained)
                .unwrap()
                + delta;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        request_retained + filler,
                    ),
                )
                .unwrap();
            let mut intent = HttpMutationRouteIntent {
                ledger_identity: ledger.identity(),
                source_routing: None,
                routing: None,
                request: Some(body.clone()),
                request_retained_bytes: Some(request_retained),
                derived_retained_bytes: 0,
                derived: HttpMutationDerivedIntent::None,
            };

            let result =
                intent.prepare_upload_initial(&storage_root, "org", "app", "main", &mut ledger);
            if delta == 0 {
                result.unwrap();
                assert_eq!(retained_http_bytes(&ledger), RETAINED_LIMIT);
                assert!(matches!(
                    intent.derived,
                    HttpMutationDerivedIntent::Upload(_)
                ));
            } else {
                assert!(matches!(result.unwrap_err(), CliError::ResourceBudget(_)));
                assert_eq!(retained_http_bytes(&ledger), request_retained + filler);
                assert!(matches!(intent.derived, HttpMutationDerivedIntent::None));
            }
            intent.release(&mut ledger).unwrap();
            ledger
                .release(codefire_core::DecisionWorkDimension::RetainedBytes, filler)
                .unwrap();
            assert_eq!(retained_http_bytes(&ledger), 0);
            assert!(!ledger.has_open_permits());
        }
    }

    #[test]
    fn store_binding_structure_rejects_live_routing_and_incomplete_payloads() {
        let upload = HttpUploadDerivedIntent {
            project_root: PathBuf::from("/tmp/project"),
            remote_branch: "main".to_string(),
            head: "CF-COMMIT-fixture".to_string(),
            local_branch: "local".to_string(),
            actor: "uploader".to_string(),
            request_target: "org/app/branches/main/upload".to_string(),
            active_project_root: None,
            objects: None,
            publication_request: None,
            response: None,
        };
        assert!(!upload_store_structure_is_complete(false, &upload));
        assert!(!upload_store_structure_is_complete(true, &upload));

        let apply = HttpApplyDerivedIntent {
            project_root: PathBuf::from("/tmp/project"),
            mr_path: PathBuf::from("/tmp/project/merge_requests/MR.json"),
            merge_request_id: "MR".to_string(),
            actor: "reviewer".to_string(),
            request_target: "org/app/merge-requests/MR/apply".to_string(),
            identity: None,
        };
        assert!(!apply_store_structure_is_complete(false, &apply));
        assert!(!apply_store_structure_is_complete(true, &apply));
    }

    #[test]
    fn missing_bootstrap_cleanup_follows_route_intent_owner_cancellation() {
        // Store's detach-failure injector is crate-private, so compose the
        // exact post-recovery owners at this HTTP seam: route intent first,
        // then the independently owned bootstrap permits and hidden stage.
        let storage_root = test_storage_root("bootstrap-intent-cancel-order");
        let project_root = http_project_root(&storage_root, "org", "app");
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "bootstrap intent cancellation",
        )
        .unwrap();
        let preparation = RemoteProjectGate::prepare_exclusive_or_bootstrap_with_ledger(
            &project_root,
            &LockOptions::default(),
            &mut ledger,
            BootstrapStageFootprint::new(0, 0),
        )
        .unwrap();
        assert!(matches!(
            &preparation,
            RemoteProjectExclusivePreparation::Unpublished(_)
        ));
        let owns_prepublication_permits = match &preparation {
            RemoteProjectExclusivePreparation::Unpublished(prepared) => {
                prepared.owns_prepublication_permits(&ledger)
            }
            RemoteProjectExclusivePreparation::Existing(_) => false,
        };
        assert!(owns_prepublication_permits);
        assert!(ledger.has_open_permits());

        let request = json!({"actor": "uploader"});
        let retained = compact_json_size(&request).unwrap();
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();
        let mut intent = Some(HttpMutationRouteIntent {
            ledger_identity: ledger.identity(),
            source_routing: None,
            routing: None,
            request: Some(request),
            request_retained_bytes: Some(retained),
            derived_retained_bytes: 0,
            derived: HttpMutationDerivedIntent::None,
        });
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_route_intent(&mut intent).unwrap();
        let mut preparation = Some(preparation);

        let recovered = cancel_http_upload_mutation_owner(&mut owner, &preparation);
        assert_eq!(retained_http_bytes(&recovered), 0);
        assert!(recovered.has_open_permits());
        let mut ledger = Some(recovered);
        let mut prepared = None;
        let mut context = http_route_context_for_test(&mut ledger, &mut prepared, &deadline);
        cancel_failed_http_upload_bootstrap(&mut preparation, &mut context).unwrap();

        let ledger = ledger.as_ref().unwrap();
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(preparation.is_none());
        assert!(!project_root.exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_request_content_length_is_single_strict_unsigned_decimal() {
        for request in [
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: -1\r\n\r\n",
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: nope\r\n\r\n",
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length:\r\n\r\n",
        ] {
            let error = read_http_request(&mut Cursor::new(request.as_bytes().to_vec())).unwrap_err();
            assert!(matches!(error, CliError::Usage(_)));
            assert!(error.to_string().contains("Content-Length"));
        }

        let duplicate = b"POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: 0\r\ncontent-length: 0\r\n\r\n";
        let error = read_http_request(&mut Cursor::new(duplicate.to_vec())).unwrap_err();
        assert!(matches!(error, CliError::Usage(_)));
        assert!(error.to_string().contains("duplicate Content-Length"));

        let overflow = b"POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: 999999999999999999999999999999999\r\n\r\n";
        let error = read_http_request(&mut Cursor::new(overflow.to_vec())).unwrap_err();
        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert_eq!(error.diagnostic()["dimension"], "request_body_bytes");
    }

    #[test]
    fn invalid_content_length_wire_response_has_no_dispatch_effect() {
        let storage_root = test_storage_root("invalid-content-length-no-effect");
        let request = b"POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n";
        let mut stream = Cursor::new(request.to_vec());

        handle_http_stream(&storage_root, &mut stream).unwrap();

        let wire = String::from_utf8(stream.into_inner()).unwrap();
        assert!(wire.contains("HTTP/1.1 400 Bad Request"));
        assert!(wire.contains("duplicate Content-Length"));
        assert!(!storage_root.join("org").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn failed_dispatch_never_publishes_its_prepared_success_response() {
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        let prepared =
            prepare_http_json_with_ledger(200, &json!({"ok": true}), &mut ledger).unwrap();
        let deadline = HttpRequestDeadline::from_profile(ledger.profile());
        let mut stream = Cursor::new(Vec::new());

        write_http_dispatch_outcome(
            &mut stream,
            Err(CliError::InvalidRepository(
                "publication failed after response planning".to_string(),
            )),
            Some(prepared),
            &mut ledger,
            &deadline,
        )
        .unwrap();

        let wire = String::from_utf8(stream.into_inner()).unwrap();
        assert!(!wire.contains("HTTP/1.1 200"));
        assert!(!wire.contains("\"ok\":true"));
        assert!(wire.contains("publication failed after response planning"));
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    #[test]
    fn committed_prepared_response_is_not_reclassified_by_expired_deadline() {
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http upload",
        )
        .unwrap();
        let payload = json!({"head": "CF-COMMIT-committed"});
        let prepared = prepare_http_json_with_ledger(200, &payload, &mut ledger).unwrap();
        let deadline = HttpRequestDeadline {
            started_at: Instant::now() - Duration::from_millis(30_001),
            limit: Duration::from_millis(30_000),
            limit_ms: 30_000,
        };
        let mut stream = Cursor::new(Vec::new());

        write_http_dispatch_outcome(
            &mut stream,
            Ok(HttpDispatchResponse {
                status: 200,
                payload,
                payload_owner: None,
            }),
            Some(prepared),
            &mut ledger,
            &deadline,
        )
        .unwrap();

        let wire = String::from_utf8(stream.into_inner()).unwrap();
        assert!(wire.contains("HTTP/1.1 200 OK"));
        assert!(!wire.contains("request_deadline_exceeded"));
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::Complete)
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    fn mutation_response_ledger(
        command: &str,
    ) -> (codefire_core::DecisionWorkLedger, HttpRequestDeadline) {
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            command,
        )
        .unwrap();
        (ledger, deadline)
    }

    fn retained_http_bytes(ledger: &codefire_core::DecisionWorkLedger) -> u64 {
        ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default()
    }

    fn prepared_apply_transaction_for_test<'project>(
        gate: &'project RemoteProjectGate,
        project_root: &Path,
        head: &str,
        branch: &str,
    ) -> PreparedHttpMutationTransaction<'project> {
        let stable_project_id = remote_project_stable_id(project_root).unwrap();
        let request = apply_publication(
            "fixture-actor",
            None,
            &ApplyPublicationIdentity {
                stable_project_id: &stable_project_id,
                merge_request_id: "fixture-mr",
                subject_digest: "fixture-subject",
                target_branch: branch,
                source_head: head,
                target_head_at_request: head,
            },
        );
        let result = apply_result("fixture-mr", branch, head);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "prepared apply transaction fixture",
        )
        .unwrap();
        let mut session =
            codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
                &project_root.join("objects"),
                ledger,
            )
            .unwrap();
        session.validate_commit(head).unwrap();
        let roots = vec![head.to_string()];
        let bound = session.bind_prepublication_validation(&roots).unwrap();
        let mut owner = PreparedHttpMutationTransaction::from_bound_validation(bound).unwrap();
        let mut nonce =
            Some(crate::signatures::PendingRequestNonce::recovered_from_matching_publication());
        owner
            .plan_publication(gate, project_root, &request, &mut nonce, &result, |_, _| {
                Ok(vec![AuthorityWrite {
                    authority: RemoteAuthority::Branch(branch.to_string()),
                    before: None,
                    after: json!({
                        "version": 1,
                        "name": branch,
                        "head": head,
                    }),
                }])
            })
            .unwrap();
        owner
    }

    #[test]
    fn dual_response_success_selection_transfers_exact_body_owner() {
        let (mut ledger, deadline) = mutation_response_ledger("dual response success");
        let payload = json!({"status": "published", "head": "CF-COMMIT-success"});
        let mut raw =
            prepare_mutation_responses_with_deadline(201, &payload, &mut ledger, &deadline)
                .unwrap();
        let success_bytes = raw.success.as_ref().unwrap().body_bytes;
        let durable_bytes = raw.durable.as_ref().unwrap().body_bytes;
        assert_eq!(retained_http_bytes(&ledger), success_bytes + durable_bytes);
        assert!(!ledger
            .snapshot()
            .cumulative
            .contains_key(&codefire_core::DecisionWorkDimension::ResponseBytes));

        ledger.seal_prepublication().unwrap();
        let mut prepared = None;
        raw.select_success(&mut ledger, &mut prepared).unwrap();

        let selected = prepared.as_ref().unwrap();
        assert_eq!(selected.status, 201);
        assert_eq!(
            serde_json::from_slice::<Value>(&selected.body).unwrap(),
            payload
        );
        assert_eq!(retained_http_bytes(&ledger), success_bytes);
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::ResponseBytes],
            success_bytes
        );
        assert_eq!(
            ledger.snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::ResponseHeaderBytes],
            http_json_header_len(201, selected.body.len()).unwrap()
        );
        release_prepared_http_response(
            &mut ledger,
            prepared.take().unwrap(),
            "release selected test success response",
        )
        .unwrap();
        assert_eq!(retained_http_bytes(&ledger), 0);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn dual_response_durable_selection_transfers_fixed_503_owner() {
        let (mut ledger, deadline) = mutation_response_ledger("dual response durable");
        let mut raw = prepare_mutation_responses_with_deadline(
            200,
            &json!({"status": "published"}),
            &mut ledger,
            &deadline,
        )
        .unwrap();
        let success_bytes = raw.success.as_ref().unwrap().body_bytes;
        let durable_bytes = raw.durable.as_ref().unwrap().body_bytes;
        assert_eq!(retained_http_bytes(&ledger), success_bytes + durable_bytes);

        ledger.seal_prepublication().unwrap();
        let mut prepared = None;
        PreparedMutationResponsesOwner::new(&mut raw, &mut ledger, &mut prepared)
            .unwrap()
            .select_durable()
            .unwrap();

        let selected = prepared.as_ref().unwrap();
        assert_eq!(selected.status, 503);
        assert_eq!(
            serde_json::from_slice::<Value>(&selected.body).unwrap(),
            json!({
                "error": "the durable mutation completed without a confirmed response; retry the same request",
                "kind": "retry_same_request",
                "retry_same_request": true,
            })
        );
        assert_eq!(retained_http_bytes(&ledger), durable_bytes);
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::ResponseBytes],
            durable_bytes
        );
        release_prepared_http_response(
            &mut ledger,
            prepared.take().unwrap(),
            "release selected test durable response",
        )
        .unwrap();
        assert_eq!(retained_http_bytes(&ledger), 0);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn dual_response_cancel_and_owner_drop_release_both_bodies_and_permits() {
        for (command, explicit_cancel) in [
            ("dual response explicit cancel", true),
            ("dual response drop cancel", false),
        ] {
            let (mut ledger, deadline) = mutation_response_ledger(command);
            let mut raw = prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "planned"}),
                &mut ledger,
                &deadline,
            )
            .unwrap();
            assert!(retained_http_bytes(&ledger) > 0);
            let mut prepared = None;
            if explicit_cancel {
                PreparedMutationResponsesOwner::new(&mut raw, &mut ledger, &mut prepared)
                    .unwrap()
                    .cancel()
                    .unwrap();
            } else {
                drop(
                    PreparedMutationResponsesOwner::new(&mut raw, &mut ledger, &mut prepared)
                        .unwrap(),
                );
            }
            assert!(prepared.is_none());
            assert_eq!(retained_http_bytes(&ledger), 0);
            assert!(ledger.snapshot().cumulative.is_empty());
            ledger
                .finish(codefire_core::DecisionWorkTerminalReason::Complete)
                .unwrap();
        }
    }

    #[test]
    fn dual_response_rejects_foreign_ledger_without_consuming_raw_owner() {
        let (mut owner_ledger, deadline) = mutation_response_ledger("dual response owner ledger");
        let (mut foreign_ledger, _) = mutation_response_ledger("dual response foreign ledger");
        let mut raw = prepare_mutation_responses_with_deadline(
            200,
            &json!({"status": "planned"}),
            &mut owner_ledger,
            &deadline,
        )
        .unwrap();
        let owner_before = owner_ledger.snapshot().clone();
        let foreign_before = foreign_ledger.snapshot().clone();
        let mut prepared = None;

        assert!(
            PreparedMutationResponsesOwner::new(&mut raw, &mut foreign_ledger, &mut prepared,)
                .is_err()
        );
        assert_eq!(owner_ledger.snapshot(), &owner_before);
        assert_eq!(foreign_ledger.snapshot(), &foreign_before);

        let select_error = raw
            .select_success(&mut foreign_ledger, &mut prepared)
            .unwrap_err();
        assert!(select_error
            .to_string()
            .contains("different decision-work ledger"));
        assert_eq!(owner_ledger.snapshot(), &owner_before);
        assert_eq!(foreign_ledger.snapshot(), &foreign_before);
        assert!(prepared.is_none());

        let cancel_error = raw.cancel(&mut foreign_ledger).unwrap_err();
        assert!(cancel_error
            .to_string()
            .contains("different decision-work ledger"));
        assert_eq!(owner_ledger.snapshot(), &owner_before);
        assert_eq!(foreign_ledger.snapshot(), &foreign_before);
        assert!(raw.success.is_some());
        assert!(raw.durable.is_some());
        assert!(raw.response_permit.is_some());
        assert!(raw.header_permit.is_some());

        raw.cancel(&mut owner_ledger).unwrap();
        assert_eq!(retained_http_bytes(&owner_ledger), 0);
        owner_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        foreign_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn dual_response_pair_consume_is_atomic_when_header_actual_is_invalid() {
        let (mut ledger, deadline) = mutation_response_ledger("dual response atomic pair");
        let mut raw = prepare_mutation_responses_with_deadline(
            200,
            &json!({"status": "planned"}),
            &mut ledger,
            &deadline,
        )
        .unwrap();
        raw.success.as_mut().unwrap().header_work = Some(http_response_work_reservation(
            codefire_core::DecisionWorkPhase::ResponseHeaders,
            codefire_core::DecisionWorkDimension::ResponseHeaderBytes,
            raw.header_upper + 1,
        ));
        ledger.seal_prepublication().unwrap();
        let before = ledger.snapshot().clone();
        let mut prepared = None;

        let error = raw.select_success(&mut ledger, &mut prepared).unwrap_err();
        assert!(!error.to_string().is_empty());
        assert_eq!(ledger.snapshot(), &before);
        assert!(prepared.is_none());
        assert!(raw.response_permit.is_some());
        assert!(raw.header_permit.is_some());

        raw.cancel(&mut ledger).unwrap();
        assert_eq!(retained_http_bytes(&ledger), 0);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn request_wide_mutation_owner_drop_cancels_midway_response_owners() {
        let (mut ledger, deadline) = mutation_response_ledger("request-wide midway drop");
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "planned"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_responses(&mut responses).unwrap();
        assert!(responses.is_none());
        let error = owner.validate_http_handoff().unwrap_err();
        assert!(error.to_string().contains("uncommitted resources"));
        assert!(owner.responses.is_some());

        // Drop runs the same ordered cancellation path as explicit abort. Its
        // fail-closed invariants prove current ownership is zero and no permit
        // remains before the ledger itself is destroyed, including release
        // builds.
        drop(owner);
    }

    #[test]
    fn request_wide_handoff_validation_is_non_consuming_until_exact() {
        let (mut ledger, deadline) = mutation_response_ledger("request-wide exact handoff");
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, 1)
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, 1),
            )
            .unwrap();
        let unrelated_permit = ledger
            .reserve(http_response_work_reservation(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkDimension::ResponseBytes,
                1,
            ))
            .unwrap();
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "exact"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_responses(&mut responses).unwrap();
        owner.revalidate_and_seal(|| false).unwrap();
        owner.commit_and_select_success().unwrap();

        assert!(owner.validate_http_handoff().is_err());
        owner
            .ledger
            .as_mut()
            .unwrap()
            .release(codefire_core::DecisionWorkDimension::TemporaryBytes, 1)
            .unwrap();
        assert!(owner.validate_http_handoff().is_err());
        owner
            .ledger
            .as_mut()
            .unwrap()
            .release(codefire_core::DecisionWorkDimension::RetainedBytes, 1)
            .unwrap();
        assert!(owner.validate_http_handoff().is_err());
        owner
            .ledger
            .as_mut()
            .unwrap()
            .cancel_reserved(&unrelated_permit)
            .unwrap();
        owner.validate_http_handoff().unwrap();

        let (mut ledger, response) = owner.into_http_parts_after_validation();
        assert_eq!(response.status, 200);
        release_prepared_http_response(&mut ledger, response, "release exact handoff response")
            .unwrap();
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn visibility_terminal_separates_effect_commit_from_success_selection() {
        let (mut ledger, deadline) = mutation_response_ledger("visibility terminal sequencing");
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "published"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_responses(&mut responses).unwrap();

        let error = owner.validate_visibility_terminal().unwrap_err();
        assert!(error.to_string().contains("requires a sealed ledger"));

        owner.revalidate_and_seal(|| false).unwrap();
        owner.commit_effects_after_seal().unwrap();
        owner.validate_visibility_terminal().unwrap();
        let mut terminal = owner.into_visibility_terminal_after_validation();

        let error = terminal.validate_http_handoff().unwrap_err();
        assert!(error.to_string().contains("unselected responses"));
        terminal.select_success().unwrap();
        terminal.validate_http_handoff().unwrap();

        let (mut ledger, response) = terminal.into_http_parts_after_validation();
        assert_eq!(response.status, 200);
        release_prepared_http_response(
            &mut ledger,
            response,
            "release visibility-terminal response",
        )
        .unwrap();
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn request_wide_take_ledger_requires_successful_explicit_cancel() {
        let (ledger, _) = mutation_response_ledger("request-wide explicit cancel");
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);

        let error = owner.take_ledger().unwrap_err();
        assert!(error.to_string().contains("complete cancellation"));
        assert!(owner.ledger.is_some());

        owner.cancel_all().unwrap();
        let mut ledger = owner.take_ledger().unwrap();
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn request_wide_drop_abandons_external_stage_unlink_failure() {
        let storage_root = test_storage_root("mutation-drop-abandon-stage");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let record = codefire_store::object_record(
            "blob",
            json!({"type": "blob", "version": 1, "content": "drop-abandon"}),
        )
        .unwrap();
        let object_id = record.object_id.clone();
        let document = codefire_store::ObjectRecordDocument {
            record,
            extensions: serde_json::Map::new(),
        };
        let (mut ledger, deadline) = mutation_response_ledger("drop abandon Store stage");
        let mut batch = Some(
            codefire_store::PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
                &objects,
                vec![document],
                &mut ledger,
                || false,
            )
            .unwrap(),
        );
        let stage_path = fs::read_dir(objects.join("blobs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".tmp"))
            })
            .expect("prepared object batch must create a private stage");
        fs::remove_file(&stage_path).unwrap();
        fs::create_dir(&stage_path).unwrap();
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "unused"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_object_batch(&mut batch).unwrap();
        owner.attach_responses(&mut responses).unwrap();

        // remove_file(stage) now fails because an external actor replaced the
        // private file with a directory. Drop seals, abandons the non-authority
        // stage for bounded recovery, retries cancellation, and returns normally.
        drop(owner);

        assert!(stage_path.is_dir());
        assert!(codefire_store::object_record_path(&objects, &object_id).is_none());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn request_wide_mutation_owner_rejects_foreign_response_before_transfer() {
        let (owner_ledger, _) = mutation_response_ledger("request-wide owner ledger");
        let (mut foreign_ledger, deadline) =
            mutation_response_ledger("request-wide foreign ledger");
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "planned"}),
                &mut foreign_ledger,
                &deadline,
            )
            .unwrap(),
        );
        let foreign_before = foreign_ledger.snapshot().clone();
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(owner_ledger);

        let error = owner.attach_responses(&mut responses).unwrap_err();
        assert!(error.to_string().contains("different decision-work ledger"));
        assert!(
            responses.is_some(),
            "foreign raw owner must stay with caller"
        );
        assert_eq!(foreign_ledger.snapshot(), &foreign_before);

        owner.cancel_all().unwrap();
        let mut owner_ledger = owner.take_ledger().unwrap();
        assert_eq!(retained_http_bytes(&owner_ledger), 0);
        owner_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();

        responses
            .as_mut()
            .unwrap()
            .cancel(&mut foreign_ledger)
            .unwrap();
        drop(responses.take());
        assert_eq!(retained_http_bytes(&foreign_ledger), 0);
        foreign_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn request_wide_mutation_owner_preseal_failure_releases_current_and_permits() {
        let (mut ledger, deadline) = mutation_response_ledger("request-wide preseal failure");
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "planned"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_responses(&mut responses).unwrap();
        owner.requires_validation_proof = true;

        let error = owner.revalidate_and_seal(|| false).unwrap_err();
        assert!(error
            .to_string()
            .contains("lost its prepublication validation proof"));
        owner.cancel_all().unwrap();
        let mut ledger = owner.take_ledger().unwrap();
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn request_wide_mutation_owner_rejects_missing_terminal_fields_before_seal() {
        let (mut ledger, deadline) = mutation_response_ledger("missing publication preseal");
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "planned"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.requires_publication = true;
        owner.requires_responses = true;
        owner.attach_responses(&mut responses).unwrap();
        let error = owner.revalidate_and_seal(|| false).unwrap_err();
        assert!(error
            .to_string()
            .contains("lost its prepared JSON publication"));
        assert!(!owner.ledger().is_prepublication_sealed());
        owner.cancel_all().unwrap();
        let mut ledger = owner.take_ledger().unwrap();
        assert_eq!(retained_http_bytes(&ledger), 0);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();

        let (ledger, _) = mutation_response_ledger("missing responses preseal");
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.requires_responses = true;
        let error = owner.revalidate_and_seal(|| false).unwrap_err();
        assert!(error
            .to_string()
            .contains("lost its prepared terminal responses"));
        assert!(!owner.ledger().is_prepublication_sealed());
        owner.cancel_all().unwrap();
        let mut ledger = owner.take_ledger().unwrap();
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn request_wide_mutation_owner_batch_durable_error_selects_retry_response() {
        let storage_root = test_storage_root("mutation-batch-durable");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let record = codefire_store::object_record(
            "blob",
            json!({"type": "blob", "version": 1, "content": "prepared"}),
        )
        .unwrap();
        let object_id = record.object_id.clone();
        let document = codefire_store::ObjectRecordDocument {
            record,
            extensions: serde_json::Map::new(),
        };
        let (mut ledger, deadline) = mutation_response_ledger("batch durable response");
        let mut batch = Some(
            codefire_store::PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
                &objects,
                vec![document],
                &mut ledger,
                || false,
            )
            .unwrap(),
        );
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "published"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_object_batch(&mut batch).unwrap();
        owner.attach_responses(&mut responses).unwrap();
        owner.revalidate_and_seal(|| false).unwrap();

        let destination = objects.join("blobs").join(format!("{object_id}.json"));
        fs::write(&destination, b"post-seal collision").unwrap();
        let error = owner.commit_and_select_success().unwrap_err();
        assert!(error.to_string().contains("retry the same request"));
        assert!(owner.object_batch.is_some());
        owner.select_durable_after_commit_error().unwrap();
        owner.validate_http_handoff().unwrap();
        let (mut ledger, response) = owner.into_http_parts_after_validation();
        assert_eq!(response.status, 503);
        release_prepared_http_response(
            &mut ledger,
            response,
            "release durable batch-error response",
        )
        .unwrap();
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn request_wide_durable_selection_abandons_external_stage_unlink_failure() {
        let storage_root = test_storage_root("mutation-durable-abandon-stage");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let record = codefire_store::object_record(
            "blob",
            json!({"type": "blob", "version": 1, "content": "durable-abandon"}),
        )
        .unwrap();
        let object_id = record.object_id.clone();
        let document = codefire_store::ObjectRecordDocument {
            record,
            extensions: serde_json::Map::new(),
        };
        let (mut ledger, deadline) = mutation_response_ledger("durable abandon Store stage");
        let mut batch = Some(
            codefire_store::PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
                &objects,
                vec![document],
                &mut ledger,
                || false,
            )
            .unwrap(),
        );
        let stage_path = fs::read_dir(objects.join("blobs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".tmp"))
            })
            .expect("prepared object batch must create a private stage");
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &json!({"status": "unused"}),
                &mut ledger,
                &deadline,
            )
            .unwrap(),
        );
        let mut owner = PreparedHttpMutationTransaction::from_ledger_for_test(ledger);
        owner.attach_object_batch(&mut batch).unwrap();
        owner.attach_responses(&mut responses).unwrap();
        owner.revalidate_and_seal(|| false).unwrap();
        fs::remove_file(&stage_path).unwrap();
        fs::create_dir(&stage_path).unwrap();

        owner.select_durable_after_commit_error().unwrap();
        assert!(owner.object_batch.is_none());
        owner.validate_http_handoff().unwrap();
        let (mut ledger, response) = owner.into_http_parts_after_validation();
        assert_eq!(response.status, 503);
        release_prepared_http_response(
            &mut ledger,
            response,
            "release abandoned-stage durable response",
        )
        .unwrap();
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        assert!(stage_path.is_dir());
        assert!(codefire_store::object_record_path(&objects, &object_id).is_none());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn request_wide_mutation_owner_real_binding_plans_and_drops_without_effect() {
        let storage_root = test_storage_root("mutation-real-binding-drop");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let head = write_http_test_commit(&remote_dirs(&project_root).objects);
        let gate =
            RemoteProjectGate::acquire_exclusive(&project_root, &LockOptions::default()).unwrap();
        let mut owner =
            prepared_apply_transaction_for_test(&gate, &project_root, &head, "planned-drop");
        assert!(owner.validation_proof.is_some());
        assert!(owner.publication.is_some());
        let deadline = HttpRequestDeadline::from_profile(owner.ledger().profile());
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &apply_result("fixture-mr", "planned-drop", &head),
                owner.ledger.as_mut().unwrap(),
                &deadline,
            )
            .unwrap(),
        );
        owner.attach_responses(&mut responses).unwrap();

        drop(owner);
        assert!(!RemoteAuthority::Branch("planned-drop".to_string())
            .path(&project_root)
            .unwrap()
            .exists());
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn request_wide_mutation_owner_json_durable_error_keeps_owner_and_selects_retry() {
        let storage_root = test_storage_root("mutation-json-durable");
        let project_root = http_project_root(&storage_root, "org", "app");
        ensure_remote_project_layout(&project_root).unwrap();
        let head = write_http_test_commit(&remote_dirs(&project_root).objects);
        let gate =
            RemoteProjectGate::acquire_exclusive(&project_root, &LockOptions::default()).unwrap();
        let mut owner =
            prepared_apply_transaction_for_test(&gate, &project_root, &head, "durable-json");
        let deadline = HttpRequestDeadline::from_profile(owner.ledger().profile());
        let mut responses = Some(
            prepare_mutation_responses_with_deadline(
                200,
                &apply_result("fixture-mr", "durable-json", &head),
                owner.ledger.as_mut().unwrap(),
                &deadline,
            )
            .unwrap(),
        );
        owner.attach_responses(&mut responses).unwrap();
        owner.revalidate_and_seal(|| false).unwrap();

        let pending = RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap();
        fs::write(&pending, b"foreign\n").unwrap();
        let error = owner.commit_and_select_success().unwrap_err();
        assert!(!error.to_string().is_empty());
        assert!(owner.publication.is_some());
        owner.select_durable_after_commit_error().unwrap();
        owner.validate_http_handoff().unwrap();
        let (mut ledger, response) = owner.into_http_parts_after_validation();
        assert_eq!(response.status, 503);
        release_prepared_http_response(
            &mut ledger,
            response,
            "release durable JSON-error response",
        )
        .unwrap();
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        assert_eq!(fs::read(&pending).unwrap(), b"foreign\n");
        assert!(!RemoteAuthority::Branch("durable-json".to_string())
            .path(&project_root)
            .unwrap()
            .exists());
        fs::remove_file(pending).unwrap();
        drop(gate);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "raw prepared mutation responses were dropped")]
    fn dual_response_raw_drop_detects_unreleased_resources() {
        let (mut ledger, deadline) = mutation_response_ledger("dual response raw drop");
        let raw = prepare_mutation_responses_with_deadline(
            200,
            &json!({"status": "planned"}),
            &mut ledger,
            &deadline,
        )
        .unwrap();
        drop(raw);
    }

    #[test]
    fn dual_response_uses_max_permit_and_rejects_limit_plus_one_pre_mutation() {
        let response_limit = codefire_core::DecisionWorkProfile::http_v1()
            .limit(codefire_core::DecisionWorkDimension::ResponseBytes)
            .unwrap()
            .limit;
        for (label, body_bytes, succeeds) in [
            ("limit-minus-one", response_limit - 1, true),
            ("limit-exact", response_limit, true),
            ("limit-plus-one", response_limit + 1, false),
        ] {
            let (mut ledger, deadline) = mutation_response_ledger(label);
            let payload = Value::String("x".repeat(
                usize::try_from(body_bytes - 2).expect("test response size must fit usize"),
            ));
            let mut mutation_calls = 0_u64;
            let result =
                prepare_mutation_responses_with_deadline(200, &payload, &mut ledger, &deadline);
            match result {
                Ok(mut raw) => {
                    mutation_calls += 1;
                    let durable_bytes = raw.durable.as_ref().unwrap().body_bytes;
                    assert_eq!(
                        retained_http_bytes(&ledger),
                        body_bytes + durable_bytes,
                        "both real bodies must be retained without summing cumulative response work"
                    );
                    ledger.seal_prepublication().unwrap();
                    let mut prepared = None;
                    PreparedMutationResponsesOwner::new(&mut raw, &mut ledger, &mut prepared)
                        .unwrap()
                        .select_success()
                        .unwrap();
                    assert_eq!(retained_http_bytes(&ledger), body_bytes);
                    assert_eq!(
                        ledger.snapshot().cumulative
                            [&codefire_core::DecisionWorkDimension::ResponseBytes],
                        body_bytes
                    );
                    release_prepared_http_response(
                        &mut ledger,
                        prepared.take().unwrap(),
                        "release boundary test response",
                    )
                    .unwrap();
                }
                Err(error) => {
                    assert!(!succeeds, "unexpected {label} error: {error}");
                    assert!(matches!(error, CliError::ResourceBudget(_)));
                    assert_eq!(error.diagnostic()["dimension"], "response_bytes");
                    assert_eq!(retained_http_bytes(&ledger), 0);
                }
            }
            assert_eq!(mutation_calls, u64::from(succeeds));
            assert_eq!(retained_http_bytes(&ledger), 0);
        }
    }

    #[test]
    fn http_request_header_accepts_exact_limit_and_rejects_limit_plus_one() {
        fn request_with_header_size(size: usize) -> Vec<u8> {
            let prefix = b"GET /v1/projects/org/app/branches HTTP/1.1\r\nX-Fill: ";
            let suffix = b"\r\n\r\n";
            assert!(size >= prefix.len() + suffix.len());
            let mut request = Vec::with_capacity(size);
            request.extend_from_slice(prefix);
            request.resize(size - suffix.len(), b'x');
            request.extend_from_slice(suffix);
            request
        }

        let exact = read_http_request(&mut Cursor::new(request_with_header_size(
            MAX_HTTP_HEADER_BYTES,
        )))
        .unwrap();
        assert_eq!(exact.method(), "GET");

        let error = read_http_request(&mut Cursor::new(request_with_header_size(
            MAX_HTTP_HEADER_BYTES + 1,
        )))
        .unwrap_err();
        assert!(matches!(error, CliError::ResourceBudget(_)));
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["dimension"], "request_header_bytes");
        assert_eq!(diagnostic["observed"], MAX_HTTP_HEADER_BYTES as u64 + 1);
        assert_eq!(diagnostic["limit"], MAX_HTTP_HEADER_BYTES as u64);
    }

    #[test]
    fn request_parser_routing_owner_is_exact_and_parse_failures_release_routing() {
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let path = format!(
            "/v1/projects/org/app/branches?view={}",
            "q".repeat(8 * 1024)
        );
        let wire = format!("GET {path} HTTP/1.1\r\nContent-Length: 0\r\n\r\n");
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile.clone(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "exact parser routing owner",
        )
        .unwrap();
        let mut request = read_http_request_with_ledger(
            &mut Cursor::new(wire.into_bytes()),
            &mut ledger,
            &deadline,
        )
        .unwrap();
        let expected = logical_string_owner_bytes("GET").unwrap()
            + logical_string_owner_bytes(&path).unwrap()
            + compact_json_size(&json!({})).unwrap();
        assert_eq!(retained_http_bytes(&ledger), expected);
        request.release_body(&mut ledger).unwrap();
        request.release_routing(&mut ledger).unwrap();
        assert_eq!(retained_http_bytes(&ledger), 0);
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));

        for (label, body, declared) in [
            ("invalid-json", b"{".as_slice(), 1_usize),
            ("truncated", b"{}".as_slice(), 3_usize),
        ] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::HttpRequest,
                format!("{label} parser routing cleanup"),
            )
            .unwrap();
            let baseline_bytes = logical_string_owner_bytes("parser-baseline").unwrap();
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::RequestHeaders,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        baseline_bytes,
                    ),
                )
                .unwrap();
            let baseline = "parser-baseline".to_string();
            let mut wire = format!(
                "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {declared}\r\n\r\n"
            )
            .into_bytes();
            wire.extend_from_slice(body);
            let error =
                read_http_request_with_ledger(&mut Cursor::new(wire), &mut ledger, &deadline)
                    .unwrap_err();
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                    .copied()
                    .unwrap_or_default(),
                baseline_bytes,
                "{label}"
            );
            assert_eq!(
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
                declared as u64,
                "{label}"
            );
            assert!(!ledger.has_open_permits(), "{label}");
            drop(baseline);
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline_bytes,
                )
                .unwrap();
            terminate_http_request(&mut ledger, http_request_terminal_reason(&error)).unwrap();
            assert!(
                ledger
                    .snapshot()
                    .current
                    .values()
                    .all(|amount| *amount == 0),
                "{label}"
            );
        }
    }

    #[test]
    fn request_parser_releases_header_temporary_bytes_before_body_reservation() {
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline::from_profile(&profile);
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http request",
        )
        .unwrap();
        let request = format!(
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {MAX_HTTP_BODY_BYTES}\r\n\r\n"
        );

        let error = read_http_request_with_ledger(
            &mut Cursor::new(request.into_bytes()),
            &mut ledger,
            &deadline,
        )
        .unwrap_err();

        assert!(error.to_string().contains("incomplete HTTP request body"));
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            MAX_HTTP_BODY_BYTES as u64
        );
        assert_eq!(
            ledger.snapshot().peak[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            MAX_HTTP_BODY_BYTES as u64
        );
        terminate_http_request(
            &mut ledger,
            codefire_core::DecisionWorkTerminalReason::CorruptSubject,
        )
        .unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
    }

    #[test]
    fn business_conflict_uses_noncorruption_terminal_reason() {
        assert_eq!(
            http_request_terminal_reason(&CliError::IdempotencyConflict(
                "branch moved".to_string()
            )),
            codefire_core::DecisionWorkTerminalReason::Complete
        );
    }

    #[test]
    fn request_lock_wait_is_capped_by_the_absolute_http_deadline() {
        let deadline = HttpRequestDeadline {
            started_at: Instant::now(),
            limit: Duration::from_millis(30_000),
            limit_ms: 30_000,
        };
        let error = acquire_with_http_deadline(
            &json!({"lock": {"wait": true}}),
            &deadline,
            "test project gate",
            |options| {
                assert!(options.wait);
                assert!(options.timeout_ms.is_some_and(|timeout| timeout <= 30_000));
                Err::<(), _>(CliError::LockContention("held".to_string()))
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            CliError::RequestDeadlineExceeded {
                phase: "test project gate",
                limit_ms: 30_000
            }
        ));

        let immediate = acquire_with_http_deadline(
            &json!({"lock": {"wait": false}}),
            &deadline,
            "test project gate",
            |_| Err::<(), _>(CliError::LockContention("held".to_string())),
        )
        .unwrap_err();
        assert!(matches!(immediate, CliError::LockContention(_)));
    }

    #[test]
    fn http_request_rejects_oversized_body_before_reading_body() {
        let request = format!(
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_HTTP_BODY_BYTES + 1
        );
        let error = read_http_request(&mut Cursor::new(request.into_bytes())).unwrap_err();
        assert!(matches!(error, CliError::ResourceBudget(_)));
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["kind"], "resource_budget_exceeded");
        assert_eq!(diagnostic["dimension"], "request_body_bytes");
        assert_eq!(diagnostic["observed"], MAX_HTTP_BODY_BYTES as u64 + 1);
        assert_eq!(diagnostic["limit"], MAX_HTTP_BODY_BYTES as u64);
        assert_eq!(diagnostic["phase"], "request_body");
        assert_eq!(diagnostic["exit_code"], 60);
    }

    #[test]
    fn http_response_rejects_oversized_body_before_reading_body() {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            MAX_HTTP_BODY_BYTES + 1
        );
        let error = read_http_response(&mut Cursor::new(response.into_bytes())).unwrap_err();
        assert!(matches!(error, CliError::ResourceBudget(_)));
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["dimension"], "response_bytes");
        assert_eq!(diagnostic["exit_code"], 60);
    }

    #[test]
    fn http_response_content_length_is_single_strict_unsigned_decimal() {
        for response in [
            "HTTP/1.1 200 OK\r\nContent-Length: -1\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Length: nope\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Length:\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Length: 0\r\ncontent-length: 0\r\n\r\n",
        ] {
            let error =
                read_http_response(&mut Cursor::new(response.as_bytes().to_vec())).unwrap_err();
            assert!(matches!(error, CliError::Usage(_)));
            assert!(error.to_string().contains("Content-Length"));
        }

        for length in [
            (MAX_HTTP_BODY_BYTES as u64 + 1).to_string(),
            "999999999999999999999999999999999".to_string(),
        ] {
            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n");
            let error = read_http_response(&mut Cursor::new(response.into_bytes())).unwrap_err();
            assert!(matches!(error, CliError::ResourceBudget(_)));
            let diagnostic = error.diagnostic();
            assert_eq!(diagnostic["dimension"], "response_bytes");
            assert_eq!(diagnostic["phase"], "response_body");
            assert_eq!(diagnostic["command"], "http response");
            assert_eq!(diagnostic["continuation_supported"], false);
        }
    }

    #[test]
    fn http_response_content_length_accepts_boundary_before_body_read() {
        for length in [MAX_HTTP_BODY_BYTES - 1, MAX_HTTP_BODY_BYTES] {
            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n");

            let error = read_http_response(&mut Cursor::new(response.into_bytes())).unwrap_err();

            assert!(!matches!(error, CliError::ResourceBudget(_)));
            assert!(error.to_string().contains("incomplete HTTP response body"));
        }
    }

    #[test]
    fn http_response_wire_accepts_limit_minus_one_and_exact_limit() {
        for (serialized_size, text_size) in [
            (MAX_HTTP_BODY_BYTES - 1, MAX_HTTP_BODY_BYTES - 3),
            (MAX_HTTP_BODY_BYTES, MAX_HTTP_BODY_BYTES - 2),
        ] {
            let mut stream = Cursor::new(Vec::new());
            write_http_json(&mut stream, 200, &Value::String("x".repeat(text_size))).unwrap();

            let wire = stream.into_inner();
            let header_end = find_header_end(&wire).unwrap();
            let header = std::str::from_utf8(&wire[..header_end]).unwrap();
            assert!(header.starts_with("HTTP/1.1 200 OK"));
            assert!(header.contains(&format!("Content-Length: {serialized_size}")));
            assert_eq!(wire.len() - header_end - 4, serialized_size);
        }
    }

    #[test]
    fn http_response_wire_rejects_limit_plus_one_without_partial_200() {
        let mut stream = Cursor::new(Vec::new());

        write_http_json(
            &mut stream,
            200,
            &Value::String("x".repeat(MAX_HTTP_BODY_BYTES - 1)),
        )
        .unwrap();

        let wire = String::from_utf8(stream.into_inner()).unwrap();
        assert!(wire.starts_with("HTTP/1.1 413 Payload Too Large"));
        assert!(!wire.contains("HTTP/1.1 200"));
        assert!(wire.contains("resource_budget_exceeded"));
        assert!(wire.contains("\"dimension\":\"response_bytes\""));
        assert!(wire.contains("\"observed\":16777217"));
        assert!(wire.contains("\"limit\":16777216"));
        assert!(wire.contains("\"continuation_supported\":false"));
    }

    #[test]
    fn http_bundle_graph_rejects_oversized_frontier_before_push() {
        let storage_root = test_storage_root("oversized-object-graph");
        let objects = storage_root.join("objects");
        let references = vec![Value::String("CF-X".to_string()); 100_001];
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let mut policy_value = serde_json::to_value(&policy).unwrap();
        policy_value["extensions"] = json!([{
            "namespace": "example.test",
            "name": "oversized-graph",
            "version": 1,
            "critical": false,
            "references": references
        }]);
        write_object_records(&objects, [policy_value]).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle",
        )
        .unwrap();

        let error =
            collect_object_records_with_work(&objects, &policy.object_id, &mut ledger).unwrap_err();

        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["kind"], "resource_budget_exceeded");
        assert_eq!(diagnostic["dimension"], "maximum_frontier");
        assert_eq!(diagnostic["observed"], 100_001);
        assert_eq!(diagnostic["limit"], 100_000);
        assert_eq!(diagnostic["command"], "http bundle");
        assert_eq!(diagnostic["continuation_supported"], false);
        assert!(ledger.snapshot().terminal_reason.is_some());
        assert_object_collection_current_zero(&ledger);
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn collected_object_records_release_exact_document_retained_ownership() {
        let storage_root = test_storage_root("accounted-object-graph-release");
        let objects = storage_root.join("objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let path = codefire_store::object_record_path(&objects, &policy.object_id).unwrap();
        let document = codefire_store::read_object_record_document_path(&path).unwrap();
        let logical_bytes = codefire_store::compact_object_record_document_size(&document).unwrap()
            + codefire_core::LOGICAL_FRAME_BASE_BYTES;
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle",
        )
        .unwrap();

        let mut records = collect_object_records_with_owned_work_and_poll(
            &objects,
            &policy.object_id,
            &mut ledger,
            || Ok(()),
        )
        .unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            logical_bytes
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        records.release(&mut ledger).unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            0
        );
        let _ = fs::remove_dir_all(storage_root);
    }

    fn assert_object_collection_current_zero(ledger: &codefire_core::DecisionWorkLedger) {
        for dimension in [
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            codefire_core::DecisionWorkDimension::RetainedBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0,
                "{dimension:?} must be released on the error path"
            );
        }
    }

    #[test]
    fn object_collection_poll_error_releases_frontier_before_return() {
        let storage_root = test_storage_root("accounted-object-graph-poll-cleanup");
        let objects = storage_root.join("objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle poll cleanup",
        )
        .unwrap();

        let error = collect_object_records_with_owned_work_and_poll(
            &objects,
            &policy.object_id,
            &mut ledger,
            || Err(CliError::InvalidRepository("cancel collection".to_string())),
        )
        .unwrap_err();

        assert!(error.to_string().contains("cancel collection"));
        assert_object_collection_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn object_collection_unknown_prefix_scan_poll_error_releases_all_owners() {
        let storage_root = test_storage_root("accounted-object-extension-scan-cleanup");
        let objects = storage_root.join("objects");
        for index in 0..128 {
            fs::create_dir_all(objects.join(format!("extension-{index:03}"))).unwrap();
        }
        let object_id = "CF-OBJECT-000000000000000000000000";
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle extension scan cleanup",
        )
        .unwrap();
        let mut polls = 0_u64;

        let error = collect_object_records_with_owned_work_and_poll(
            &objects,
            object_id,
            &mut ledger,
            || {
                polls += 1;
                if polls == 40 {
                    Err(CliError::InvalidRepository(
                        "cancel extension scan".to_string(),
                    ))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("cancel extension scan"));
        assert_eq!(polls, 40);
        assert_object_collection_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn object_collection_parse_error_releases_scratch_and_frontier() {
        let storage_root = test_storage_root("accounted-object-graph-parse-cleanup");
        let objects = storage_root.join("objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let object_id = policy.object_id;
        let path = objects
            .join(codefire_store::object_subdir("policy").unwrap())
            .join(format!("{object_id}.json"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{not-json").unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle parse cleanup",
        )
        .unwrap();

        collect_object_records_with_owned_work_and_poll(&objects, &object_id, &mut ledger, || {
            Ok(())
        })
        .unwrap_err();

        assert_object_collection_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn accounted_object_records_reject_foreign_ledger_without_consuming_owner() {
        let storage_root = test_storage_root("accounted-object-graph-ledger-identity");
        let objects = storage_root.join("objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let mut owner = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle owner ledger",
        )
        .unwrap();
        let mut foreign = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "http bundle foreign ledger",
        )
        .unwrap();
        let mut records = collect_object_records_with_owned_work_and_poll(
            &objects,
            &policy.object_id,
            &mut owner,
            || Ok(()),
        )
        .unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        let error = records.release(&mut foreign).unwrap_err();
        assert!(error.to_string().contains("different decision-work ledger"));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(records.len(), 1);

        records.release(&mut owner).unwrap();
        assert_object_collection_current_zero(&owner);
        owner
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        foreign
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn object_publication_poll_error_drops_values_before_releasing_owner() {
        let storage_root = test_storage_root("accounted-object-publication-poll-cleanup");
        let source_objects = storage_root.join("source-objects");
        let destination_objects = storage_root.join("destination-objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&source_objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "object publication poll cleanup",
        )
        .unwrap();
        let mut records = collect_object_records_with_owned_work_and_poll(
            &source_objects,
            &policy.object_id,
            &mut ledger,
            || Ok(()),
        )
        .unwrap();

        let error = records
            .publish_with_work_and_poll(&destination_objects, &mut ledger, || {
                Err(CliError::InvalidRepository(
                    "cancel publication".to_string(),
                ))
            })
            .unwrap_err();

        assert!(error.to_string().contains("cancel publication"));
        assert_object_collection_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn object_publication_budget_error_releases_collected_owner() {
        let storage_root = test_storage_root("accounted-object-publication-budget-cleanup");
        let source_objects = storage_root.join("source-objects");
        let destination_objects = storage_root.join("destination-objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&source_objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "object publication budget cleanup",
        )
        .unwrap();
        let mut records = collect_object_records_with_owned_work_and_poll(
            &source_objects,
            &policy.object_id,
            &mut ledger,
            || Ok(()),
        )
        .unwrap();
        ledger.seal_prepublication().unwrap();

        records
            .publish_with_work_and_poll(&destination_objects, &mut ledger, || Ok(()))
            .unwrap_err();

        assert_object_collection_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn object_publication_success_releases_collected_owner_exactly_once() {
        let storage_root = test_storage_root("accounted-object-publication-success-cleanup");
        let source_objects = storage_root.join("source-objects");
        let destination_objects = storage_root.join("destination-objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&source_objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "object publication success cleanup",
        )
        .unwrap();
        let mut records = collect_object_records_with_owned_work_and_poll(
            &source_objects,
            &policy.object_id,
            &mut ledger,
            || Ok(()),
        )
        .unwrap();

        records
            .publish_with_work_and_poll(&destination_objects, &mut ledger, || Ok(()))
            .unwrap();

        assert_object_collection_current_zero(&ledger);
        assert!(
            codefire_store::object_record_path(&destination_objects, &policy.object_id).is_some()
        );
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn object_record_http_payload_keeps_typed_owner_until_payload_drop() {
        let storage_root = test_storage_root("accounted-object-http-payload-owner");
        let objects = storage_root.join("objects");
        let policy = codefire_store::object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        write_object_records(&objects, [serde_json::to_value(&policy).unwrap()]).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "object payload owner",
        )
        .unwrap();
        let mut records = collect_object_records_with_owned_work_and_poll(
            &objects,
            &policy.object_id,
            &mut ledger,
            || Ok(()),
        )
        .unwrap();
        let retained_before = retained_http_bytes(&ledger);

        let (payload, mut owner) = records.transfer_to_http_payload(&ledger).unwrap();
        assert_eq!(retained_http_bytes(&ledger), retained_before);
        drop(payload);
        owner.release_after_payload_drop(&mut ledger).unwrap();

        assert_object_collection_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn oversized_request_receives_typed_413_before_dispatch() {
        let storage_root = test_storage_root("oversized-request-wire");
        let request = format!(
            "POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_HTTP_BODY_BYTES + 1
        );
        let mut stream = Cursor::new(request.into_bytes());

        handle_http_stream(&storage_root, &mut stream).unwrap();

        let wire = String::from_utf8(stream.into_inner()).unwrap();
        assert!(wire.contains("HTTP/1.1 413 Payload Too Large"));
        assert!(wire.contains("resource_budget_exceeded"));
        assert!(wire.contains("request_body_bytes"));
        assert!(wire.contains("\"exit_code\":60"));
        assert!(!storage_root.join("org").exists());
        let _ = fs::remove_dir_all(storage_root);
    }

    #[test]
    fn http_remote_error_preserves_structured_diagnostic() {
        let error = http_remote_error(
            403,
            &json!({
                "error": "signature rejected",
                "exit_code": ExitCode::AuthenticationOrSignatureFailure.code(),
                "diagnostics": [
                    {"kind": "remote_signature_rejected", "message": "bad signature"}
                ]
            }),
        );
        assert_eq!(
            error.exit_code(),
            ExitCode::AuthenticationOrSignatureFailure.code()
        );
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["kind"], "remote_signature_rejected");
        assert_eq!(diagnostic["http_status"], 403);
        assert_eq!(
            diagnostic["exit_code"],
            ExitCode::AuthenticationOrSignatureFailure.code()
        );
    }

    #[test]
    fn http_remote_error_preserves_every_resource_budget_field() {
        let error = http_remote_error(
            413,
            &json!({
                "error": "bounded response rejected",
                "exit_code": 60,
                "diagnostics": [{
                    "kind": "resource_budget_exceeded",
                    "exit_code": 60,
                    "profile": "codefire.decision-work.v1",
                    "dimension": "response_bytes",
                    "observed": 16777217,
                    "limit": 16777216,
                    "phase": "response_body",
                    "command": "http bundle",
                    "continuation_supported": false
                }]
            }),
        );

        assert_eq!(error.exit_code(), 60);
        let diagnostic = error.diagnostic();
        assert_eq!(diagnostic["kind"], "resource_budget_exceeded");
        assert_eq!(diagnostic["http_status"], 413);
        assert_eq!(diagnostic["profile"], "codefire.decision-work.v1");
        assert_eq!(diagnostic["dimension"], "response_bytes");
        assert_eq!(diagnostic["observed"], 16_777_217);
        assert_eq!(diagnostic["limit"], 16_777_216);
        assert_eq!(diagnostic["phase"], "response_body");
        assert_eq!(diagnostic["command"], "http bundle");
        assert_eq!(diagnostic["continuation_supported"], false);
    }

    #[test]
    fn parse_http_url_accepts_bracketed_ipv6_authority() {
        let endpoint = parse_http_url("https://[::1]:9443/v1/projects/org/app").unwrap();

        assert_eq!(endpoint.scheme, "https");
        assert_eq!(endpoint.host, "::1");
        assert_eq!(endpoint.port, 9443);
        assert_eq!(endpoint.host_header(), "[::1]:9443");
        assert!(parse_http_url("https://::1:9443/v1").is_err());
        assert!(parse_http_url("https://[::1:9443/v1").is_err());
    }

    #[test]
    fn http_timeout_uses_positive_env_override() {
        let _guard = HTTP_TIMEOUT_ENV_LOCK.lock().unwrap();
        std::env::set_var("CODEFIRE_HTTP_TIMEOUT_MS", "1234");
        assert_eq!(http_timeout().unwrap(), Duration::from_millis(1234));
        std::env::remove_var("CODEFIRE_HTTP_TIMEOUT_MS");
        assert_eq!(
            http_timeout().unwrap(),
            Duration::from_millis(DEFAULT_HTTP_TIMEOUT_MS)
        );
    }

    #[test]
    fn http_timeout_rejects_invalid_env_override() {
        let _guard = HTTP_TIMEOUT_ENV_LOCK.lock().unwrap();
        std::env::set_var("CODEFIRE_HTTP_TIMEOUT_MS", "abc");
        let invalid = http_timeout().unwrap_err();
        assert!(invalid.to_string().contains("CODEFIRE_HTTP_TIMEOUT_MS"));
        assert!(invalid
            .to_string()
            .contains("positive integer milliseconds"));

        std::env::set_var("CODEFIRE_HTTP_TIMEOUT_MS", "0");
        let nonpositive = http_timeout().unwrap_err();
        assert!(nonpositive.to_string().contains("CODEFIRE_HTTP_TIMEOUT_MS"));
        assert!(nonpositive
            .to_string()
            .contains("positive integer milliseconds"));
        std::env::remove_var("CODEFIRE_HTTP_TIMEOUT_MS");
    }

    #[test]
    fn expired_request_deadline_rejects_before_parse_or_mutation_with_typed_503() {
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let deadline = HttpRequestDeadline {
            started_at: Instant::now() - Duration::from_millis(30_001),
            limit: Duration::from_millis(30_000),
            limit_ms: 30_000,
        };
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "expired request",
        )
        .unwrap();
        let request = b"GET /v1/projects/org/app/branches HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        let error = read_http_request_with_ledger(
            &mut Cursor::new(request.to_vec()),
            &mut ledger,
            &deadline,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            CliError::RequestDeadlineExceeded {
                phase: "request header parsing",
                limit_ms: 30_000
            }
        ));
        assert!(ledger.snapshot().cumulative.is_empty());
        let (status, payload) = http_error_response(&error);
        assert_eq!(status, 503);
        assert_eq!(payload["kind"], "request_deadline_exceeded");
        assert_eq!(payload["exit_code"], 60);

        let storage_root = test_storage_root("expired-deadline-no-mutation");
        let mut owned = Some(ledger);
        let mut prepared = None;
        let upload_error = handle_http_upload_with_ledger(
            &storage_root,
            "org",
            "app",
            "main",
            &json!({}),
            &mut HttpRouteContext {
                ledger: &mut owned,
                prepared: &mut prepared,
                deadline: &deadline,
                payload_owner: None,
            },
        )
        .unwrap_err();
        assert!(matches!(
            upload_error,
            CliError::RequestDeadlineExceeded {
                phase: "upload planning",
                ..
            }
        ));
        assert!(!storage_root.exists());
    }

    #[test]
    fn http_request_rejects_unsupported_method_and_invalid_path() {
        let lowercase = b"get /v1/projects/org/app/branches HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        let error = read_http_request(&mut Cursor::new(lowercase.to_vec())).unwrap_err();
        assert!(error.to_string().contains("unsupported HTTP method"));

        let bad_percent =
            b"GET /v1/projects/org/%2Fapp/branches HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        let error = read_http_request(&mut Cursor::new(bad_percent.to_vec())).unwrap_err();
        assert!(error.to_string().contains("invalid HTTP path component"));
    }

    #[test]
    fn http_upload_tmp_dirs_are_unique_for_same_head() {
        let project_root = Path::new("/tmp/codefire-http-test");
        let first = unique_http_upload_tmp_dir(project_root, "CF-COMMIT-same");
        let second = unique_http_upload_tmp_dir(project_root, "CF-COMMIT-same");

        assert_ne!(first, second);
        assert_eq!(first.parent(), second.parent());
    }

    #[test]
    fn server_connection_limiter_caps_and_releases_active_connections() {
        let limiter = Arc::new(ServerConnectionLimiter::new(2));
        let first = limiter.try_acquire().unwrap();
        let second = limiter.try_acquire().unwrap();

        assert_eq!(limiter.active(), 2);
        assert!(limiter.try_acquire().is_none());

        drop(first);
        assert_eq!(limiter.active(), 1);
        let third = limiter.try_acquire().unwrap();
        assert_eq!(limiter.active(), 2);

        drop(second);
        drop(third);
        assert_eq!(limiter.active(), 0);
    }

    #[test]
    fn spawned_http_handler_keeps_slow_connection_from_blocking_next_request() {
        let storage_root = Arc::new(test_storage_root("concurrent-server"));
        fs::create_dir_all(storage_root.as_path()).unwrap();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let limiter = Arc::new(ServerConnectionLimiter::new(2));
        let server_storage_root = Arc::clone(&storage_root);
        let server_limiter = Arc::clone(&limiter);

        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let permit = server_limiter.try_acquire().unwrap();
                spawn_http_connection_handler(
                    Arc::clone(&server_storage_root),
                    None,
                    stream,
                    permit,
                )
                .unwrap();
            }
        });

        let mut slow_client = TcpStream::connect(address).unwrap();
        slow_client
            .write_all(b"POST /v1/projects/org/app/branches/main/upload HTTP/1.1\r\n")
            .unwrap();

        let started_at = Instant::now();
        let mut fast_client = TcpStream::connect(address).unwrap();
        fast_client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        fast_client
            .write_all(b"GET /invalid HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
            .unwrap();

        let mut response = [0u8; 128];
        let read = loop {
            match fast_client.read(&mut response) {
                Ok(read) => break read,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => panic!("fast HTTP client read failed: {error}"),
            }
        };

        assert!(started_at.elapsed() < Duration::from_secs(2));
        assert!(std::str::from_utf8(&response[..read])
            .unwrap()
            .starts_with("HTTP/1.1 400 Bad Request"));

        drop(slow_client);
        server.join().unwrap();
        let _ = fs::remove_dir_all(storage_root.as_path());
    }

    fn test_storage_root(label: &str) -> PathBuf {
        let suffix = HTTP_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("codefire-{label}-{}-{suffix}", std::process::id()))
    }

    fn count_regular_files(root: &Path) -> usize {
        if !root.exists() {
            return 0;
        }
        fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .map(|path| {
                if path.is_dir() {
                    count_regular_files(&path)
                } else {
                    usize::from(path.is_file())
                }
            })
            .sum()
    }
}
