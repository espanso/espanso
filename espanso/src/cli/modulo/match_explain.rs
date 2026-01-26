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

use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use espanso_config::{
    config::{AppProperties, ConfigStore},
    matches::store::MatchStore,
};
use espanso_ipc::IPCClient;
use log::warn;

use crate::{
    cli::match_cli::explain::{explain_output, ExplainOptions},
    icon::IconPaths,
    ipc::{create_ipc_client_to_worker, IPCEvent},
    path::Paths,
};

pub fn match_explain_main(
    _matches: &ArgMatches,
    paths: &Paths,
    icon_paths: &IconPaths,
    config_store: Box<dyn ConfigStore>,
    match_store: Box<dyn MatchStore>,
) -> i32 {
    let ipc_client = match create_ipc_client_to_worker(&paths.runtime) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("unable to connect to espanso worker: {err:?}");
            return 1;
        }
    };

    let ipc_client: Arc<Mutex<Box<dyn IPCClient<IPCEvent> + Send>>> =
        Arc::new(Mutex::new(Box::new(ipc_client)));
    let previous_enabled = Arc::new(Mutex::new(None::<bool>));

    let on_focus_gained = {
        let ipc_client = Arc::clone(&ipc_client);
        let previous_enabled = Arc::clone(&previous_enabled);
        move || {
            let Ok(mut client) = ipc_client.lock() else {
                warn!("unable to acquire ipc client lock for focus gained");
                return;
            };

            match client.send_sync(IPCEvent::QueryEnabledState) {
                Ok(IPCEvent::EnabledState(is_enabled)) => {
                    let mut previous = previous_enabled
                        .lock()
                        .expect("unable to lock previous enabled state");
                    *previous = Some(is_enabled);
                    if is_enabled {
                        if let Err(err) = client.send_async(IPCEvent::DisableRequest) {
                            warn!("unable to send disable request: {err:?}");
                        }
                    }
                }
                Ok(other) => {
                    warn!("unexpected response when querying enabled state: {other:?}");
                }
                Err(err) => {
                    warn!("unable to query enabled state: {err:?}");
                }
            }
        }
    };

    let on_focus_lost = {
        let ipc_client = Arc::clone(&ipc_client);
        let previous_enabled = Arc::clone(&previous_enabled);
        move || {
            let Ok(mut client) = ipc_client.lock() else {
                warn!("unable to acquire ipc client lock for focus lost");
                return;
            };

            let previous = {
                let mut lock = previous_enabled
                    .lock()
                    .expect("unable to lock previous enabled state");
                lock.take()
            };

            if previous == Some(true) {
                if let Err(err) = client.send_async(IPCEvent::EnableRequest) {
                    warn!("unable to send enable request: {err:?}");
                }
            }
        }
    };

    let on_check = {
        let config_store = config_store;
        let match_store = match_store;
        move |trigger: &str| -> String {
            let output = explain_output(
                ExplainOptions {
                    trigger,
                    show_all: false,
                    json_output: false,
                    app_properties: AppProperties {
                        title: None,
                        class: None,
                        exec: None,
                    },
                },
                &*config_store,
                &*match_store,
            );

            match output {
                Ok(text) => text,
                Err(err) => format!("Error: {err:?}"),
            }
        }
    };

    let options = espanso_modulo::match_explain_dialog::MatchExplainDialogOptions {
        window_icon_path: icon_paths
            .wizard_icon
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        handlers: espanso_modulo::match_explain_dialog::MatchExplainDialogHandlers {
            on_check: Box::new(on_check),
            on_focus_gained: Box::new(on_focus_gained),
            on_focus_lost: Box::new(on_focus_lost),
        },
    };

    if let Err(err) = espanso_modulo::match_explain_dialog::show(options) {
        eprintln!("unable to show match explain dialog: {err:?}");
        return 1;
    }

    0
}
