//! error is the responsible for error handling stuff.

/// Maps an IO Error to a String error with a generic context.
pub fn map_io_err(err: std::io::Error) -> String {
    format!("io: {}", err)
}

/// Maps an OpenSSL error to a String error with a generic context.
pub fn map_openssl_err(err: openssl::error::ErrorStack) -> String {
    format!("openssl: {}", err)
}
