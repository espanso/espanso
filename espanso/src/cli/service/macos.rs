/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::{bail, Result};
use log::{info, warn};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs::create_dir_all, process::ExitStatus};
use thiserror::Error;

use crate::cli::util::prevent_running_as_root_on_macos;
use crate::error_eprintln;

#[cfg(target_os = "macos")]
const SERVICE_PLIST_CONTENT: &str = include_str!("../../res/macos/com.federicoterzi.espanso.plist");
#[cfg(target_os = "macos")]
const SERVICE_LABEL: &str = "com.federicoterzi.espanso";

pub fn register() -> Result<()> {
    prevent_running_as_root_on_macos();

    if crate::cli::util::is_subject_to_app_translocation_on_macos() {
        error_eprintln!("Unable to register Espanso as service, please move the Espanso.app bundle inside the /Applications directory to proceed.");
        error_eprintln!(
            "For more information, please see: https://github.com/espanso/espanso/issues/844"
        );
        bail!("macOS activated app-translocation on Espanso");
    }

    let plist_file = plist_path();

    // Make sure agents directory exists
    if let Some(agents_dir) = plist_file.parent() {
        create_dir_all(agents_dir)?;
    }

    if !plist_file.exists() {
        info!(
            "creating LaunchAgents entry: {}",
            plist_file.to_str().unwrap_or_default()
        );

        let espanso_path = std::env::current_exe()?;
        info!(
            "entry will point to: {}",
            espanso_path.to_str().unwrap_or_default()
        );

        let plist_content = String::from(SERVICE_PLIST_CONTENT).replace(
            "{{{espanso_path}}}",
            espanso_path.to_str().unwrap_or_default(),
        );

        // Copy the user PATH variable and inject it in the Plist file so that
        // it gets loaded by Launchd.
        // To see why this is necessary: https://github.com/espanso/espanso/issues/233
        let user_path = std::env::var("PATH").unwrap_or_else(|_| String::new());
        let plist_content = plist_content.replace("{{{PATH}}}", &user_path);

        std::fs::write(plist_file.clone(), plist_content).expect("Unable to write plist file");
    }

    ensure_loaded()
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("launchctl load failed")]
    LaunchCtlLoadFailed,
}

// Avoid `launchctl unload -w` here: when `register` runs from the wizard, it is a
// child of the launchd job it would unload, so it gets killed before the matching
// `load -w`, leaving the job disabled. See https://github.com/espanso/espanso/issues/2844
fn ensure_loaded() -> Result<()> {
    info!("enabling espanso launchctl entry");

    match Command::new("launchctl")
        .args(["enable", &service_target()])
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => warn!("enable command exited with {status}"),
        Err(err) => warn!("enable command failed: {err}"),
    }

    if is_loaded() {
        return Ok(());
    }

    info!("loading espanso launchctl entry");

    let loaded = Command::new("launchctl")
        .args([
            "bootstrap",
            &domain_target(),
            plist_path().to_str().unwrap_or_default(),
        ])
        .status()
        .is_ok_and(|status| status.success());

    if loaded {
        Ok(())
    } else {
        Err(RegisterError::LaunchCtlLoadFailed.into())
    }
}

fn is_loaded() -> bool {
    Command::new("launchctl")
        .args(["print", &service_target()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn domain_target() -> String {
    // SAFETY: getuid has no preconditions and always succeeds
    let uid = unsafe { libc::getuid() };
    format!("gui/{uid}")
}

fn service_target() -> String {
    format!("{}/{SERVICE_LABEL}", domain_target())
}

fn plist_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("could not get user home directory");
    home_dir
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{SERVICE_LABEL}.plist"))
}

pub fn unregister() -> Result<()> {
    prevent_running_as_root_on_macos();

    let plist_file = plist_path();
    if plist_file.exists() {
        let _res = Command::new("launchctl")
            .args(["bootout", &service_target()])
            .output();

        std::fs::remove_file(&plist_file)?;

        Ok(())
    } else {
        Err(UnregisterError::PlistNotFound.into())
    }
}

#[derive(Error, Debug)]
pub enum UnregisterError {
    #[error("plist entry not found")]
    PlistNotFound,
}

pub fn is_registered() -> bool {
    plist_path().is_file()
}

pub fn start_service() -> Result<()> {
    if !is_registered() {
        eprintln!("Unable to start espanso as a service as it's not been registered.");
        eprintln!("You can either register it first with `espanso service register` or");
        eprintln!("you can run it in unmanaged mode with `espanso service start --unmanaged`");
        eprintln!();
        eprintln!("NOTE: unmanaged mode means espanso does not rely on the system service manager");
        eprintln!("      to run, but as a result, you are in charge of starting/stopping espanso");
        eprintln!("      when needed.");
        return Err(StartError::NotRegistered.into());
    }

    // The job might have been disabled or unloaded in the meantime, in which
    // case `launchctl start` would fail with exit code 3.
    ensure_loaded()?;

    let res = Command::new("launchctl")
        .args(["kickstart", &service_target()])
        .status();

    if let Ok(status) = res {
        if status.success() {
            Ok(())
        } else {
            Err(StartError::LaunchCtlNonZeroExit(status).into())
        }
    } else {
        Err(StartError::LaunchCtlFailure.into())
    }
}

#[derive(Error, Debug)]
pub enum StartError {
    #[error("not registered as a service")]
    NotRegistered,

    #[error("launchctl failed to run")]
    LaunchCtlFailure,

    #[error("launchctl exited with non-zero code `{0}`")]
    LaunchCtlNonZeroExit(ExitStatus),
}
