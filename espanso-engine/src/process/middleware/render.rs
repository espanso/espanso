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

use std::collections::HashMap;

use log::error;

use super::super::Middleware;
use crate::event::{effect::TextInjectRequest, internal::RenderedEvent, Event, EventType};
use anyhow::Result;
use thiserror::Error;

pub trait Renderer<'a> {
    fn render(
        &'a self,
        match_id: i32,
        trigger: Option<&str>,
        trigger_args: HashMap<String, String>,
    ) -> Result<String>;
}

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("rendering error")]
    RenderingError(#[from] anyhow::Error),

    #[error("match not found")]
    NotFound,

    #[error("aborted")]
    Aborted,
}

pub struct RenderMiddleware<'a> {
    renderer: &'a dyn Renderer<'a>,
}

impl<'a> RenderMiddleware<'a> {
    pub fn new(renderer: &'a dyn Renderer<'a>) -> Self {
        Self { renderer }
    }
}

impl<'a> Middleware for RenderMiddleware<'a> {
    fn name(&self) -> &'static str {
        "render"
    }

    fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
        if let EventType::RenderingRequested(m_event) = event.etype {
            match self.renderer.render(
                m_event.match_id,
                m_event.trigger.as_deref(),
                m_event.trigger_args,
            ) {
                Ok(body) => {
                    let body = if let Some(right_separator) = m_event.right_separator {
                        format!("{body}{right_separator}")
                    } else {
                        body
                    };

                    return Event::caused_by(
                        event.source_id,
                        EventType::Rendered(RenderedEvent {
                            match_id: m_event.match_id,
                            body,
                            format: m_event.format,
                        }),
                    );
                }
                Err(err) => {
                    if let Some(RendererError::Aborted) = err.downcast_ref::<RendererError>() {
                        return Event::caused_by(event.source_id, EventType::NOOP);
                    }
                    error!("error during rendering: {:?}", err);

                    dispatch(Event::caused_by(
                    event.source_id,
                    EventType::TextInject(TextInjectRequest {
                      text: "[Espanso]: An error occurred during rendering, please examine the logs for more information.".to_string(),
                      ..Default::default()
                    }),
                  ));

                    return Event::caused_by(event.source_id, EventType::RenderingError);
                }
            }
        }

        event
    }
}

// TODO: test
