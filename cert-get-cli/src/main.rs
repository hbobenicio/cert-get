//! cert-get-cli is a CLI utility for downloading certificates from HTTPS servers.

use log::error;

mod app;
mod progress;

/// Main function.
fn main() {
    log_init();

    if let Err(err) = app::run() {
        error!("{}", err);
        std::process::exit(1);
    }
}

/// log_init is responsible for the logging setup.
fn log_init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
}
