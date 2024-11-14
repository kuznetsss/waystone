use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tracing::error;

use waystone::*;

/// Waystone: Load balancer and rate limiter supporting HTTP and WebSocket
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Config path
    #[arg(short, long, default_value = "./config.yaml")]
    config: PathBuf,
}

fn main_impl() -> Result<(), Box<dyn Error>> {
    let cli_args = CliArgs::parse();
    let config = config::Config::from_file(&cli_args.config)?;
    load_balancer::run(&config);
    Ok(())
}

fn main() -> ExitCode {
    let tracer = tracing_subscriber::fmt().with_ansi(false).finish();
    tracing::subscriber::set_global_default(tracer).unwrap();

    if let Err(e) = main_impl() {
        error!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
