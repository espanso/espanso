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

pub mod effect;
pub mod external;
pub mod input;
pub mod internal;
pub mod ui;

pub type SourceId = u32;

#[derive(Debug, Clone)]
pub struct Event {
  // The source id is a unique, monothonically increasing number
  // that is given to each event by the source and is propagated
  // to all consequential events.
  // For example, if a keyboard event with source_id = 5 generates
  // a detected match event, this event will have source_id = 5
  pub source_id: SourceId,
  pub etype: EventType,
}

impl Event {
  pub fn caused_by(cause_id: SourceId, event_type: EventType) -> Event {
    Event {
      source_id: cause_id,
      etype: event_type,
    }
  }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum EventType {
  NOOP,
  ExitRequested(ExitMode),
  Exit(ExitMode),
  Heartbeat,

  // Inputs
  Keyboard(input::KeyboardEvent),
  Mouse(input::MouseEvent),
  HotKey(input::HotKeyEvent),
  TrayIconClicked,
  ContextMenuClicked(input::ContextMenuClickedEvent),

  // External requests
  MatchExecRequest(external::MatchExecRequestEvent),

  // Internal
  MatchesDetected(internal::MatchesDetectedEvent),
  MatchSelected(internal::MatchSelectedEvent),
  CauseCompensatedMatch(internal::CauseCompensatedMatchEvent),

  RenderingRequested(internal::RenderingRequestedEvent),
  ImageRequested(internal::ImageRequestedEvent),
  Rendered(internal::RenderedEvent),
  ImageResolved(internal::ImageResolvedEvent),
  MatchInjected,
  DiscardPrevious(internal::DiscardPreviousEvent),
  DiscardBetween(internal::DiscardBetweenEvent),
  Undo(internal::UndoEvent),
  RenderingError,

  Disabled,
  Enabled,
  DisableRequest,
  EnableRequest,
  ToggleRequest,
  SecureInputEnabled(internal::SecureInputEnabledEvent),
  SecureInputDisabled,

  // Effects
  TriggerCompensation(effect::TriggerCompensationEvent),
  CursorHintCompensation(effect::CursorHintCompensationEvent),

  KeySequenceInject(effect::KeySequenceInjectRequest),
  TextInject(effect::TextInjectRequest),
  MarkdownInject(effect::MarkdownInjectRequest),
  HtmlInject(effect::HtmlInjectRequest),
  ImageInject(effect::ImageInjectRequest),

  // UI
  ShowContextMenu(ui::ShowContextMenuEvent),
  IconStatusChange(ui::IconStatusChangeEvent),
  DisplaySecureInputTroubleshoot,
  ShowConfigFolder,
  ShowSearchBar,
  ShowText(ui::ShowTextEvent),
  ShowLogs,

  // Other
  LaunchSecureInputAutoFix,
}

#[derive(Debug, Clone)]
pub enum ExitMode {
  Exit,
  ExitAllProcesses,
  RestartWorker,
}
