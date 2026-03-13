use rustls::{ServerConfig, pki_types::CertificateDer};
use rustls_pemfile::{certs, private_key};
use std::{fs::File, io::BufReader, sync::Arc};

pub fn load_tls_config() -> Result<Arc<ServerConfig>, Box<dyn std::error::Error>> {
    // get the certificate and key files
    let cert_reader = &mut BufReader::new(File::open("certs/cert.pem")?);
    let mut key_reader = &mut BufReader::new(File::open("certs/key.pem")?);

    let cert_chain: Vec<CertificateDer> = certs(cert_reader).collect::<Result<Vec<_>, _>>()?;

    let key_der = private_key(&mut key_reader)?.ok_or("Private key not found in key.pem")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)?;

    Ok(Arc::new(config))
}
