// use std::io::Write;

use log::debug;
use cert_get_core as core;

// use crate::progress;

const ARG_HELP: &str = "HELP";
const ARG_BATCH: &str = "BATCH";
const ARG_HOST: &str = "HOST";
const ARG_PORT: &str = "PORT";
const ARG_OUTPUT_DIR: &str = "OUTPUT_DIR";
const ARG_INSECURE: &str = "INSECURE";

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: &str = "443";
const DEFAULT_OUTPUT_DIR: &str = ".";

struct CLIContext {
    host: String,
    port: String,
    output_dir: String,
    insecure: bool,
}

impl CLIContext {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl std::convert::From<&CLIContext> for core::DownloadParams {
    fn from(cli_context: &CLIContext) -> Self {
        core::DownloadParams {
            address: cli_context.address(),
            output_dir: cli_context.output_dir.clone(),
            insecure: cli_context.insecure
        }
    }
}

pub fn run() -> Result<(), String> {
    let arg_matches = clap_app_new().get_matches();

    let cli_context = if arg_matches.is_present(ARG_BATCH) {
        parse_cli_args(&arg_matches)?
    } else {
        run_interactive_mode(&arg_matches)?
    };

    // let spinner = progress::new_clock_spinner("downloading certificates...");

    core::download_certs(&core::DownloadParams::from(&cli_context))?;

    // spinner.finish_with_message("done.");

    Ok(())
}

fn run_interactive_mode(arg_matches: &clap::ArgMatches) -> Result<CLIContext, String> {
    debug!("entering interactive mode");

    let default_host: String = arg_matches.value_of(ARG_HOST)
        .unwrap_or(DEFAULT_HOST)
        .to_string();
    let host: String = dialoguer::Input::new()
        .with_prompt("Server host/ip")
        .default(default_host)
        .interact()
        .map_err(core::error::map_io_err)?;

    let default_port: String = arg_matches.value_of(ARG_PORT)
        .unwrap_or(DEFAULT_PORT)
        .to_string();
    let port: String = dialoguer::Input::new()
        .with_prompt("Server port")
        .default(default_port)
        .interact()
        .map_err(core::error::map_io_err)?;

    let default_output_dir: String = arg_matches.value_of(ARG_OUTPUT_DIR)
        .unwrap_or(DEFAULT_OUTPUT_DIR)
        .to_string();
    let output_dir: String = dialoguer::Input::new()
        .with_prompt("Output directory")
        .default(default_output_dir)
        .interact()
        .map_err(core::error::map_io_err)?;
    // let output_dir = std::path::Path::new(&output_dir).to_owned();

    let insecure_options = ["No", "Yes"];
    let default_insecure_index: usize = if arg_matches.is_present(ARG_INSECURE) { 1 } else { 0 };
    let insecure = dialoguer::Select::new()
        .with_prompt("Skip TLS validation")
        .items(&insecure_options)
        .default(default_insecure_index)
        .interact()
        .map(|selected_index|  selected_index != 0)
        .map_err(core::error::map_io_err)?;

    Ok(CLIContext {
        host,
        port,
        output_dir,
        insecure
    })
}

fn parse_cli_args(arg_matches: &clap::ArgMatches) -> Result<CLIContext, String> {
    debug!("entering batch mode (non-interactive)");

    // Unwraps são seguros, pois:
    // - host é obrigatório
    // - port não é obrigatório, mas possui valor default
    // - output_dir não é obrigatório, mas possui valor default
    let host = arg_matches.value_of(ARG_HOST).unwrap().to_string();
    let port = arg_matches.value_of(ARG_PORT).unwrap().to_string();
    let output_dir = arg_matches.value_of(ARG_OUTPUT_DIR).unwrap().to_string();
    // let output_dir = std::path::Path::new(&output_dir).to_owned();
    let insecure = arg_matches.is_present(ARG_INSECURE);

    Ok(CLIContext {
        host,
        port,
        output_dir,
        insecure
    })
}

fn clap_app_new<'a>() -> clap::App<'a, 'a> {
    clap::App::new("cert-get")
        .version("0.1.0")
        .author("Hugo Benício Miranda de Oliveira <hbobenicio@gmail.com>")
        .about("CLI utility for downloading HTTPS servers certificates")
        .arg(
            clap::Arg::with_name(ARG_HELP)
                .help("display the help text about how this utility works")
                .long("help")
                .required(false)
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_BATCH)
                .help("enter batch mode (non-interactive)")
                .short("b")
                .long("batch")
                .required(false)
                .requires(ARG_HOST)
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_HOST)
                .help("Servers host/ip")
                .short("h")
                .long("host")
                .value_name(ARG_HOST)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_PORT)
                .help("Servers port")
                .short("p")
                .long("port")
                .required(false)
                .value_name(ARG_PORT)
                .takes_value(true)
                .default_value("443"),
        )
        .arg(
            clap::Arg::with_name(ARG_OUTPUT_DIR)
                .help("Output directory where certificates will be saved")
                .short("o")
                .long("output-dir")
                .required(false)
                .value_name(ARG_OUTPUT_DIR)
                .default_value("."),
        )
        .arg(
            clap::Arg::with_name(ARG_INSECURE)
                .help("Insecure connection (skip tls validations)")
                .short("k")
                .long("insecure")
                .required(false)
                .value_name(ARG_INSECURE)
                .takes_value(false)
        )
}
