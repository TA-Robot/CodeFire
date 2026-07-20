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
    let request = match read_http_request_with_ledger(
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
        &request,
        &mut ledger,
        &mut prepared,
        &deadline,
    );
    ledger
        .as_mut()
        .expect("request ledger must return from dispatch")
        .release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            request.retained_body_bytes,
        )
        .map_err(|error| upload_work_error(error, "release HTTP request body"))?;

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
    method: String,
    path: String,
    body: Value,
    retained_body_bytes: u64,
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
        if self.validation_proof.is_some()
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
        let owns_child = self.validation_proof.is_some()
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
        let mut first_error = self.cancel_effect_owners().err();
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
        if self.responses.is_some() {
            return Err(CliError::InvalidRepository(
                "HTTP mutation visibility terminal still owns unselected responses".to_string(),
            ));
        }
        let response = self.prepared_response.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "HTTP mutation visibility terminal has no selected response".to_string(),
            )
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
                "HTTP mutation visibility terminal ownership does not exactly match its selected response"
                    .to_string(),
            ));
        }
        Ok(())
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
                            let frame_bytes = retain_merge_request_subject_digest_cache_entry(
                                ledger,
                                &source_head,
                                &digest,
                            )?;
                            subject_digests.insert(source_head.clone(), (digest, frame_bytes));
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
                                subject_digests.remove(&source_head);
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
    let header = String::from_utf8_lossy(&buffer[..header_end]);
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
    drop(header);
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            u64::try_from(header_bytes).map_err(|_| {
                CliError::InvalidRepository("HTTP request header size overflow".to_string())
            })?,
        )
        .map_err(|error| upload_work_error(error, "release request header input"))?;
    buffer.drain(..header_bytes);
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
    let body_bytes = &buffer[..content_length];
    let body = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(body_bytes)?
    };
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
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            content_length_u64,
        )
        .map_err(|error| upload_work_error(error, "release request body scratch"))?;
    deadline.check("request dispatch")?;
    Ok(HttpRequest {
        method,
        path,
        body,
        retained_body_bytes,
    })
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

fn parse_http_request_line(request_line: &str) -> Result<(String, String), CliError> {
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(CliError::InvalidRepository(
            "invalid HTTP request line".to_string(),
        ));
    }
    let method = parts[0];
    if !matches!(method, "GET" | "POST") {
        return Err(CliError::Usage(format!(
            "unsupported HTTP method: {method}"
        )));
    }
    if parts[2] != "HTTP/1.1" {
        return Err(CliError::Usage(format!(
            "unsupported HTTP version: {}",
            parts[2]
        )));
    }
    let path = normalize_http_request_path(parts[1])?;
    Ok((method.to_string(), path))
}

fn normalize_http_request_path(raw_path: &str) -> Result<String, CliError> {
    if !raw_path.starts_with('/') {
        return Err(CliError::Usage("HTTP path must be absolute".to_string()));
    }
    if raw_path.contains('\\')
        || raw_path.contains("://")
        || raw_path.as_bytes().iter().any(u8::is_ascii_control)
    {
        return Err(CliError::Usage("invalid HTTP path".to_string()));
    }
    let (path, query) = raw_path
        .split_once('?')
        .map_or((raw_path, None), |(path, query)| (path, Some(query)));
    for part in path.split('/').filter(|part| !part.is_empty()) {
        let _ = decode_http_path_component(part)?;
    }
    Ok(match query {
        Some(query) => format!("{path}?{query}"),
        None => path.to_string(),
    })
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
fn dispatch_http(storage_root: &Path, request: &HttpRequest) -> Result<(u16, Value), CliError> {
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
    request: &HttpRequest,
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
    let (org, app, rest) = http_project_parts(&request.path)?;
    let rest = rest.iter().map(String::as_str).collect::<Vec<_>>();
    let response = match (request.method.as_str(), rest.as_slice()) {
        ("GET", ["doctor"]) => handle_http_doctor(
            storage_root,
            &org,
            &app,
            doctor_quick_requested(&request.path)?,
            &mut context,
        ),
        ("GET", ["branches"]) => handle_http_list(storage_root, &org, &app, &mut context),
        ("GET", ["branches", branch, "bundle"]) => {
            handle_http_bundle(storage_root, &org, &app, branch, &mut context)
        }
        ("GET", ["merge-requests"]) => {
            handle_http_request_list(storage_root, &org, &app, &mut context)
        }
        ("POST", ["branches", branch, "upload"]) => handle_http_upload_with_ledger(
            storage_root,
            &org,
            &app,
            branch,
            &request.body,
            &mut context,
        ),
        ("POST", ["merge-requests"]) => {
            handle_http_request_merge(storage_root, &org, &app, &request.body, &mut context)
        }
        ("POST", ["merge-requests", mr_id, "review"]) => {
            handle_http_request_review(storage_root, &org, &app, mr_id, &request.body, &mut context)
        }
        ("POST", ["merge-requests", mr_id, "apply"]) => {
            handle_http_request_apply(storage_root, &org, &app, mr_id, &request.body, &mut context)
        }
        _ => Ok((404, json!({"error": "unknown endpoint"}))),
    };
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

fn handle_http_upload_with_ledger(
    storage_root: &Path,
    org: &str,
    app: &str,
    branch_name: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("upload planning")?;
    let project_root = http_project_root(storage_root, org, app);
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
    let request_target =
        super::signatures::remote_request_target(org, app, "upload", Some(branch_name), None);
    super::signatures::preflight_remote_request_signature_with_work_and_poll(
        &project_root,
        "upload",
        actor,
        &request_target,
        request,
        request.get("request_signature"),
        context
            .ledger
            .as_mut()
            .expect("upload signature preflight ledger must be present"),
        || context.deadline.check("upload request signature preflight"),
    )?;
    let project_gate = acquire_with_http_deadline(
        request,
        context.deadline,
        "upload project gate",
        |options| RemoteProjectGate::acquire_exclusive(&project_root, options),
    )?;
    let dirs = remote_dirs(&project_root);
    let preflight = preflight_http_upload(
        &dirs.objects,
        &head,
        records,
        context
            .ledger
            .as_mut()
            .expect("upload ledger must be present"),
        context.deadline,
    )?;
    context.deadline.check("upload object preflight")?;
    let stable_project_id = remote_project_stable_id(&project_root)?;
    let publication_request = upload_publication(
        actor,
        remote_idempotency_key(request),
        &UploadPublicationIdentity {
            stable_project_id: &stable_project_id,
            remote_branch: branch_name,
            local_branch: &local_branch,
            head: &head,
            object_set_digest: preflight.object_set_digest.as_str(),
        },
    );
    let response = upload_result(branch_name, &head);
    preflight_publication_signature_with_recovery_and_poll(
        &project_gate,
        &project_root,
        &publication_request,
        "upload",
        actor,
        &request_target,
        request,
        request.get("request_signature"),
        context
            .ledger
            .as_mut()
            .expect("upload nonce preflight ledger must be present"),
        || context.deadline.check("upload request nonce preflight"),
    )?;

    if !context
        .ledger
        .as_ref()
        .expect("upload validation ledger must be present")
        .snapshot()
        .current
        .values()
        .all(|amount| *amount == 0)
    {
        return Err(CliError::InvalidRepository(
            "upload Store binding requires all route-owned current memory to be released"
                .to_string(),
        ));
    }

    let overlay_clone_bytes = preflight.overlay_clone_bytes;
    context
        .ledger
        .as_mut()
        .expect("upload ledger must be present for overlay clone reservation")
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                overlay_clone_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "reserve upload overlay clones"))?;
    let mut overlay_documents = Vec::with_capacity(records.len());
    for record in records {
        if let Err(error) = context.deadline.check("upload overlay parsing") {
            drop(overlay_documents);
            context
                .ledger
                .as_mut()
                .expect("upload ledger must remain present on overlay cancellation")
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    overlay_clone_bytes,
                )
                .map_err(|accounting| {
                    upload_work_error(accounting, "release cancelled upload overlay clones")
                })?;
            return Err(error);
        }
        match codefire_store::object_record_document_from_value(record.clone()) {
            Ok(document) => overlay_documents.push(document),
            Err(error) => {
                drop(overlay_documents);
                context
                    .ledger
                    .as_mut()
                    .expect("upload ledger must remain present on overlay parse failure")
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        overlay_clone_bytes,
                    )
                    .map_err(|accounting| {
                        upload_work_error(accounting, "release failed upload overlay clones")
                    })?;
                return Err(CliError::from(error));
            }
        }
    }
    let owned_ledger = context
        .ledger
        .take()
        .expect("upload ledger must be present before sealed validation");
    let overlay_input =
        match codefire_store::prepare_sealed_commit_overlay_validation_input_with_cancellation_and_temporary_transfer(
            overlay_documents,
            owned_ledger,
            overlay_clone_bytes,
            || context.deadline.is_expired(),
        ) {
            Ok(input) => input,
            Err(error) => {
                let (error, recovered) = error.discard_documents_and_recover_ledger();
                *context.ledger = Some(recovered);
                let temporary = context
                    .ledger
                    .as_ref()
                    .and_then(|ledger| {
                        ledger
                            .snapshot()
                            .current
                            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    })
                    .copied()
                    .unwrap_or_default();
                if temporary >= overlay_clone_bytes && overlay_clone_bytes > 0 {
                    context
                        .ledger
                        .as_mut()
                        .expect("upload ledger must be recovered after overlay prepare failure")
                        .release(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            overlay_clone_bytes,
                        )
                        .map_err(|accounting| {
                            upload_work_error(
                                accounting,
                                "release rejected upload overlay clones",
                            )
                        })?;
                }
                if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                    return Err(context.deadline.error("upload overlay preparation"));
                }
                return Err(CliError::from(error));
            }
        };
    let mut session = match codefire_store::begin_sealed_commit_overlay_validation_session_with_recoverable_ledger_and_cancellation(
        &dirs.objects,
        overlay_input,
        || context.deadline.is_expired(),
    ) {
            Ok(session) => session,
            Err(error) => {
                let (error, recovered) = error.discard_documents_and_recover_ledger();
                *context.ledger = Some(recovered);
                if matches!(error, codefire_store::StoreError::ValidationCancelled) {
                    return Err(context.deadline.error("upload overlay binding"));
                }
                return Err(CliError::from(error));
            }
        };
    if let Err(error) =
        session.validate_commit_with_cancellation(&head, || context.deadline.is_expired())
    {
        let reason = http_store_terminal_reason(&error);
        *context.ledger = Some(abort_http_validation_session(&mut session, reason));
        if matches!(error, codefire_store::StoreError::ValidationCancelled) {
            return Err(context.deadline.error("upload sealed validation"));
        }
        return Err(CliError::from(error));
    }
    let roots = vec![head.clone()];
    let bound = match session.bind_prepublication_validation(&roots) {
        Ok(bound) => bound,
        Err(error) => {
            let (error, mut session) = error.into_parts();
            *context.ledger = Some(discard_http_validation_session(&mut session));
            return Err(http_store_error(error));
        }
    };
    let mut owner = match PreparedHttpMutationTransaction::from_bound_validation(bound) {
        Ok(owner) => owner,
        Err(failure) => {
            let (error, ledger) = recover_http_validation_build_failure(failure);
            *context.ledger = Some(ledger);
            return Err(error);
        }
    };

    let pending_nonce = validate_publication_signature_with_recovery_and_poll(
        &project_gate,
        &project_root,
        &publication_request,
        "upload",
        actor,
        &request_target,
        request,
        request.get("request_signature"),
        owner
            .ledger
            .as_mut()
            .expect("upload mutation owner retains its ledger"),
        || {
            context
                .deadline
                .check("upload request signature validation")
        },
    );
    let mut pending_nonce = match pending_nonce {
        Ok(nonce) => Some(nonce),
        Err(error) => {
            *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
            return Err(error);
        }
    };
    let mut authority_accounting = None;
    let mut fresh = false;
    let publication_plan = owner.plan_publication(
        &project_gate,
        &project_root,
        &publication_request,
        &mut pending_nonce,
        &response,
        |subject, ledger| {
            let (values, accounting) = prepare_upload_authority_values(
                UploadAuthorityInput {
                    project_root: &project_root,
                    remote_branch: branch_name,
                    local_branch: &local_branch,
                    actor,
                    head: &head,
                    subject,
                },
                ledger,
                context.deadline,
            )?;
            authority_accounting = Some(accounting);
            fresh = true;
            Ok(values.into_writes(branch_name))
        },
    );
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
        *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
        return Err(error);
    }
    if let Err(error) = publication_plan {
        *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
        return Err(error);
    }

    if fresh {
        if let Err(error) = prepare_and_attach_upload_object_batch(
            &mut owner,
            &dirs.objects,
            records,
            preflight.overlay_clone_bytes,
            context.deadline,
        ) {
            *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
            return Err(error);
        }
    }
    if let Err(error) = owner.prepare_and_attach_responses(200, &response, context.deadline) {
        *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
        return Err(error);
    }
    drop(response);

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
        *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
        return Err(error);
    }
    if owner.commit_effects_after_seal().is_ok() {
        if let Err(error) = owner.validate_visibility_terminal() {
            *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
            return Err(error);
        }
        let mut terminal = owner.into_visibility_terminal_after_validation();
        if let Err(error) = terminal.select_success() {
            *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
            return Err(error);
        }
        if let Err(error) = terminal.validate_http_handoff() {
            *context.ledger = Some(cancel_http_mutation_terminal(&mut terminal));
            return Err(error);
        }
        let (ledger, prepared) = terminal.into_http_parts_after_validation();
        debug_assert_eq!(prepared.status, 200);
        *context.ledger = Some(ledger);
        *context.prepared = Some(prepared);
        return Ok((200, Value::Null));
    }

    if let Err(error) = owner.select_durable_after_commit_error() {
        *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
        return Err(error);
    }
    if let Err(error) = owner.validate_http_handoff() {
        *context.ledger = Some(cancel_http_mutation_owner(&mut owner));
        return Err(error);
    }
    let (ledger, prepared) = owner.into_http_parts_after_validation();
    debug_assert_eq!(prepared.status, 503);
    *context.ledger = Some(ledger);
    *context.prepared = Some(prepared);
    Ok((503, Value::Null))
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

fn prepare_and_attach_upload_object_batch(
    owner: &mut PreparedHttpMutationTransaction<'_>,
    objects: &Path,
    records: &[Value],
    clone_bytes: u64,
    deadline: &HttpRequestDeadline,
) -> Result<(), CliError> {
    let ledger = owner
        .ledger
        .as_mut()
        .expect("upload mutation owner retains its ledger");
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                clone_bytes,
            ),
        )
        .map_err(|error| upload_work_error(error, "reserve upload batch input clones"))?;

    let documents = (|| -> Result<Vec<_>, CliError> {
        let mut documents = Vec::with_capacity(records.len());
        for record in records {
            deadline.check("upload object batch parsing")?;
            documents.push(codefire_store::object_record_document_from_value(
                record.clone(),
            )?);
        }
        Ok(documents)
    })();
    let documents = match documents {
        Ok(documents) => documents,
        Err(error) => {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    clone_bytes,
                )
                .map_err(|release| {
                    upload_work_error(release, "release rejected upload batch input clones")
                })?;
            return Err(error);
        }
    };

    let prepared = codefire_store::PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
        objects,
        documents,
        ledger,
        || deadline.is_expired(),
    )
    .map_err(http_store_error);
    let mut candidate = match prepared {
        Ok(batch) => Some(batch),
        Err(error) => {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    clone_bytes,
                )
                .map_err(|release| {
                    upload_work_error(release, "release failed upload batch input clones")
                })?;
            return Err(error);
        }
    };
    if let Err(error) = ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        clone_bytes,
    ) {
        if let Err(cancel) = candidate
            .as_mut()
            .expect("prepared upload batch remains present")
            .cancel(ledger)
        {
            fail_closed_http_mutation_drop(format!(
                "upload batch input release failed ({error}) and batch cancellation failed: {cancel}"
            ));
        }
        drop(candidate.take());
        return Err(upload_work_error(
            error,
            "release upload batch input clones",
        ));
    }
    if let Err(error) = owner.attach_object_batch(&mut candidate) {
        if let Err(cancel) = candidate
            .as_mut()
            .expect("rejected upload batch remains with caller")
            .cancel(
                owner
                    .ledger
                    .as_mut()
                    .expect("upload mutation owner retains its ledger"),
            )
        {
            fail_closed_http_mutation_drop(format!(
                "upload batch attachment failed ({error}) and batch cancellation failed: {cancel}"
            ));
        }
        drop(candidate.take());
        return Err(error);
    }
    Ok(())
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
    let digest_retained_bytes =
        codefire_core::logical_retained_frame_bytes(HTTP_OBJECT_SET_DIGEST_TEMPLATE)
            .map_err(|error| upload_work_error(error, "reserve upload object-set digest"))?;
    work.acquire(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        digest_retained_bytes,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::Validation,
            codefire_core::DecisionWorkSurface::Atomic,
        ),
        "reserve upload object-set digest",
    )?;
    let object_set_digest = source_object_set_digest_ordered_with_poll(
        records,
        incoming.values().copied(),
        ledger,
        &mut polls.digest,
    )?;
    (polls.digest_result)()?;
    let inline_object_set_digest = InlineHttpObjectSetDigest::copy_from(&object_set_digest)?;
    drop(object_set_digest);
    work.release(
        ledger,
        codefire_core::DecisionWorkDimension::RetainedBytes,
        digest_retained_bytes,
        "release upload object-set digest",
    )?;

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
    )?;
    with_http_sealed_validation(
        context,
        &target_objects,
        &[target_head.as_str()],
        "merge-request target validation",
        |_| Ok(()),
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
    let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
        mr_id: &mr_id,
        source_project_root: &source_root,
        target_project_root: &target_root,
        source_branch: &source_info.branch,
        target_branch: &target_info.branch,
        source_head: &source_head,
        target_head: &target_head,
        source_object_set_digest: &object_set_digest,
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
            source_object_set_digest: &object_set_digest,
        },
    );
    let gate = project_gates.exclusive_for(&target_root)?;
    source_records.publish_with_work_and_poll(
        &target_objects,
        context
            .ledger
            .as_mut()
            .expect("merge-request ledger must be present for object escrow"),
        || context.deadline.check("merge-request object escrow"),
    )?;
    if let Some(published) = replay_json_publication(gate, &target_root, &publication_request)? {
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
    let published = execute_json_publication(
        gate,
        &target_root,
        &publication_request,
        &[AuthorityWrite {
            authority: RemoteAuthority::MergeRequest(mr_id),
            before: None,
            after: mr_payload,
        }],
        &response,
    )?;
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
    target_root: PathBuf,
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
) -> (CliError, codefire_core::DecisionWorkLedger) {
    match failure.cancel_and_take_ledger(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
    {
        Ok(recovered) => recovered,
        Err((cause, cancellation)) => {
            match cancellation.retry(codefire_core::DecisionWorkTerminalReason::CorruptSubject) {
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
    request: &Value,
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
        let applied_by = request
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("local");
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

fn handle_http_request_apply(
    storage_root: &Path,
    org: &str,
    app: &str,
    mr_id: &str,
    request: &Value,
    context: &mut HttpRouteContext<'_>,
) -> Result<(u16, Value), CliError> {
    context.deadline.check("apply planning")?;
    if context.prepared.is_some() {
        return Err(CliError::InvalidRepository(
            "apply route started with an existing prepared response".to_string(),
        ));
    }
    let project_root = http_project_root(storage_root, org, app);
    let mr_path = crate::remote::remote_merge_request_path(&project_root, mr_id);
    let actor = request
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let request_target =
        super::signatures::remote_request_target(org, app, "apply", None, Some(mr_id));
    // Authenticate before gate acquisition or merge-request-specific work can
    // become observable. Nonce-cache validation and ownership stay deferred
    // until the proof owner exists, so Store binding still receives a ledger
    // whose current ownership is zero.
    super::signatures::preflight_remote_request_signature_with_work_and_poll(
        &project_root,
        "apply",
        actor,
        &request_target,
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
            RemoteProjectGate::acquire_exclusive(&project_root, options)
        })?;

    let Some(accounted_mr) =
        read_optional_http_json_accounted(&mr_path, context, "apply merge-request read")?
    else {
        return Ok((404, json!({"error": "merge request not found"})));
    };
    let (mr, mr_retained) = accounted_mr.into_parts();
    let identity = (|| -> Result<Result<ApplyMutationIdentity, (u16, Value)>, CliError> {
        validate_http_merge_request_record_shape(storage_root, &mr)?;
        let source_info = parse_cf_http_url(&required_string(&mr, &["source_url"])?)?;
        let target_info = parse_cf_http_url(&required_string(&mr, &["target_url"])?)?;
        let target_root = http_project_root(storage_root, &target_info.org, &target_info.app);
        if target_root != project_root || target_info.org != org || target_info.app != app {
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
        let source_head = required_string(&mr, &["source_head"])?;
        let target_head_at_request = required_string(&mr, &["target_head_at_request"])?;
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
        let response = apply_result(mr_id, &target_info.branch, &source_head);
        Ok(Ok(ApplyMutationIdentity {
            target_root,
            target_branch: target_info.branch,
            source_head,
            target_head_at_request,
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
    preflight_publication_signature_with_recovery_and_poll(
        &project_gate,
        &identity.target_root,
        &identity.publication_request,
        "apply",
        actor,
        &request_target,
        request,
        request.get("request_signature"),
        context
            .ledger
            .as_mut()
            .expect("apply nonce preflight ledger must be present"),
        || context.deadline.check("apply request nonce preflight"),
    )?;
    let current_is_zero = context
        .ledger
        .as_ref()
        .expect("apply validation ledger must be present")
        .snapshot()
        .current
        .values()
        .all(|amount| *amount == 0);
    if !current_is_zero {
        return Err(CliError::InvalidRepository(
            "apply Store binding requires all route-owned current memory to be released"
                .to_string(),
        ));
    }

    debug_assert!(context
        .ledger
        .as_ref()
        .expect("apply signature preflight ledger must be present")
        .snapshot()
        .current
        .values()
        .all(|amount| *amount == 0));

    let owned_ledger = context
        .ledger
        .take()
        .expect("apply validation ledger must be present");
    let mut session = match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger_and_cancellation(
        &remote_dirs(&identity.target_root).objects,
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
            let reason = http_store_terminal_reason(&error);
            let ledger = abort_http_validation_session(&mut session, reason);
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
            let (error, ledger) = recover_http_validation_build_failure(failure);
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
        &request_target,
        request,
        request.get("request_signature"),
        owner
            .ledger
            .as_mut()
            .expect("apply mutation owner retains its ledger"),
        || context.deadline.check("apply request signature validation"),
    );
    let mut pending_nonce = match pending_nonce {
        Ok(nonce) => Some(nonce),
        Err(error) => {
            let ledger = cancel_http_mutation_owner(&mut owner);
            *context.ledger = Some(ledger);
            return Err(error);
        }
    };
    let target_branch = identity.target_branch.clone();
    let mut authority_accounting = None;
    let mut fresh_rejection = None;
    let publication_plan = owner.plan_publication(
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
                &project_root,
                &mr_path,
                mr_id,
                request,
                &identity,
                ledger,
                context.deadline,
            )? {
                ApplyAuthorityPreparation::Ready(values, accounting) => {
                    authority_accounting = Some(accounting);
                    Ok((*values).into_writes(&target_branch, mr_id))
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
    if let Err(error) =
        owner.prepare_and_attach_responses(200, &identity.response, context.deadline)
    {
        let ledger = cancel_http_mutation_owner(&mut owner);
        *context.ledger = Some(ledger);
        return Err(error);
    }
    drop(identity.response);

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
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/v1/projects/org/app/doctor?quick=1".to_string(),
            body: json!({}),
            retained_body_bytes: 0,
        };

        let (status, payload) = dispatch_http(&storage_root, &request).unwrap();

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
        assert!(matches!(
            non_ff.unwrap_err(),
            CliError::IdempotencyConflict(_)
        ));
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
    fn http_upload_missing_project_rejects_without_layout_lock_nonce_or_objects() {
        let storage_root = test_storage_root("upload-missing-project");
        let project_root = http_project_root(&storage_root, "org", "app");
        let request = json!({
            "branch": "local-main",
            "head": format!("CF-COMMIT-{}", "a".repeat(64)),
            "objects": [],
            "actor": "uploader",
            "idempotency_key": "missing-project",
        });

        let (result, ledger, prepared) = run_accounted_upload(&storage_root, &request);

        assert!(result.is_err());
        assert!(prepared.is_none());
        assert!(!project_root.exists());
        assert!(!storage_root.exists());
        assert!(ledger
            .as_ref()
            .unwrap()
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
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

        let digest = source_object_set_digest_ordered(&records, &[0], &mut ledger).unwrap();

        assert_eq!(digest, source_object_set_digest(&records).unwrap());
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed],
            codefire_store::canonical_json(&Value::Array(records))
                .unwrap()
                .len() as u64
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
        assert!(error.to_string().contains("no selected response"));
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
        assert_eq!(exact.method, "GET");

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
