use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod adapter;
mod builder;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let config =
        config::load_config(&args.target_config).context("loading target config failed")?;

    match check::check_network(&config).await {
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
