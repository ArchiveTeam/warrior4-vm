//! Warrior virtual appliance manager entry point

mod config;
mod container;
mod ipc;
mod logging;
mod manager;
mod state;

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "/etc/warrior4-appliance.toml")]
    config: PathBuf,

    #[arg(long, default_value_t = false)]
    skip_machine_env_check: bool,
}

fn main() -> anyhow::Result<()> {
    match wrapped_main() {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!(?error);
            Err(error)
        }
    }
}

/// We want to log errors before exiting so this wrapper function exists.
fn wrapped_main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.skip_machine_env_check {
        exit_if_not_vm()?;
    }

    let config = config::load_config(&args.config).context("loading config failed")?;

    logging::set_up_logging(&config.log_path).context("logging setup failed")?;

    let mut manager = manager::Manager::new(config);
    manager.run()?;

    Ok(())
}

/// Check if a specific file exists, otherwise return error
fn exit_if_not_vm() -> anyhow::Result<()> {
    let release_path = Path::new("/etc/warrior4-env");

    if !release_path.exists() {
        anyhow::bail!("Warrior 4 virtual machine not detected");
    }

    Ok(())
}
