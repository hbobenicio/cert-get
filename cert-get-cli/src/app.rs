use log::debug;
use cert_get_core as core;

const ARG_HELP: &'static str = "HELP";
const ARG_INTERACTIVE: &'static str = "INTERACTIVE";
const ARG_HOST: &'static str = "HOST";
const ARG_PORT: &'static str = "PORT";
const ARG_OUTPUT_DIR: &'static str = "OUTPUT_DIR";

struct CLIContext {
    host: String,
    port: String,
    addr: String,
    output_dir: std::path::PathBuf,
}

pub fn run() -> Result<(), String> {
    let arg_matches = clap_app_new().get_matches();

    let cli_context = if arg_matches.is_present(ARG_INTERACTIVE) {
        debug!("interactive mode");

        let host: String = dialoguer::Input::new()
            .with_prompt("Server host/ip")
            .interact()
            .map_err(cert_get_core::map_io_err)?;

        let port: String = dialoguer::Input::new()
            .with_prompt("Server port")
            .default(String::from("443"))
            .interact()
            .map_err(cert_get_core::map_io_err)?;

        let output_dir: String = dialoguer::Input::new()
            .with_prompt("Output directory")
            .default(String::from("."))
            .interact()
            .map_err(cert_get_core::map_io_err)?;
        let output_dir = std::path::Path::new(&output_dir).to_owned();

        let addr = format!("{}:{}", host, port);

        CLIContext { host, port, addr, output_dir }

    } else {
        debug!("non-interactive mode");

        // Unwraps são seguros, pois:
        // - host é obrigatório
        // - port não é obrigatório, mas possui valor default
        // - output_dir não é obrigatório, mas possui valor default
        let host = arg_matches.value_of(ARG_HOST).unwrap().to_string();
        let port = arg_matches.value_of(ARG_PORT).unwrap().to_string();
        let output_dir = arg_matches.value_of(ARG_OUTPUT_DIR).unwrap().to_string();
        let output_dir = std::path::Path::new(&output_dir).to_owned();

        let addr = format!("{}:{}", host, port);

        CLIContext { host, port, addr, output_dir }
    };

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.enable_steady_tick(100);
    spinner.set_message("downloading certificates...");
    // For more spinners check out the cli-spinners project:
    // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
    spinner.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_strings(&["🕛","🕐","🕑","🕒","🕓","🕔","🕕","🕖","🕗","🕘","🕙","🎉"])
            .template("{spinner:.blue} {msg}")
    );

    core::download_certs(&cli_context.addr, &cli_context.output_dir)?;

    spinner.finish_with_message("done.");
    Ok(())
}

fn clap_app_new<'a>() -> clap::App<'a, 'a> {
    clap::App::new("cert-get")
        .version("0.1.0")
        .author("Hugo Benício Miranda de Oliveira <hbobenicio@gmail.com>")
        .about("CLI utility for downloading HTTPS servers certificates")
        .arg(clap::Arg::with_name(ARG_HELP)
            .help("display the help text about how this utility works")
            .long("help")
            .required(false)
            .takes_value(false)
        )
        .arg(clap::Arg::with_name(ARG_INTERACTIVE)
            .help("enter interactive mode")
            .short("i")
            .long("interactive")
            .required(false)
            .takes_value(false)
        )
        .arg(clap::Arg::with_name(ARG_HOST)
            .help("Servers host/ip")
            .short("h")
            .long("host")
            .required_unless(ARG_INTERACTIVE)
            .value_name(ARG_HOST)
            .takes_value(true)
        )
        .arg(clap::Arg::with_name(ARG_PORT)
            .help("Servers port")
            .short("p")
            .long("port")
            .required(false)
            .value_name(ARG_PORT)
            .takes_value(true)
            .default_value("443")
        )
        .arg(clap::Arg::with_name(ARG_OUTPUT_DIR)
            .help("Output directory where certificates will be saved")
            .short("o")
            .long("output-dir")
            .required(false)
            .value_name(ARG_OUTPUT_DIR)
            .default_value(".")
        )
}
