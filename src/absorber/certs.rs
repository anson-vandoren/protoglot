use std::{
    io::{Cursor, Read as _},
    path::PathBuf,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use flate2::bufread::GzDecoder;
use log::{debug, trace};
use rcgen::{BasicConstraints, CertificateParams, CertifiedKey, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose};
use reqwest::Client;
use rustls::pki_types::{pem::PemObject as _, CertificateDer, PrivateKeyDer};
use tar::Archive;

use super::CertType;

pub(super) struct CertKey {
    key_pem: String,
    cert_pem: String,
}

impl CertKey {
    pub fn cert(&self) -> Vec<CertificateDer<'static>> {
        CertificateDer::pem_slice_iter(self.cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse certificate PEM data")
    }

    pub fn key(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::from_pem_slice(self.key_pem.as_bytes()).expect("Failed to parse key PEM data")
    }
}

impl From<CertifiedKey> for CertKey {
    fn from(value: CertifiedKey) -> Self {
        CertKey {
            key_pem: value.key_pair.serialize_pem(),
            cert_pem: value.cert.pem(),
        }
    }
}

pub(super) async fn get_cert(cert_type: &CertType) -> Result<Option<CertKey>> {
    match cert_type {
        CertType::None => Ok(None),
        CertType::SelfSigned => Ok(Some(gen_self_signed()?)),
        CertType::PublicCA => Ok(Some(pull_public_certs().await?)),
        CertType::PrivateCA => Ok(Some(gen_private_ca()?)),
    }
}

fn generate_cert(ca: Option<&CertifiedKey>, is_ca: bool) -> Result<CertifiedKey> {
    let sans = match is_ca {
        false => vec!["localhost".to_string(), "local.fucktls.com".to_string()],
        true => vec![],
    };
    let mut params = CertificateParams::new(sans)?;

    // set the validity period
    let (start, end) = validity_interval();
    params.not_before = start.into();
    params.not_after = end.into();

    // set the allowable usages
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);

    if is_ca {
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages.push(KeyUsagePurpose::KeyCertSign);
        params.key_usages.push(KeyUsagePurpose::CrlSign);
    } else {
        params.use_authority_key_identifier_extension = true;
        params.extended_key_usages.push(ExtendedKeyUsagePurpose::ServerAuth);
    }

    let key_pair = KeyPair::generate()?;
    let cert = match ca {
        Some(ca) => params.signed_by(&key_pair, &ca.cert, &ca.key_pair)?,
        None => params.self_signed(&key_pair)?,
    };

    Ok(CertifiedKey { cert, key_pair })
}

fn gen_self_signed() -> Result<CertKey> {
    let cert_key = generate_cert(None, false)?;
    save_and_print_certs(&cert_key, None)?;
    debug!("Generated self-signed cert");
    Ok(cert_key.into())
}

fn gen_private_ca() -> Result<CertKey> {
    let ca = generate_cert(None, true)?;
    let cert_key = generate_cert(Some(&ca), false)?;
    save_and_print_certs(&cert_key, Some(&ca))?;
    debug!("Generated private CA & server cert");
    Ok(cert_key.into())
}

async fn pull_public_certs() -> Result<CertKey> {
    let client = Client::new();
    let res = client.get("https://fucktls.com/certs.tar.gz").send().await?;
    if !res.status().is_success() {
        anyhow::bail!("Failed to download certs: {:?}", res);
    }
    trace!("Downloaded public certs for local.fucktls.com");

    let bytes = res.bytes().await?;
    let cursor = Cursor::new(bytes);

    let gz_decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz_decoder);

    let mut key_pem = String::new();
    let mut cert_pem = String::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        if let Some(filename) = path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str == "privkey.pem" {
                    trace!("Found private key in public certs");
                    entry.read_to_string(&mut key_pem)?;
                } else if filename_str == "fullchain.pem" {
                    trace!("Found cert chain in public certs");
                    entry.read_to_string(&mut cert_pem)?;
                }
            }
        }
    }

    Ok(CertKey { cert_pem, key_pem })
}

fn validity_interval() -> (SystemTime, SystemTime) {
    let day = Duration::new(86400, 0);
    let year = day * 365;

    let yesterday = SystemTime::now().checked_sub(day).unwrap();
    let next_year = SystemTime::now().checked_add(year).unwrap();

    (yesterday, next_year)
}

const DEFAULT_CERT_PATH: &str = "/tmp/protoglot";

fn save_and_print_certs(cert_key: &CertifiedKey, ca_cert: Option<&CertifiedKey>) -> Result<()> {
    let base_path = PathBuf::from(DEFAULT_CERT_PATH);
    std::fs::create_dir_all(&base_path)?;

    let cert = cert_key.cert.pem();
    let cert_path = base_path.join("server_cert.pem");
    println!("Writing server cert to: {}\n\n{}", cert_path.display(), cert);
    std::fs::write(cert_path, cert)?;

    if let Some(ca_cert) = ca_cert {
        let ca_path = base_path.join("ca_cert.pem");
        let ca_cert = ca_cert.cert.pem();
        println!("Writing CA cert to: {}\n\n{}", ca_path.display(), ca_cert);
        std::fs::write(ca_path, ca_cert)?;
    }
    Ok(())
}
