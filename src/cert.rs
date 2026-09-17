use std::{
    fs,
    io::{self, IsTerminal as _, Write as _},
    net::IpAddr,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedKey, DistinguishedName, DnType, ExtendedKeyUsagePurpose, Ia5String, IsCa, KeyPair,
    KeyUsagePurpose, SanType,
};

use crate::config::{
    cli::{CertCommands, CertType},
    project_config_dir,
};

const CA_NAME: &str = "ca";

pub(crate) fn run(command: &CertCommands) -> Result<()> {
    match command {
        CertCommands::Generate {
            cert_type,
            name,
            ip,
            dns,
            uri,
            email,
            force,
        } => generate(cert_type, name, ip, dns, uri, email, *force),
        CertCommands::Print { name } => print(name),
    }
}

fn generate(
    cert_type: &CertType,
    name: &str,
    ips: &[IpAddr],
    dns_names: &[String],
    uris: &[String],
    emails: &[String],
    force: bool,
) -> Result<()> {
    validate_leaf_name(name)?;
    let base_path = certs_path();
    let cert_path = base_path.join(format!("{name}.pem"));
    let key_path = base_path.join(format!("{name}_key.pem"));

    if (cert_path.exists() || key_path.exists()) && !force && !confirm_replace(&cert_path, &key_path)? {
        eprintln!("Certificate was not replaced.");
        return Ok(());
    }

    fs::create_dir_all(&base_path).with_context(|| format!("Failed to create certificate directory '{}'", base_path.display()))?;
    let ca = load_or_create_ca(&base_path)?;
    let params = leaf_params(cert_type, ips, dns_names, uris, emails)?;
    let key_pair = KeyPair::generate().context("Failed to generate certificate private key")?;
    let cert = params
        .signed_by(&key_pair, &ca.cert, &ca.key_pair)
        .context("Failed to sign certificate with the Protoglot CA")?;

    write_pair(&cert_path, &key_path, &cert.pem(), &key_pair.serialize_pem())?;
    println!("Wrote certificate to {}", cert_path.display());
    println!("Wrote private key to {}", key_path.display());
    Ok(())
}

fn leaf_params(
    cert_type: &CertType,
    ips: &[IpAddr],
    dns_names: &[String],
    uris: &[String],
    emails: &[String],
) -> Result<CertificateParams> {
    let mut params = CertificateParams::default();
    let mut distinguished_name = DistinguishedName::new();
    let (common_name, extended_usage) = match cert_type {
        CertType::Server => ("Protoglot Test Server", ExtendedKeyUsagePurpose::ServerAuth),
        CertType::Client => ("Protoglot Test Client", ExtendedKeyUsagePurpose::ClientAuth),
    };
    distinguished_name.push(DnType::CommonName, common_name);
    params.distinguished_name = distinguished_name;
    set_validity(&mut params)?;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.extended_key_usages.push(extended_usage);
    params.use_authority_key_identifier_extension = true;

    params.subject_alt_names.extend(ips.iter().copied().map(SanType::IpAddress));
    for value in dns_names {
        params.subject_alt_names.push(SanType::DnsName(to_ia5("DNS", value)?));
    }
    for value in uris {
        params.subject_alt_names.push(SanType::URI(to_ia5("URI", value)?));
    }
    for value in emails {
        params.subject_alt_names.push(SanType::Rfc822Name(to_ia5("email", value)?));
    }

    Ok(params)
}

fn to_ia5(kind: &str, value: &str) -> Result<Ia5String> {
    value
        .to_string()
        .try_into()
        .with_context(|| format!("Invalid {kind} subject alternative name '{value}'"))
}

fn load_or_create_ca(base_path: &Path) -> Result<CertifiedKey> {
    let ca_path = base_path.join(CA_NAME);
    let cert_path = ca_path.join("ca.pem");
    let key_path = ca_path.join("ca_key.pem");

    match (cert_path.exists(), key_path.exists()) {
        (true, true) => load_ca(&cert_path, &key_path),
        (false, false) => {
            fs::create_dir_all(&ca_path).with_context(|| format!("Failed to create CA directory '{}'", ca_path.display()))?;
            let ca = generate_ca()?;
            write_pair(&cert_path, &key_path, &ca.cert.pem(), &ca.key_pair.serialize_pem())?;
            eprintln!("Created Protoglot test CA at {}", cert_path.display());
            Ok(ca)
        }
        _ => anyhow::bail!(
            "Incomplete Protoglot CA: expected both '{}' and '{}'; restore or remove the remaining file",
            cert_path.display(),
            key_path.display()
        ),
    }
}

fn generate_ca() -> Result<CertifiedKey> {
    let mut params = CertificateParams::default();
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, "Protoglot Test CA");
    params.distinguished_name = distinguished_name;
    set_validity(&mut params)?;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages.extend([
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ]);
    let key_pair = KeyPair::generate().context("Failed to generate CA private key")?;
    let cert = params
        .self_signed(&key_pair)
        .context("Failed to generate self-signed CA certificate")?;
    Ok(CertifiedKey { cert, key_pair })
}

fn load_ca(cert_path: &Path, key_path: &Path) -> Result<CertifiedKey> {
    let cert_pem = fs::read_to_string(cert_path).with_context(|| format!("Failed to read CA certificate '{}'", cert_path.display()))?;
    let key_pem = fs::read_to_string(key_path).with_context(|| format!("Failed to read CA key '{}'", key_path.display()))?;
    let params = CertificateParams::from_ca_cert_pem(&cert_pem)
        .with_context(|| format!("Failed to parse CA certificate '{}'", cert_path.display()))?;
    if !matches!(params.is_ca, IsCa::Ca(_)) {
        anyhow::bail!("Certificate '{}' is not a CA certificate", cert_path.display());
    }
    let key_pair = KeyPair::from_pem(&key_pem).with_context(|| format!("Failed to parse CA key '{}'", key_path.display()))?;
    let cert = params.self_signed(&key_pair).with_context(|| {
        format!(
            "Failed to use CA certificate and key in '{}'",
            cert_path.parent().unwrap_or(cert_path).display()
        )
    })?;
    Ok(CertifiedKey { cert, key_pair })
}

fn set_validity(params: &mut CertificateParams) -> Result<()> {
    let day = Duration::from_secs(86_400);
    let year = day * 365;
    params.not_before = SystemTime::now()
        .checked_sub(day)
        .context("System clock is too early to set certificate validity")?
        .into();
    params.not_after = SystemTime::now()
        .checked_add(year)
        .context("Certificate validity exceeds the supported system time")?
        .into();
    Ok(())
}

fn validate_leaf_name(name: &str) -> Result<()> {
    if name == CA_NAME {
        anyhow::bail!("The certificate name 'ca' is reserved for the Protoglot signing CA");
    }
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
        || name == "."
        || name == ".."
    {
        anyhow::bail!("Invalid certificate name '{name}'; use only ASCII letters, numbers, '.', '_', and '-'");
    }
    Ok(())
}

fn confirm_replace(cert_path: &Path, key_path: &Path) -> Result<bool> {
    let warning = format!("WARNING: '{}' or '{}' already exists.", cert_path.display(), key_path.display());
    if io::stderr().is_terminal() {
        eprintln!("\x1b[33m{warning}\x1b[0m");
    } else {
        eprintln!("{warning}");
    }
    eprint!("Replace both certificate and key? [y/N] ");
    io::stderr().flush().context("Failed to display replacement prompt")?;

    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return Ok(false);
    }
    Ok(matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
}

fn print(name: &str) -> Result<()> {
    let cert_path = if name == CA_NAME {
        certs_path().join("ca/ca.pem")
    } else {
        validate_leaf_name(name)?;
        certs_path().join(format!("{name}.pem"))
    };
    let pem = fs::read(&cert_path).with_context(|| format!("Failed to read certificate '{}'", cert_path.display()))?;
    io::stdout()
        .write_all(&pem)
        .with_context(|| format!("Failed to print certificate '{}'", cert_path.display()))
}

fn certs_path() -> PathBuf {
    project_config_dir().join("certs")
}

fn write_pair(cert_path: &Path, key_path: &Path, cert_pem: &str, key_pem: &str) -> Result<()> {
    let cert_temp = temporary_path(cert_path);
    let key_temp = temporary_path(key_path);
    fs::write(&cert_temp, cert_pem).with_context(|| format!("Failed to write temporary certificate '{}'", cert_temp.display()))?;
    if let Err(error) = write_private_key(&key_temp, key_pem) {
        let _ = fs::remove_file(&cert_temp);
        return Err(error);
    }

    let result = (|| {
        replace_file(&cert_temp, cert_path)?;
        replace_file(&key_temp, key_path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&cert_temp);
        let _ = fs::remove_file(&key_temp);
    }
    result
}

fn temporary_path(path: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let filename = path.file_name().and_then(|name| name.to_str()).unwrap_or("certificate");
    path.with_file_name(format!(".{filename}.tmp-{}-{nonce}", std::process::id()))
}

fn replace_file(source: &Path, destination: &Path) -> Result<()> {
    #[cfg(windows)]
    if destination.exists() {
        fs::remove_file(destination).with_context(|| format!("Failed to replace '{}'", destination.display()))?;
    }
    fs::rename(source, destination).with_context(|| format!("Failed to replace '{}'", destination.display()))
}

fn write_private_key(path: &Path, pem: &str) -> Result<()> {
    fs::write(path, pem).with_context(|| format!("Failed to write temporary private key '{}'", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to secure private key '{}'", path.display()))?;
    }
    Ok(())
}
