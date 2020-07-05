use cert_get_core as core;

pub fn run() -> Result<(), String> {
    let arg_matches = clap_app_new().get_matches();

    // TODO centralizar essas &'static str em algum lugar comum
    let host = arg_matches.value_of("HOST").unwrap();
    let port = arg_matches.value_of("PORT").unwrap();
    let output_dir = arg_matches.value_of("OUTPUT_DIR").unwrap();
    let addr = format!("{}:{}", host, port);

    core::download_certs(&addr, output_dir)?;

    Ok(())
}

fn clap_app_new<'a>() -> clap::App<'a, 'a> {
    clap::App::new("cert-get")
        .version("0.1.0")
        .author("Hugo Benício Miranda de Oliveira <hbobenicio@gmail.com>")
        .about("CLI utility for downloading HTTPS servers certificates")
        .arg(clap::Arg::with_name("HOST")
            .help("Servers host/ip")
            .short("h")
            .long("host")
            .required(true)
            .value_name("HOST")
            .takes_value(true)
        )
        .arg(clap::Arg::with_name("PORT")
            .help("Servers port")
            .short("p")
            .long("port")
            .required(false)
            .value_name("PORT")
            .takes_value(true)
            .default_value("443")
        )
        .arg(clap::Arg::with_name("OUTPUT_DIR")
            .help("Output directory where certificates will be saved")
            .short("o")
            .long("output-dir")
            .required(false)
            .value_name("OUTPUT_DIR")
            .default_value(".")
        )
}
