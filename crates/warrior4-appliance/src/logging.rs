//! Application logging setup

use std::{
    ffi::OsString,
    io::Read,
    path::Path,
    process::{Command, ExitStatus, Output, Stdio},
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

pub fn trace_command_output(command: &mut Command) -> anyhow::Result<Output> {
    let program = command.get_program().to_owned();
    let args = command
        .get_args()
        .map(|s| s.to_owned())
        .collect::<Vec<OsString>>();
    let output = command.output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    tracing::trace!(?program, ?args, %stderr, %stdout, exit_status = %output.status, "command output");

    Ok(output)
}

pub fn monitor_command_output<C>(
    command: &mut Command,
    output_callback: C,
) -> anyhow::Result<ExitStatus>
where
    C: Fn(&[u8]) + Send,
{
    let mut output = Vec::new();

    let program = command.get_program().to_owned();
    let args = command
        .get_args()
        .map(|s| s.to_owned())
        .collect::<Vec<OsString>>();

    let mut child = command
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    read_std_streams(stdout, stderr, |data| {
        output.extend_from_slice(data);
        output_callback(&output)
    })?;

    let exit_status = child.wait()?;
    let output = String::from_utf8_lossy(&output);

    tracing::debug!(?program, ?args, %output, %exit_status, "command output");
    tracing::info!(?program, ?args, %exit_status, "command exited");

    Ok(exit_status)
}

fn read_std_streams<C>(
    mut stdout: std::process::ChildStdout,
    mut stderr: std::process::ChildStderr,
    mut callback: C,
) -> std::io::Result<()>
where
    C: FnMut(&[u8]),
{
    let (sender, receiver) = std::sync::mpsc::sync_channel(8);
    let sender2 = sender.clone();

    let out_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();

        loop {
            buf.resize(4096, 0);
            let len = stdout.read(&mut buf)?;
            buf.truncate(len);

            if len == 0 {
                break;
            }

            sender.send(buf.clone()).unwrap();
        }

        Ok::<(), std::io::Error>(())
    });

    let err_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();

        loop {
            buf.resize(4096, 0);
            let len = stderr.read(&mut buf)?;
            buf.truncate(len);

            if len == 0 {
                break;
            }

            sender2.send(buf.clone()).unwrap();
        }

        Ok::<(), std::io::Error>(())
    });

    while let Ok(data) = receiver.recv() {
        tracing::trace!(command_output_data = ?&data);
        callback(&data);
    }

    out_handle.join().unwrap()?;
    err_handle.join().unwrap()?;

    Ok(())
}

pub fn get_last_line(text: &str) -> String {
    text.lines().last().unwrap_or_default().to_string()
}
