//! Docker container manager and system maintenance
use std::{
    os::unix::prelude::OpenOptionsExt,
    process::Command,
    time::{Duration, Instant},
};

use anyhow::Context;

use crate::{config::AppConfig, ipc::DisplayIPC, state::State};

const PATCH_FILE_PATH: &str = "/tmp/warrior4-appliance-patch";

pub struct Manager {
    config: AppConfig,
    state: State,
    display_ipc: DisplayIPC,
}

impl Manager {
    pub fn new(config: AppConfig) -> Self {
        let state = State::new();
        let display_ipc = DisplayIPC::new(config.display_ipc_address);
        Self {
            config,
            state,
            display_ipc,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.wait_for_docker()?;

        match self.run_init_steps_loop() {
            Ok(_) => {
                tracing::debug!("run init steps completed")
            }
            Err(error) => {
                tracing::debug!("run init steps failed");
                self.reboot_due_to_error(format!("{:#}", error))?;
            }
        }

        match self.run_monitor_steps_loop() {
            Ok(_) => {
                tracing::debug!("run monitor steps completed")
            }
            Err(error) => {
                tracing::debug!("run monitor steps failed");
                self.reboot_due_to_error(format!("{:#}", error))?;
            }
        }

        tracing::info!("manager done");

        Ok(())
    }

    fn run_init_steps_loop(&mut self) -> anyhow::Result<()> {
        for index in 0..10 {
            match self.run_init_steps() {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    tracing::error!(?error, "run init steps error");
                    let error_message =
                        format!("A problem occurred during start up\n\n{:#}", error);

                    let sleep_time = 60 * 2u64.pow(index);
                    tracing::info!(sleep_time, "sleeping");
                    self.countdown_timer(&error_message, sleep_time);
                }
            }
        }

        anyhow::bail!("running init steps failed")
    }

    fn run_init_steps(&mut self) -> anyhow::Result<()> {
        let _span = tracing::info_span!("run init steps");

        self.load_state().context("loading system state failed")?;

        if let Err(error) = self.patch_system().context("patching the system failed") {
            tracing::warn!(?error, "skipping patch system");
            self.update_progress(format!("{:#}", error), 0);
            std::thread::sleep(Duration::from_secs(5));
        }

        self.create_containers()
            .context("creating the containers failed")?;

        if let Err(error) = self
            .update_containers()
            .context("updating the containers failed")
        {
            tracing::warn!(?error, "skipping updating containers");
            self.update_progress(format!("{:#}", error), 0);
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

    fn run_monitor_steps_loop(&mut self) -> anyhow::Result<()> {
        for index in 0..10 {
            match self.run_monitor_steps() {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    tracing::error!(?error, "run monitor steps error");
                    let error_message = format!("A problem occurred\n\n{}", error);

                    let sleep_time = 60 * 2u64.pow(index);
                    tracing::info!(sleep_time, "sleeping");
                    self.countdown_timer(&error_message, sleep_time);
                }
            }
        }

        anyhow::bail!("running monitor steps failed")
    }

    fn run_monitor_steps(&mut self) -> anyhow::Result<()> {
        let _span = tracing::info_span!("run monitor steps");

        self.monitor_containers()
            .context("monitoring the containers failed")?;

        Ok(())
    }

    fn wait_for_docker(&self) -> anyhow::Result<()> {
        tracing::info!("wait for docker");
        self.update_progress("Waiting for Docker to be ready", 0);

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

    fn reboot_due_to_error<S: AsRef<str>>(&self, text: S) -> anyhow::Result<()> {
        tracing::info!("reboot due to error");

        self.countdown_timer(text, 300);

        tracing::info!("reboot now");
        Command::new("reboot").spawn()?;

        Ok(())
    }

    fn countdown_timer<S: AsRef<str>>(&self, text: S, seconds: u64) {
        let when = Instant::now() + Duration::from_secs(seconds);

        loop {
            let remaining = when.saturating_duration_since(Instant::now());

            if remaining.is_zero() {
                break;
            }

            let message = format!(
                "{}\n\nRetrying in {} seconds.",
                text.as_ref(),
                remaining.as_secs()
            );

            self.update_progress(message, 0);
            std::thread::sleep(Duration::from_secs(5));
        }
    }

    fn reboot_gracefully(&self) -> anyhow::Result<()> {
        tracing::info!("reboot gracefully");

        self.update_progress("The system is now rebooting as requested", 0);

        let mut command = Command::new("reboot");
        crate::logging::log_command_output(&mut command)?;

        Ok(())
    }

    fn poweroff_gracefully(&self) -> anyhow::Result<()> {
        tracing::info!("poweroff gracefully");

        self.update_progress("The system is now powering off as requested", 0);

        let mut command = Command::new("poweroff");
        crate::logging::log_command_output(&mut command)?;

        Ok(())
    }

    fn update_progress<S: Into<String>>(&self, text: S, percent: u8) {
        match self.display_ipc.send_progress(text, percent) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    fn update_ready<S: Into<String>>(&self, text: S) {
        match self.display_ipc.send_ready(text) {
            Ok(_) => {}
            Err(error) => tracing::error!(?error, "display ipc error"),
        }
    }

    fn load_state(&mut self) -> anyhow::Result<()> {
        self.update_progress("Loading appliance manager state", 0);

        if self.config.state_path.try_exists()? {
            tracing::info!("loading state");

            self.state = State::load(&self.config.state_path)?;
        } else {
            tracing::info!("saving new state");

            self.state.save(&self.config.state_path)?;
        }

        Ok(())
    }

    fn patch_system(&self) -> anyhow::Result<()> {
        if let Some(url) = &self.config.patch_script_url {
            self.download_patch_file(url)?;

            tracing::info!("executing patch file");
            self.update_progress("Patching the system", 0);

            let mut command = std::process::Command::new(PATCH_FILE_PATH);
            let output = crate::logging::log_command_output(&mut command)?;

            if !output.status.success() {
                anyhow::bail!("patch program exited with exit status {}", output.status);
            }

            tracing::info!("patching success");
        }

        Ok(())
    }

    fn download_patch_file(&self, url: &str) -> anyhow::Result<()> {
        tracing::info!("downloading patch file");
        self.update_progress("Downloading system patch file", 0);

        let mut response = reqwest::blocking::get(url)?;

        tracing::debug!(status_code = %response.status(), "patch file response");

        if !response.status().is_success() {
            anyhow::bail!("download patch file failed");
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

    fn create_containers(&self) -> anyhow::Result<()> {
        let containers = vec![
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
            self.update_progress(format!("Creating container {}", name), percent);

            let mut command = Command::new(creator);
            let output = crate::logging::log_command_output(&mut command)?;

            if !output.status.success() {
                anyhow::bail!(
                    "container creator program exited with status {}",
                    output.status
                );
            }
        }

        tracing::info!("containers created");

        Ok(())
    }

    fn update_containers(&self) -> anyhow::Result<()> {
        tracing::info!("update containers");
        self.update_progress("Updating containers", 0);

        let (output1, output2) =
            crate::container::run_container_foreground(&self.config.watchtower_run_once_name)?;

        if !output1.status.success() || !output2.status.success() {
            anyhow::bail!("update container program exited with failure");
        } else {
            Ok(())
        }
    }

    fn start_containers(&self) -> anyhow::Result<()> {
        self.run_pre_start_command()?;

        let containers = vec![&self.config.watchtower_name, &self.config.payload_name];

        for (index, name) in containers.iter().enumerate() {
            let percent = (index as f32 / containers.len() as f32 * 100.0) as u8;

            tracing::info!(name, "start container");
            self.update_progress(format!("Starting container {}", name), percent);

            let output = crate::container::start_container(name)?;

            if !output.status.success() {
                anyhow::bail!("pre start command exited with status {}", output.status);
            }
        }

        self.run_post_start_command()?;

        tracing::info!("containers started");

        Ok(())
    }

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

    fn wait_for_payload(&self) -> anyhow::Result<()> {
        tracing::info!("wait for payload");
        self.update_progress("Waiting for payload to start", 0);

        let mut command = Command::new(&self.config.payload_pre_start);
        let output = crate::logging::log_command_output(&mut command)?;

        if !output.status.success() {
            anyhow::bail!("wait for payload exited with status {}", output.status);
        } else {
            Ok(())
        }
    }

    fn show_ready_message(&self) {
        tracing::info!("payload ready");
        self.update_ready(&self.config.payload_ready_message);
    }

    fn monitor_containers(&self) -> anyhow::Result<()> {
        loop {
            self.monitor_container_loop_iteration()?;
            std::thread::sleep(Duration::from_secs(10));
        }
    }

    fn monitor_container_loop_iteration(&self) -> anyhow::Result<()> {
        tracing::trace!("monitor containers loop iteration");

        if self.payload_wants_reboot()? {
            self.reboot_gracefully()?;
        } else if self.payload_wants_poweroff()? {
            self.poweroff_gracefully()?;
        }

        // TODO: Check if the warrior container exited
        // It is tricky to check the reason for the exited state:
        // * it may have crashed
        // * Watchtower may be updating it
        // * the check reboot or check poweroff scripts aren't working correctly

        Ok(())
    }

    fn payload_wants_reboot(&self) -> anyhow::Result<bool> {
        let mut command = Command::new(&self.config.payload_reboot_check);
        let output = crate::logging::trace_command_output(&mut command)?;

        Ok(output.status.success())
    }

    fn payload_wants_poweroff(&self) -> anyhow::Result<bool> {
        let mut command = Command::new(&self.config.payload_poweroff_check);
        let output = crate::logging::trace_command_output(&mut command)?;

        Ok(output.status.success())
    }
}
