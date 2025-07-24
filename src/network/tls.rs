use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::{io::Error as IoError, sync::Arc};
use tokio_rustls::TlsAcceptor;

use crate::{core::read_file, prelude::*};

#[cfg(all(debug_assertions, feature = "dev-certs"))]
pub fn generate_self_signed_cert_rust() -> Result<()> {
    if std::path::Path::new("cert.pem").exists() && std::path::Path::new("key.pem").exists() {
        return Ok(());
    }
    println!("Certificate files not found. Generating self-signed certificate with Rust...");
    let mut params: rcgen::CertificateParams =
        rcgen::CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .map_err(|e| Error::Tls(format!("Failed to create certificate params: {e}")))?;
    let mut distinguished_name: rcgen::DistinguishedName = rcgen::DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::CommonName, "localhost");
    distinguished_name.push(rcgen::DnType::OrganizationName, "Game Server");
    distinguished_name.push(rcgen::DnType::CountryName, "IR");
    params.distinguished_name = distinguished_name;
    let key_pair: rcgen::KeyPair = rcgen::KeyPair::generate()
        .map_err(|err: rcgen::Error| Error::Tls(format!("Failed to generate key pair: {err}")))?;
    let cert: rcgen::Certificate = params.self_signed(&key_pair).map_err(|err: rcgen::Error| {
        Error::Tls(format!("Failed to generate certificate: {err}"))
    })?;
    let pem_serialized: String = cert.pem();
    std::fs::write("cert.pem", pem_serialized).map_err(|err: IoError| {
        Error::FileOperation(format!("unable to write file error: {err}"))
    })?;
    std::fs::write("key.pem", key_pair.serialize_pem()).map_err(|err: IoError| {
        Error::FileOperation(format!("unable to write file error: {err}"))
    })?;
    println!("Certificate generated successfully!");
    Ok(())
}

pub fn init_crypto_provider() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");
}

fn load_tls_config() -> Result<Arc<ServerConfig>> {
    let config: &'static Config = get_config();
    let cert_file: Vec<u8> = read_file(&config.tls.cert)?;
    let key_file: Vec<u8> = read_file(&config.tls.key)?;
    let cert_chain: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_file.as_slice())
        .collect::<Result<Vec<_>, IoError>>()
        .map_err(|err: IoError| Error::Tls(format!("Failed to parse certificates: {err}")))?;
    let keys: Vec<PrivateKeyDer<'static>> =
        rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
            .map(|key| key.map(PrivateKeyDer::Pkcs8))
            .collect::<Result<Vec<_>, IoError>>()
            .map_err(|err: IoError| Error::Tls(format!("Failed to parse private keys: {err}")))?;
    if keys.is_empty() {
        return Err(Error::Tls("No private keys found".to_string()));
    }
    let private_key: PrivateKeyDer<'static> = keys
        .into_iter()
        .next()
        .ok_or_else(|| Error::Tls("No private keys available after parsing".to_string()))?;
    let tls_config: ServerConfig = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .map_err(|err: rustls::Error| Error::Tls(format!("Failed to build TLS config: {err}")))?;
    Ok(Arc::new(tls_config))
}

pub fn get_tls_acceptor() -> Result<TlsAcceptor> {
    load_tls_config().map(|config: Arc<ServerConfig>| TlsAcceptor::from(config))
}
