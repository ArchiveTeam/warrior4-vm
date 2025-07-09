use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

mod adapter;
mod check;
mod config;

// Command line arguments
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "/usr/share/warrior4-network-check/target.json"
    )]
    target_config: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config =
        config::load_config(&args.target_config).context("loading target config failed")?;

    match check::check_network(&config) {
        Ok(report) => {
            if !report.is_pass() {
                anyhow::bail!("check failed")
            } else {
                Ok(())
            }
        }
        Err((_report, error)) => Err(error.into()),
    }
}
