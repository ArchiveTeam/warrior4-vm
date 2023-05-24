//! Helper functions to the Docker commands

use std::process::{Command, Output};

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
                tracing::debug!(%error, "get container status");
                return String::new();
            }
        }
    };

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn run_container_foreground<S: AsRef<str>>(name: S) -> anyhow::Result<(Output, Output)> {
    let mut command = Command::new("docker");
    command.arg("start").arg(name.as_ref());

    let output = crate::logging::log_command_output(&mut command)?;

    let mut command2 = Command::new("docker");
    command.arg("wait").arg(name.as_ref());

    let output2 = crate::logging::log_command_output(&mut command2)?;

    Ok((output, output2))
}

pub fn start_container<S: AsRef<str>>(name: S) -> anyhow::Result<Output> {
    let mut command = Command::new("docker");
    command.arg("start").arg(name.as_ref());

    let output = crate::logging::log_command_output(&mut command)?;

    Ok(output)
}
