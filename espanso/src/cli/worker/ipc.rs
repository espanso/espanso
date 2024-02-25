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

use std::path::Path;

use anyhow::Result;
use crossbeam::channel::Sender;
use espanso_engine::event::{external::MatchExecRequestEvent, EventType, ExitMode};
use espanso_ipc::{EventHandlerResponse, IPCServer};
use log::{error, warn};

use crate::ipc::IPCEvent;

pub fn initialize_and_spawn(
  runtime_dir: &Path,
  exit_notify: Sender<ExitMode>,
  event_notify: Sender<EventType>,
) -> Result<()> {
  let server = crate::ipc::create_worker_ipc_server(runtime_dir)?;

  std::thread::Builder::new()
    .name("worker-ipc-handler".to_string())
    .spawn(move || {
      server
        .run(Box::new(move |event| match event {
          IPCEvent::Exit => {
            if let Err(err) = exit_notify.send(ExitMode::Exit) {
              error!(
                "experienced error while sending exit signal from worker ipc handler: {}",
                err
              );
            }

            EventHandlerResponse::NoResponse
          }
          IPCEvent::ExitAllProcesses => {
            if let Err(err) = exit_notify.send(ExitMode::ExitAllProcesses) {
              error!(
                "experienced error while sending exit signal from worker ipc handler: {}",
                err
              );
            }

            EventHandlerResponse::NoResponse
          }
          IPCEvent::DisableRequest => send_event(&event_notify, EventType::DisableRequest),
          IPCEvent::EnableRequest => send_event(&event_notify, EventType::EnableRequest),
          IPCEvent::ToggleRequest => send_event(&event_notify, EventType::ToggleRequest),
          IPCEvent::OpenSearchBar => send_event(&event_notify, EventType::ShowSearchBar),
          IPCEvent::OpenConfigFolder => send_event(&event_notify, EventType::ShowConfigFolder),
          IPCEvent::RequestMatchExpansion(payload) => send_event(
            &event_notify,
            EventType::MatchExecRequest(MatchExecRequestEvent {
              trigger: payload.trigger,
              args: payload.args,
            }),
          ),
          #[allow(unreachable_patterns)]
          unexpected_event => {
            warn!(
              "received unexpected event in worker ipc handler: {:?}",
              unexpected_event
            );

            EventHandlerResponse::NoResponse
          }
        }))
        .expect("unable to spawn IPC server");
    })?;

  Ok(())
}

fn send_event(
  event_notify: &Sender<EventType>,
  event: EventType,
) -> EventHandlerResponse<IPCEvent> {
  if let Err(err) = event_notify.send(event) {
    error!(
      "experienced error while sending event signal from worker ipc handler: {}",
      err
    );
  }

  EventHandlerResponse::NoResponse
}
