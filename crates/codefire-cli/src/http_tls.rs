use super::CliError;
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::server::AllowAnyAuthenticatedClient;
use rustls::{
    Certificate, ClientConfig, ClientConnection, Error as TlsError, OwnedTrustAnchor, PrivateKey,
    RootCertStore, ServerConfig, ServerConnection, ServerName, StreamOwned,
};
use std::ffi::OsStr;
use std::fs;
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

pub(crate) type TlsClientStream = StreamOwned<ClientConnection, TcpStream>;
pub(crate) type TlsServerStream = StreamOwned<ServerConnection, TcpStream>;

pub(crate) fn connect_client(stream: TcpStream, host: &str) -> Result<TlsClientStream, CliError> {
    let server_name = ServerName::try_from(host)
        .map_err(|_| CliError::Usage(format!("invalid TLS server name: {host}")))?;
    let config = if tls_insecure_enabled() {
        ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(InsecureVerifier))
            .with_no_client_auth()
    } else {
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store())
            .with_no_client_auth()
    };
    let connection = ClientConnection::new(Arc::new(config), server_name)
        .map_err(|error| CliError::Usage(format!("TLS client error: {error}")))?;
    Ok(StreamOwned::new(connection, stream))
}

pub(crate) fn tls_insecure_enabled() -> bool {
    insecure_env_value_enabled(std::env::var_os("CODEFIRE_TLS_INSECURE").as_deref())
}

pub(crate) fn insecure_env_value_enabled(value: Option<&OsStr>) -> bool {
    value.is_some_and(|value| value == "1" || value == "true")
}

pub(crate) fn server_config(
    cert_path: &Path,
    key_path: &Path,
    client_ca_path: Option<&Path>,
) -> Result<Arc<ServerConfig>, CliError> {
    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;
    let builder = ServerConfig::builder().with_safe_defaults();
    let config = if let Some(client_ca_path) = client_ca_path {
        let verifier = AllowAnyAuthenticatedClient::new(load_client_ca_roots(client_ca_path)?);
        builder
            .with_client_cert_verifier(Arc::new(verifier))
            .with_single_cert(certs, key)
    } else {
        builder.with_no_client_auth().with_single_cert(certs, key)
    }
    .map_err(|error| CliError::Usage(format!("invalid TLS certificate/key: {error}")))?;
    Ok(Arc::new(config))
}

pub(crate) fn accept_server(
    stream: TcpStream,
    config: Arc<ServerConfig>,
) -> Result<TlsServerStream, CliError> {
    let connection = ServerConnection::new(config)
        .map_err(|error| CliError::Usage(format!("TLS server error: {error}")))?;
    Ok(StreamOwned::new(connection, stream))
}

fn root_store() -> RootCertStore {
    let mut roots = RootCertStore::empty();
    roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|anchor| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            anchor.subject,
            anchor.spki,
            anchor.name_constraints,
        )
    }));
    roots
}

fn load_certs(path: &Path) -> Result<Vec<Certificate>, CliError> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|_| CliError::Usage(format!("invalid certificate PEM: {}", path.display())))?;
    if certs.is_empty() {
        return Err(CliError::Usage(format!(
            "certificate PEM has no certificates: {}",
            path.display()
        )));
    }
    Ok(certs.into_iter().map(Certificate).collect())
}

fn load_client_ca_roots(path: &Path) -> Result<RootCertStore, CliError> {
    let certs = load_certs(path)?;
    let mut roots = RootCertStore::empty();
    for cert in certs {
        roots.add(&cert).map_err(|error| {
            CliError::Usage(format!(
                "invalid TLS client CA certificate {}: {error}",
                path.display()
            ))
        })?;
    }
    Ok(roots)
}

pub(crate) fn load_private_key(path: &Path) -> Result<PrivateKey, CliError> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let pkcs8_keys = rustls_pemfile::pkcs8_private_keys(&mut reader).map_err(|_| {
        CliError::Usage(format!(
            "invalid PKCS#8 private key PEM: {}",
            path.display()
        ))
    })?;
    if let Some(key) = pkcs8_keys.into_iter().next() {
        return Ok(PrivateKey(key));
    }

    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let rsa_keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| CliError::Usage(format!("invalid RSA private key PEM: {}", path.display())))?;
    if let Some(key) = rsa_keys.into_iter().next() {
        return Ok(PrivateKey(key));
    }

    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let ec_keys = rustls_pemfile::ec_private_keys(&mut reader).map_err(|_| {
        CliError::Usage(format!(
            "invalid SEC1 EC private key PEM: {}",
            path.display()
        ))
    })?;
    ec_keys.into_iter().next().map(PrivateKey).ok_or_else(|| {
        CliError::Usage(format!(
            "private key PEM has no supported keys (PKCS#8, RSA, SEC1 EC): {}",
            path.display()
        ))
    })
}

struct InsecureVerifier;

impl ServerCertVerifier for InsecureVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }
}
