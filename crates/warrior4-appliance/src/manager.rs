//! Docker container manager and system maintenance
use std::{
    os::unix::prelude::OpenOptionsExt,
    process::Command,
    time::{Duration, Instant},
};

use anyhow::Context;

use crate::{config::AppConfig, ipc::DisplayIPC, state::State};

const PATCH_FILE_PATH: &str = "/tmp/warrior4-appliance-patch";
const MAX_UNHEALTHY_TIME: Duration = Duration::from_secs(60 * 15);

pub struct Manager {
    config: AppConfig,
    state: State,
    display_ipc: DisplayIPC,
    payload_crashed: bool,
    unheathy_timestamp: Option<Instant>,
}

impl Manager {
    pub fn new(config: AppConfig) -> Self {
        let state = State::new();
        let display_ipc = DisplayIPC::new(config.display_ipc_address);
        Self {
            config,
            state,
            display_ipc,
            payload_crashed: false,
            unheathy_timestamp: None,
        }
    }

    /// Start up, monitor the system and containers
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.wait_for_docker()?;
        self.wait_for_internet_connection()?;

        match self.init_system_with_retry() {
            Ok(_) => {
                tracing::debug!("initialization completed")
            }
            Err(error) => {
                tracing::debug!("initialization failed");
                self.reboot_due_to_error(format!("{error:#}"))?;
            }
        }

        match self.monitor_system_with_retry() {
            Ok(_) => {
                tracing::debug!("monitor completed")
            }
            Err(error) => {
                tracing::debug!("monitor failed");
                self.reboot_due_to_error(format!("{error:#}"))?;
            }
        }

        tracing::info!("manager done");

        Ok(())
    }

    /// Run the initialization steps with retries
    fn init_system_with_retry(&mut self) -> anyhow::Result<()> {
        for index in 0..10 {
            match self.init_system() {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    tracing::error!(?error, "initialization error");
                    let error_message = format!("A problem occurred during start up\n\n{error:#}");

                    let sleep_time = 60 * 2u64.pow(index);
                    tracing::info!(sleep_time, "sleeping");
                    self.countdown_timer(&error_message, sleep_time, CountdownKind::Retry);
                }
            }
        }

        anyhow::bail!("initialization failed repeatedly")
    }

    /// Run the initialization steps that creates and starts up the containers
    fn init_system(&mut self) -> anyhow::Result<()> {
        let _span = tracing::info_span!("initialization");

        self.load_state().context("loading system state failed")?;

        if let Err(error) = self.patch_system() {
            tracing::warn!(?error, "skipping patch system");
            self.display_warning(format!(
                "Patching is skipped because it is unavailable or has an error. It will be retried later.\n\nError: {error:#}"
            ));
            std::thread::sleep(Duration::from_secs(5));
        }

        self.create_containers()
            .context("creating the containers failed")?;

        if let Err(error) = self
            .update_containers()
            .context("updating the containers failed")
        {
            tracing::warn!(?error, "skipping updating containers");
            self.display_warning(format!("{error:#}"));
            std::thread::sleep(Duration::from_secs(5));
        }

        self.start_containers()
            .context("starting the containers failed")?;
        self.wait_for_payload()
            .context("starting the web interface failed")?;
        self.show_ready_message();

        tracing::debug!("completed");

        Ok(())
    }

    /// Run the system and containers monitoring steps with retries
    fn monitor_system_with_retry(&mut self) -> anyhow::Result<()> {
        for index in 0..10 {
            match self.monitor_system() {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    tracing::error!(?error, "run monitor steps error");
                    let error_message = format!("A problem occurred\n\n{error}");

                    let sleep_time = 60 * 2u64.pow(index);
                    tracing::info!(sleep_time, "sleeping");
                    self.countdown_timer(&error_message, sleep_time, CountdownKind::Retry);
                }
            }
        }

        anyhow::bail!("monitoring system failed")
    }

    /// Run the system and container monitoring steps in a loop
    fn monitor_system(&mut self) -> anyhow::Result<()> {
        let _span = tracing::info_span!("monitor system");

        loop {
            self.check_containers()
                .context("checking the containers failed")?;
            std::thread::sleep(Duration::from_secs(10));
        }
    }

    /// Wait for the Docker daemon to start
    fn wait_for_docker(&self) -> anyhow::Result<()> {
        tracing::info!("wait for docker");
        self.display_info("Waiting for Docker to be ready");

        loop {
            let mut command = Command::new("docker");
            command.arg("version");

            let output = crate::logging::log_command_output(&mut command)?;

            if output.status.success() {
                break;
            }

            tracing::debug!("sleep for docker");
            std::thread::sleep(Duration::from_secs(5));
        }

        Ok(())
    }

    /// Show an error message and reboot the OS after a countdown
    fn reboot_due_to_error<S: AsRef<str>>(&mut self, text: S) -> anyhow::Result<()> {
        tracing::info!(text = text.as_ref(), "reboot due to error");

        // If stuck in a reboot loop, don't constantly fetch things over the network
        let seconds = {
            if chrono::Utc::now() - self.state.last_forced_reboot < chrono::Duration::minutes(5) {
                3600
            } else {
                300
            }
        };

        self.countdown_timer(text, seconds, CountdownKind::Reboot);

        self.state.last_forced_reboot = chrono::Utc::now();
        match self.save_state() {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(?error, "save state");
            }
        }

        tracing::info!("reboot now");
        Command::new("reboot").spawn()?;

        Ok(())
    }

    /// Block and show a countdown timer indicating a retry
    fn countdown_timer<S: AsRef<str>>(&self, text: S, seconds: u64, kind: CountdownKind) {
        let when = Instant::now() + Duration::from_secs(seconds);

        loop {
            let remaining = when.saturating_duration_since(Instant::now());

            if remaining.is_zero() {
                break;
            }

            let message = match kind {
                CountdownKind::Retry => format!(
                    "{}\n\nRetrying in {} seconds.",
                    text.as_ref(),
                    remaining.as_secs()
                ),
                CountdownKind::Reboot => format!(
                    "{}\n\nRestarting the system in {} seconds.",
                    text.as_ref(),
                    remaining.as_secs()
                ),
            };

            self.display_error(message);
            std::thread::sleep(Duration::from_secs(5));
        }
    }

    /// Restart the machine
    fn reboot_gracefully(&self) -> anyhow::Result<()> {
        tracing::info!("reboot gracefully");

        self.display_info("The system is now rebooting as requested");

        let mut command = Command::new("reboot");
        crate::logging::log_command_output(&mut command)?;

        Ok(())
    }

    /// Power off the machine
    fn poweroff_gracefully(&self) -> anyhow::Result<()> {
        tracing::info!("poweroff gracefully");

        self.display_info("The system is now powering off as requested");

        let mut command = Command::new("poweroff");
        crate::logging::log_command_output(&mut command)?;

        Ok(())
    }

    /// Send an info message to the display service
    fn display_info<S: Into<String>>(&self, text: S) {
        match self.display_ipc.send_info(text) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    /// Send a warning message to the display service
    fn display_warning<S: Into<String>>(&self, text: S) {
        match self.display_ipc.send_warning(text) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    /// Send an error message to the display service
    fn display_error<S: Into<String>>(&self, text: S) {
        match self.display_ipc.send_error(text) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    /// Send a progress bar message to the display service
    fn display_progress<S: Into<String>>(&self, text: S, percent: u8) {
        match self.display_ipc.send_progress(text, percent) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    /// Send a command output text message to the display service
    fn display_command_output<S: Into<String>>(&self, text: S) {
        match self.display_ipc.send_command_output(text) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    /// Send a finished initialization message to the display service
    fn display_ready<S: Into<String>>(&self, text: S) {
        match self.display_ipc.send_ready(text) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    /// Load application state from disk
    fn load_state(&mut self) -> anyhow::Result<()> {
        self.display_info("Loading appliance manager state");

        if self.config.state_path.try_exists()? {
            tracing::info!("loading state");

            self.state = State::load(&self.config.state_path)?;
        } else {
            tracing::info!("saving new state");

            self.state.save(&self.config.state_path)?;
        }

        Ok(())
    }

    /// Save application state to disk
    fn save_state(&mut self) -> anyhow::Result<()> {
        tracing::info!("saving state");

        self.state.save(&self.config.state_path)?;

        Ok(())
    }

    /// Wait for an internet connection
    fn wait_for_internet_connection(&self) -> anyhow::Result<()> {
        tracing::info!("wait for internet connection");
        self.display_info("Waiting for internet connection");

        loop {
            let mut command = Command::new("ping");
            command.arg("warriorhq.archiveteam.org").arg("-c").arg("1");

            let output = crate::logging::log_command_output(&mut command)?;

            if output.status.success() {
                break;
            }

            tracing::debug!("sleep for internet connection");
            std::thread::sleep(Duration::from_secs(5));
        }

        Ok(())
    }

    /// Download and an execute a file to modify the system
    fn patch_system(&self) -> anyhow::Result<()> {
        if let Some(url) = &self.config.patch_script_url {
            self.download_patch_file(url)?;

            tracing::info!("executing patch file");
            self.display_info("Patching the system");

            let mut command = std::process::Command::new(PATCH_FILE_PATH);
            let status = crate::logging::monitor_command_output(&mut command, |output| {
                let text = String::from_utf8_lossy(output);
                self.display_command_output(crate::logging::get_last_line(&text));
            })?;

            if !status.success() {
                anyhow::bail!("patch program exited with exit status {}", status);
            }

            tracing::info!("patching success");
            self.display_command_output("");
        }

        Ok(())
    }

    /// Download the patch file to disk and make it executable
    fn download_patch_file(&self, url: &str) -> anyhow::Result<()> {
        tracing::info!("downloading patch file");
        self.display_info("Downloading system patch file");

        let mut response = reqwest::blocking::get(url)?;

        tracing::debug!(status_code = %response.status(), "patch file response");

        if !response.status().is_success() {
            anyhow::bail!("download patch file failed: {}", response.status());
        }

        let mut patch_file = std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o755)
            .open(PATCH_FILE_PATH)?;
        response.copy_to(&mut patch_file)?;
        patch_file.sync_all()?;

        Ok(())
    }

    /// Create all the Docker containers (but do not start them)
    fn create_containers(&self) -> anyhow::Result<()> {
        let containers = [
            (
                &self.config.watchtower_name,
                &self.config.watchtower_creator,
            ),
            (
                &self.config.watchtower_run_once_name,
                &self.config.watchtower_run_once_creator,
            ),
            (&self.config.payload_name, &self.config.payload_creator),
        ];

        for (index, (name, creator)) in containers.iter().enumerate() {
            let status = crate::container::get_container_status(name);

            tracing::debug!(name, status, "queried container status");

            if !status.is_empty() {
                continue;
            }

            tracing::info!(name, ?creator, "create container");

            let percent = (index as f32 / containers.len() as f32 * 100.0) as u8;
            self.display_progress(
                format!(
                    "Downloading and creating container {name}\n\nPlease wait. This may take a while."
                ),
                percent,
            );

            let mut command = Command::new(creator);
            let status = crate::logging::monitor_command_output(&mut command, |output| {
                let text = String::from_utf8_lossy(output);
                self.display_command_output(crate::logging::get_last_line(&text));
            })?;

            if !status.success() {
                anyhow::bail!("container creator program exited with status {}", status);
            }

            self.display_command_output("");
        }

        tracing::info!("containers created");

        Ok(())
    }

    /// Start the Watchtower run-once container to force the containers to update
    fn update_containers(&self) -> anyhow::Result<()> {
        tracing::info!("update containers");
        self.display_info("Updating containers\n\nPlease wait. This may take a while.");

        let (output1, output2) =
            crate::container::run_container_foreground(&self.config.watchtower_run_once_name)?;

        if !output1.status.success() || !output2.status.success() {
            anyhow::bail!("update container program exited with failure");
        } else {
            Ok(())
        }
    }

    /// Start the Watchtower and payload containers
    fn start_containers(&self) -> anyhow::Result<()> {
        self.run_pre_start_command()?;

        let containers = [&self.config.watchtower_name, &self.config.payload_name];

        for (index, name) in containers.iter().enumerate() {
            let percent = (index as f32 / containers.len() as f32 * 100.0) as u8;
            let status = crate::container::get_container_status(name);

            if status == "running" {
                tracing::debug!(name, "container already running");
                continue;
            }

            tracing::info!(name, "start container");
            self.display_progress(format!("Starting container {name}"), percent);

            let output = crate::container::start_container(name)?;

            if !output.status.success() {
                anyhow::bail!("pre start command exited with status {}", output.status);
            }
        }

        self.run_post_start_command()?;

        tracing::info!("containers started");

        Ok(())
    }

    /// Run a script before the payload is started
    fn run_pre_start_command(&self) -> anyhow::Result<()> {
        tracing::info!("run pre start command");

        let mut command = Command::new(&self.config.payload_pre_start);
        let output = crate::logging::log_command_output(&mut command)?;

        if !output.status.success() {
            anyhow::bail!("pre start command exited with status {}", output.status);
        } else {
            Ok(())
        }
    }

    /// Run a script when the payload is started
    fn run_post_start_command(&self) -> anyhow::Result<()> {
        tracing::info!("run post start command");

        let mut command = Command::new(&self.config.payload_post_start);
        let output = crate::logging::log_command_output(&mut command)?;

        if !output.status.success() {
            anyhow::bail!("post start command exited with status {}", output.status);
        } else {
            Ok(())
        }
    }

    /// Wait for the payload checker to say the payload is ready to use
    fn wait_for_payload(&self) -> anyhow::Result<()> {
        tracing::info!("wait for payload");
        self.display_info("Waiting for payload to start");

        let mut command = Command::new(&self.config.payload_wait_ready);
        let output = crate::logging::log_command_output(&mut command)?;

        if !output.status.success() {
            anyhow::bail!("wait for payload exited with status {}", output.status);
        } else {
            Ok(())
        }
    }

    /// Tell the user that they can use the web interface
    fn show_ready_message(&self) {
        tracing::info!("payload ready");

        let eth0_ip_address = crate::net::get_eth0_ip_address();
        let message = self
            .config
            .payload_ready_message
            .replace("{ETH0_IP_ADDRESS}", &eth0_ip_address);

        self.display_ready(message);
    }

    /// Run the steps to check if the containers want anything
    fn check_containers(&mut self) -> anyhow::Result<()> {
        tracing::trace!("monitor containers loop iteration");

        if self.check_payload_wants_reboot()? {
            self.reboot_gracefully()?;
        } else if self.check_payload_wants_poweroff()? {
            self.poweroff_gracefully()?;
        } else {
            self.check_payload_status()?;
        }

        Ok(())
    }

    /// Check the payload to see if it's still running properly
    fn check_payload_status(&mut self) -> anyhow::Result<()> {
        // It is tricky to check the reason for the exited state:
        // * it may have crashed
        // * Watchtower may be updating it
        // * the check reboot or check poweroff scripts aren't working correctly
        // * the user stopped it
        // * the container ignores errors and does not return a useful exit code

        if !self.payload_crashed
            && self.check_payload_has_exited_error()?
            && self.config.reboot_on_payload_exit_error
        {
            self.payload_crashed = true;
            tracing::warn!("payload container appears crashed");
            self.reboot_due_to_error("The container unexpectedly stopped")?;
        } else if !self.payload_crashed
            && self.check_payload_is_unhealthy()?
            && self.config.reboot_on_payload_unhealthy
        {
            self.payload_crashed = true;
            tracing::warn!("payload container appears unhealthy");
            self.reboot_due_to_error("The container stopped responding")?;
        }

        Ok(())
    }

    /// Check if the container exited with application error
    fn check_payload_has_exited_error(&mut self) -> anyhow::Result<bool> {
        let status = crate::container::get_container_status(&self.config.payload_name);
        let exit_code =
            crate::container::get_container_exit_code(&self.config.payload_name).unwrap_or(0);

        tracing::trace!(status, exit_code, "check payload status");

        // https://docs.docker.com/engine/reference/run/#exit-status

        Ok(status == "exited" && (1..=124).contains(&exit_code))
    }

    /// Check if the container is unhealthy
    fn check_payload_is_unhealthy(&mut self) -> anyhow::Result<bool> {
        let status = crate::container::get_container_status(&self.config.payload_name);
        let health =
            crate::container::get_container_health(&self.config.payload_name).unwrap_or_default();

        tracing::trace!(status, health, "check payload health");

        if status == "running" && health == "unhealthy" {
            // The container's health check can be unreliable and may recover after some time
            if let Some(timestamp) = self.unheathy_timestamp {
                Ok(timestamp.elapsed() > MAX_UNHEALTHY_TIME)
            } else {
                tracing::debug!("first seen unhealthy container");
                self.unheathy_timestamp = Some(Instant::now());

                Ok(false)
            }
        } else {
            self.unheathy_timestamp = None;
            Ok(false)
        }
    }

    /// Returns whether the payload is requesting a machine restart
    fn check_payload_wants_reboot(&self) -> anyhow::Result<bool> {
        let mut command = Command::new(&self.config.payload_reboot_check);
        let output = crate::logging::trace_command_output(&mut command)?;

        Ok(output.status.success())
    }

    /// Returns whether the payload is requesting a machine power off
    fn check_payload_wants_poweroff(&self) -> anyhow::Result<bool> {
        let mut command = Command::new(&self.config.payload_poweroff_check);
        let output = crate::logging::trace_command_output(&mut command)?;

        Ok(output.status.success())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountdownKind {
    Retry,
    Reboot,
}
