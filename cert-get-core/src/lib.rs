use std::net::TcpStream;

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

pub(crate) fn map_io_err(err: std::io::Error) -> String {
    format!("io: {}", err)
}

pub(crate) fn map_openssl_err(err: openssl::error::ErrorStack) -> String {
    format!("openssl: {}", err)
}


pub fn download_certs(url: &str, _output_dir: &str) -> Result<(), String> {
    openssl_probe::init_ssl_cert_env_vars();

    let mut connector_builder: SslConnectorBuilder = SslConnector::builder(SslMethod::tls())
        .map_err(map_openssl_err)?;

    connector_builder.set_verify(SslVerifyMode::NONE);

    let connector: SslConnector = connector_builder.build();

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
                error!("não foi possível obter o common name do certificado: {}", err);
                continue;
            }
        };
        
        let file_name = &format!("{:02}-{}.pem", i, common_name);
        
        if let Err(err) = save_cert(file_name, cert) {
            error!("{}: {:?} -> {} [ERR: {}]", i, common_name, file_name, err);
            continue;
        } else {
            info!("{}: {:?} -> {} [OK]", i, common_name, file_name);
        }
    }

    Ok(())
}

pub fn save_cert(file_name: &str, cert: X509) -> Result<(), String> {
    let pem_data: Vec<u8> = cert.to_pem().map_err(|openssl_error_stack| {
        format!("openssl: pem encoding: {:?}", openssl_error_stack.errors())
    })?;

    std::fs::write(file_name, &pem_data).map_err(|ioerr| {
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
