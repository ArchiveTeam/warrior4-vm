//! Helper functions to the Docker commands

use std::process::{Command, Output};

use chrono::{DateTime, Utc};

pub fn get_container_status<S: AsRef<str>>(name: S) -> String {
    let output = {
        let result = Command::new("docker")
            .arg("inspect")
            .arg("--type=container")
            .arg("--format")
            .arg("{{.State.Status}}")
            .arg(name.as_ref())
            .output();

        match result {
            Ok(output) => output,
            Err(error) => {
                tracing::trace!(?error, "get container status");
                return String::new();
            }
        }
    };

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn get_container_exit_code<S: AsRef<str>>(name: S) -> Option<u32> {
    let result = Command::new("docker")
        .arg("inspect")
        .arg("--type=container")
        .arg("--format")
        .arg("{{.State.ExitCode}}")
        .arg(name.as_ref())
        .output();

    match result {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().parse().ok(),
        Err(error) => {
            tracing::trace!(?error, "get container exit code");
            None
        }
    }
}

pub fn get_container_finished_at<S: AsRef<str>>(name: S) -> Option<DateTime<Utc>> {
    let result = Command::new("docker")
        .arg("inspect")
        .arg("--type=container")
        .arg("--format")
        .arg("{{.State.FinishedAt}}")
        .arg(name.as_ref())
        .output();

    match result {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().parse().ok(),
        Err(error) => {
            tracing::trace!(?error, "get container finished at");
            None
        }
    }
}

pub fn run_container_foreground<S: AsRef<str>>(name: S) -> anyhow::Result<(Output, Output)> {
    let mut start_command = Command::new("docker");
    start_command.arg("start").arg(name.as_ref());

    let start_output = crate::logging::log_command_output(&mut start_command)?;

    let mut wait_command = Command::new("docker");
    wait_command.arg("wait").arg(name.as_ref());

    let wait_output = crate::logging::log_command_output(&mut wait_command)?;

    Ok((start_output, wait_output))
}

pub fn start_container<S: AsRef<str>>(name: S) -> anyhow::Result<Output> {
    let mut command = Command::new("docker");
    command.arg("start").arg(name.as_ref());

    let output = crate::logging::log_command_output(&mut command)?;

    Ok(output)
}
