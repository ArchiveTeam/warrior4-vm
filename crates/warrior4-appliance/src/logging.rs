//! Application logging setup

use std::{
    ffi::OsString,
    path::Path,
    process::{Command, Output},
};

use tracing::metadata::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, Layer};

/// Set up logging that goes to a file (no stderr)
pub fn set_up_logging(path: &Path) -> anyhow::Result<()> {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .compact()
        .with_filter(LevelFilter::INFO);

    let file_appender =
        tracing_appender::rolling::never(path.parent().unwrap(), path.file_name().unwrap());
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .compact()
        .with_filter(LevelFilter::DEBUG);

    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(stderr_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::debug!("logging configured");

    Ok(())
}

pub fn log_command_output(command: &mut Command) -> anyhow::Result<Output> {
    let program = command.get_program().to_owned();
    let args = command
        .get_args()
        .map(|s| s.to_owned())
        .collect::<Vec<OsString>>();
    let output = command.output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    tracing::debug!(?program, ?args, %stderr, %stdout, exit_status = %output.status, "command output");
    tracing::info!(?program, ?args, exit_status = %output.status, "command exited");

    Ok(output)
}
