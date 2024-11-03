use clap::Parser;

mod config;

/// Waystone: Load balancer and rate limiter supporting HTTP and WebSocket
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Config path
    #[arg(short, long, default_value = "./config.yaml")]
    config: String,
}

fn main() {
    let cliArgs = CliArgs::parse();
    dbg!(cliArgs.config);
}
