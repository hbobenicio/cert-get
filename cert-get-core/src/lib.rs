use std::net::TcpStream;
use std::path::{Path, PathBuf};

use log::{error, info};
use openssl::nid::Nid;
use openssl::ssl::{SslConnector, SslConnectorBuilder, SslMethod, SslStream, SslVerifyMode};
use openssl::stack::StackRef;
use openssl::x509::{X509, X509Ref};

/// Maps an IO Error to a String error with a generic context
pub fn map_io_err(err: std::io::Error) -> String {
    format!("io: {}", err)
}

/// Maps an OpenSSL error to a String error with a generic context
pub(crate) fn map_openssl_err(err: openssl::error::ErrorStack) -> String {
    format!("openssl: {}", err)
}

/// Get a vec of certificates from a https server for a given url
pub fn get_certs(url: &str) -> Result<Vec<X509>, String> {
    openssl_probe::init_ssl_cert_env_vars();

    let connector: SslConnector = new_insecure_ssl_connector()?;

    let stream: TcpStream = TcpStream::connect(&url).map_err(map_io_err)?;

    let stream: SslStream<TcpStream> = connector
        .connect(&url, stream)
        .map_err(|openssl_err| format!("openssl: handshake: {}", openssl_err))?;

    let cert_stack: &StackRef<X509> = stream.ssl()
        .peer_cert_chain()
        .ok_or(String::from("it was not possible to get certificate chain from server"))?;

    let certs: Vec<X509> = cert_stack.iter()
        .map(X509Ref::to_owned)
        .collect();

    Ok(certs)
}

/// Download all the certificates from a https server for a given url
pub fn download_certs<P>(url: &str, output_dir: P) -> Result<(), String>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let certs = get_certs(url)?;

    info!("got {} certificate(s).", certs.len());

    for (i, cert) in certs.iter().enumerate() {
        let common_name = match cert_common_name(cert) {
            Ok(cn) => cn,
            Err(err) => {
                error!("it was not possible to get certificate's common name: {}", err);
                continue;
            }
        };

        let file_name = &format!("{:02}-{}", i, common_name);

        let mut file_path = PathBuf::new();
        file_path.push(&output_dir);
        file_path.push(file_name);
        file_path.set_extension("pem");

        let file_path_str = match file_path.to_str() {
            Some(s) => s,
            None => {
                let err_msg = format!(
                    "non utf-8 characters found on output file path: output_dir={:?}, file_name={}",
                    output_dir, file_name,
                );
                return Err(err_msg);
            }
        };

        if let Err(err) = save_cert(&file_path, cert) {
            error!("{}: {:?} -> {} [ERR: {}]", i, common_name, file_path_str, err);
            continue;
        }

        info!("{}: {:?} -> {} [OK]", i, common_name, file_path_str);
    }

    Ok(())
}

fn new_insecure_ssl_connector() -> Result<SslConnector, String> {
    let mut connector_builder: SslConnectorBuilder =
        SslConnector::builder(SslMethod::tls()).map_err(map_openssl_err)?;

    connector_builder.set_verify(SslVerifyMode::NONE);
    // connector_builder.set_default_verify_paths();

    Ok(connector_builder.build())
}

pub fn save_cert<P: AsRef<Path>>(file_path: P, cert: &X509) -> Result<(), String> {
    let pem_data: Vec<u8> = cert.to_pem().map_err(|openssl_error_stack| {
        format!("openssl: pem encoding: {:?}", openssl_error_stack.errors())
    })?;

    std::fs::write(file_path, &pem_data)
        .map_err(|ioerr| format!("fs: io: {:?}", ioerr))?;

    Ok(())
}

// Returns the common name of the certificate
pub fn cert_common_name(cert: &X509) -> Result<String, String> {
    let name_entries = cert.subject_name().entries();
    for name_entry in name_entries {
        let obj = name_entry.object();
        if obj.nid() == Nid::COMMONNAME {
            return name_entry
                .data()
                .as_utf8()
                .map(|openssl_str| openssl_str.to_string())
                .map_err(|openssl_error_stack| {
                    format!("openssl: utf-8 parsing: {:?}", openssl_error_stack.errors())
                });
        }
    }

    Err(String::from("common name not found"))
}

/// This may be useful...
pub fn _generate_truststore<P>(certs: &[X509], output_file_path: P, password: &str) -> Result<(), String>
where P: AsRef<Path>
{
    // keytool -noprompt -import -trustcacerts -alias $ALIAS -keystore $KEYSTORE -storepass $PASSWORD
    for cert in certs {
        let status: std::process::ExitStatus = std::process::Command::new("keytool")
            .arg("-noprompt")
            .arg("-import")
            .arg("-trustcacerts")
            .arg("-alias").arg(cert_common_name(cert)?)
            .arg("-keystore").arg(output_file_path.as_ref())
            .arg("-password").arg(password)
            .status()
            .map_err(|err: std::io::Error| format!("it was not possible to call keytool command: io: {}", err))?;
        if !status.success() {
            return Err(format!("keytool failed: {:?} - {}", status.code(), status.to_string()));
        }
    }
    Ok(())
}
