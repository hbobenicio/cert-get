//! cert-get-core is responsible for the main logic of cert-get.

pub mod error;

use std::net::TcpStream;
use std::path::{Path, PathBuf};

use log::{error, info};
use openssl::nid::Nid;
use openssl::ssl::{SslConnector, SslConnectorBuilder, SslMethod, SslStream, SslVerifyMode};
use openssl::stack::StackRef;
use openssl::x509::{X509Ref, X509};

use crate::error::{map_io_err, map_openssl_err};

/// DownloadParams represents options for the download of server certificates
pub struct DownloadParams {
    /// address is the address of the server in the format: "HOST:PORT" or "IP:PORT"
    pub address: String,

    /// output_dir is the filesystem directory path where certificates will be downloaded to.
    pub output_dir: String,

    /// insecure tells to skip TLS validation
    pub insecure: bool,

    /// generate_jks flags the optional generation of the JKS output file
    pub generate_jks: bool,
}

/// Get a vec of certificates from a https server for a given url.
pub fn get_certs(url: &str, insecure: bool) -> Result<Vec<X509>, String> {
    openssl_probe::init_ssl_cert_env_vars();

    let connector: SslConnector = new_ssl_connector(insecure)?;

    let stream: TcpStream = TcpStream::connect(&url).map_err(map_io_err)?;

    let stream: SslStream<TcpStream> = connector
        .connect(&url, stream)
        .map_err(|openssl_err| format!("openssl: handshake: {}", openssl_err))?;

    let cert_stack: &StackRef<X509> = stream.ssl().peer_cert_chain().ok_or(String::from(
        "it was not possible to get certificate chain from server",
    ))?;

    let certs: Vec<X509> = cert_stack.iter().map(X509Ref::to_owned).collect();

    Ok(certs)
}

/// Download all certificates from a https server
pub fn download_certs(params: &DownloadParams) -> Result<(), String> {
    let output_dir = &params.output_dir;

    info!("downloading certificates...");
    let certs = get_certs(&params.address, params.insecure)?;
    info!("got {} certificate(s).", certs.len());

    let mut output_dir_path = PathBuf::new();
    output_dir_path.push(&output_dir);

    info!("ensuring output directory exists...");
    std::fs::create_dir_all(&output_dir_path).map_err(|err: std::io::Error| {
        format!(
            "could not ensure output directory \"{}\" exists: {}",
            output_dir,
            map_io_err(err),
        )
    })?;

    let mut full_cert_chain_bytes: Vec<u8> = Vec::new();
    for (i, cert) in certs.iter().enumerate() {
        let common_name = match cert_common_name(cert) {
            Ok(cn) => cn,
            Err(err) => {
                error!(
                    "it was not possible to get common name from certificate: {}",
                    err
                );
                continue;
            }
        };

        let file_name = &format!("{:02}-{}", i, common_name);

        let mut file_path = output_dir_path.clone();
        file_path.push(file_name);
        file_path.set_extension("pem");

        let file_path_str = match file_path.to_str() {
            Some(s) => s,
            None => {
                let err_msg = format!(
                    "non utf-8 characters found on output file path: output_dir={}, file_name={}",
                    output_dir, file_name,
                );
                return Err(err_msg);
            }
        };

        if let Err(err) = save_cert(&file_path, cert) {
            error!(
                "{}: {:?} -> {} [ERR: {}]",
                i, common_name, file_path_str, err
            );
            continue;
        }

        // Append certificate to the chain
        // TODO this is duplicate. Done in save_cert too. Improve it
        let mut pem_data: Vec<u8> = cert.to_pem().map_err(|openssl_error_stack| {
            format!("openssl: pem encoding: {:?}", openssl_error_stack.errors())
        })?;

        full_cert_chain_bytes.append(&mut pem_data);

        info!("{}: {:?} -> {} [OK]", i, common_name, file_path_str);
    }

    let file_name = "fullchain";
    let mut file_path = output_dir_path.clone();
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
    std::fs::write(&file_path, &full_cert_chain_bytes).map_err(|ioerr| {
        format!(
            "could not save fullchain certificate file: fs: io: {:?}",
            ioerr
        )
    })?;
    info!("{}: {:?} -> {} [OK]", certs.len(), file_name, file_path_str);

    if params.generate_jks {
        info!("generating truststore...");

        // TODO get password as input from user
        generate_truststore(&certs, &params.output_dir, "changeit")?;
    }

    Ok(())
}

fn new_ssl_connector(insecure: bool) -> Result<SslConnector, String> {
    let mut connector_builder: SslConnectorBuilder =
        SslConnector::builder(SslMethod::tls()).map_err(map_openssl_err)?;

    if insecure {
        connector_builder.set_verify(SslVerifyMode::NONE);
    } else {
        connector_builder.set_default_verify_paths();
    }

    Ok(connector_builder.build())
}

pub fn save_cert<P: AsRef<Path>>(file_path: P, cert: &X509) -> Result<(), String> {
    let pem_data: Vec<u8> = cert.to_pem().map_err(|openssl_error_stack| {
        format!("openssl: pem encoding: {:?}", openssl_error_stack.errors())
    })?;

    std::fs::write(file_path, &pem_data).map_err(|ioerr| format!("fs: io: {:?}", ioerr))?;

    Ok(())
}

/// Returns the common name of the certificate
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

pub fn merge_certificates() {
    unimplemented!();
}

/// Generates a JKS file from the downloaded certificates
/// TODO Rethink this interface... avoid code duplications and be more flexible
/// TODO We need: absolute file paths of certificate files (pem), aliases names, output file path (jks)
pub fn generate_truststore<P>(
    certs: &[X509], // TODO make this a list of paths
    output_file_path: P,
    password: &str,
) -> Result<(), String>
where
    P: AsRef<Path>,
{
    // # Alias should not be equal
    // # If keystore does not exist, keytool will create it for us
    // keytool -importcert -noprompt -alias demo -keystore google.jks -file 00-\*.google.pem
    // keytool -importcert -noprompt -alias demo2 -keystore google.jks -file 01-GTS\ CA\ 1O1.pem
    for (i, cert) in certs.iter().enumerate() {
        // TODO move this to a function and remove this duplicated stuff
        let common_name = match cert_common_name(cert) {
            Ok(cn) => cn,
            Err(err) => {
                error!(
                    "it was not possible to get common name from certificate: {}",
                    err
                );
                continue;
            }
        };

        let file_name = &format!("{:02}-{}", i, common_name);

        let mut file_path = output_dir_path.clone();
        file_path.push(file_name);
        file_path.set_extension("pem");

        let file_path_str = match file_path.to_str() {
            Some(s) => s,
            None => {
                let err_msg = format!(
                    "non utf-8 characters found on output file path: output_dir={}, file_name={}",
                    output_dir, file_name,
                );
                return Err(err_msg);
            }
        };

        let status: std::process::ExitStatus = std::process::Command::new("keytool")
            .arg("-importcert")
            // .arg("-noprompt")
            // .arg("-trustcacerts")
            .arg("-alias")
            .arg(common_name)
            .arg("-keystore")
            .arg(output_file_path.as_ref())
            .arg("-file")
            .arg(file_path_str)
            .arg("-storepass")
            .arg(password)
            .status()
            .map_err(|err: std::io::Error| {
                format!(
                    "it was not possible to call keytool command: {}",
                    map_io_err(err)
                )
            })?;
        if !status.success() {
            return Err(format!(
                "keytool failed: {:?} - {}",
                status.code(),
                status.to_string()
            ));
        }
    }
    Ok(())
}
