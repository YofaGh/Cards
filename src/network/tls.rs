use rustls::{Certificate, ServerConfig};
use std::{io::Error as IoError, sync::Arc};
use tokio_rustls::TlsAcceptor;

use crate::{core::read_file, prelude::*};

#[cfg(all(debug_assertions, feature = "dev-certs"))]
pub fn generate_self_signed_cert_rust() -> Result<()> {
    println!("Generating self-signed certificate with Rust...");
    let mut params =
        rcgen::CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()]);
    let mut distinguished_name = rcgen::DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::CommonName, "localhost");
    distinguished_name.push(rcgen::DnType::OrganizationName, "Game Server");
    distinguished_name.push(rcgen::DnType::CountryName, "IR");
    params.distinguished_name = distinguished_name;
    let cert: rcgen::Certificate = rcgen::Certificate::from_params(params).unwrap();
    let pem_serialized: String = cert.serialize_pem().unwrap();
    std::fs::write("cert.pem", pem_serialized).map_err(|err: IoError| {
        Error::FileOperation(format!("unable to write file error: {err}"))
    })?;
    std::fs::write("key.pem", cert.serialize_private_key_pem()).map_err(|err: IoError| {
        Error::FileOperation(format!("unable to write file error: {err}"))
    })?;
    println!("Certificate generated successfully!");
    Ok(())
}

fn load_tls_config() -> Result<Arc<ServerConfig>> {
    let config: &'static Config = get_config();
    let cert_file: Vec<u8> = read_file(&config.tls.cert)?;
    let key_file: Vec<u8> = read_file(&config.tls.key)?;
    let cert_chain: Vec<Certificate> = rustls_pemfile::certs(&mut cert_file.as_slice())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e: IoError| Error::Tls(format!("Failed to parse certificates: {e}")))?
        .into_iter()
        .map(|cert| Certificate(cert.to_vec()))
        .collect();
    let keys: Vec<Vec<u8>> = rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e: IoError| Error::Tls(format!("Failed to parse private keys: {e}")))?
        .into_iter()
        .map(|key| key.secret_pkcs8_der().to_vec())
        .collect();
    if keys.is_empty() {
        return Err(Error::Tls("No private keys found".to_string()));
    }
    let first_key: Vec<u8> = keys
        .into_iter()
        .next()
        .ok_or_else(|| Error::Tls("No private keys available after parsing".to_string()))?;
    let tls_config: ServerConfig = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, rustls::PrivateKey(first_key))
        .map_err(|e: rustls::Error| Error::Tls(format!("Failed to build TLS config: {e}")))?;
    Ok(Arc::new(tls_config))
}

pub fn get_tls_acceptor() -> Result<TlsAcceptor> {
    load_tls_config().map(|config: Arc<ServerConfig>| TlsAcceptor::from(config))
}
