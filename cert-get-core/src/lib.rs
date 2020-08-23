use std::net::TcpStream;
use std::path::{Path, PathBuf};

use log::{info, error};
use openssl::ssl::{
    SslStream,
    SslConnector,
    SslConnectorBuilder,
    SslMethod,
    SslVerifyMode,
};
use openssl::stack::StackRef;
use openssl::x509::X509;
use openssl::nid::Nid;

pub fn map_io_err(err: std::io::Error) -> String {
    format!("io: {}", err)
}

pub(crate) fn map_openssl_err(err: openssl::error::ErrorStack) -> String {
    format!("openssl: {}", err)
}

pub fn download_certs<P: AsRef<Path> + std::fmt::Debug>(url: &str, output_dir: P) -> Result<(), String> {
    openssl_probe::init_ssl_cert_env_vars();

    let connector: SslConnector = new_insecure_ssl_connector()?;

    let stream: TcpStream = TcpStream::connect(&url)
        .map_err(map_io_err)?;

    let stream: SslStream<TcpStream> = connector.connect(&url, stream).map_err(|openssl_err| {
        format!("openssl: handshake: {}", openssl_err)
    })?;

    let maybe_certs: Option<&StackRef<X509>> = stream.ssl().peer_cert_chain();
    if maybe_certs.is_none() {
        error!("it was not possible to get certificate chain from server");
        std::process::exit(1);
    }
    let cert_stack: &StackRef<X509> = maybe_certs.unwrap();

    info!("got {} certificate(s).", cert_stack.len());

    for (i, cert) in cert_stack.iter().enumerate() {
        let cert: X509 = cert.to_owned();
        let common_name = match cert_common_name(&cert) {
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
                    output_dir,
                    file_name,
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
    let mut connector_builder: SslConnectorBuilder = SslConnector::builder(SslMethod::tls())
        .map_err(map_openssl_err)?;

    connector_builder.set_verify(SslVerifyMode::NONE);
    // connector_builder.set_default_verify_paths();

    Ok(connector_builder.build())
}

pub fn save_cert<P: AsRef<Path>>(file_path: P, cert: X509) -> Result<(), String> {
    let pem_data: Vec<u8> = cert.to_pem().map_err(|openssl_error_stack| {
        format!("openssl: pem encoding: {:?}", openssl_error_stack.errors())
    })?;

    std::fs::write(file_path, &pem_data).map_err(|ioerr| {
        format!("fs: create/write: io: {:?}", ioerr)
    })?;

    Ok(())
}

pub fn cert_common_name(cert: &X509) -> Result<String, String> {

    let name_entries = cert.subject_name().entries();
    for name_entry in name_entries {
        let obj = name_entry.object();
        if obj.nid() == Nid::COMMONNAME {
            return name_entry.data()
                .as_utf8()
                .map(|openssl_str| openssl_str.to_string())
                .map_err(|openssl_error_stack| {
                    format!("openssl: utf-8 parsing: {:?}", openssl_error_stack.errors())
                });
        }
    }

    Err(format!("common name não encontrado"))
}

// May be useful when generating jks's
// let status: std::process::ExitStatus = process::Command::new("python3")
//     .env("PYTHONPATH", ".")
//     .arg("scripts/ceph-delete-bucket.py")
//     .arg(bucket_name)
//     .status()
//     .map_err(|_e: std::io::Error| "Não foi possível iniciar processo python".to_string())?;

// if status.success() {
//     Ok(())
// } else {
//     let error_message = String::from("Processo filho retornou com erro");
//     error(&error_message);
//     Err(error_message)
// }