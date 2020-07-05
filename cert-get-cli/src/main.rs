use log::error;

mod app;

fn main() {
    log_init();

    if let Err(err) = app::run() {
        error!("{}", err);
        std::process::exit(1);
    }
}

fn log_init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
}
