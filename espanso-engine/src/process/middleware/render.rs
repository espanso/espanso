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
use crate::event::{
    effect::TextInjectRequest,
    internal::{ImageRequestedEvent, RenderedEvent},
    Event, EventType, SourceId,
};
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

impl Middleware for RenderMiddleware<'_> {
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
                Err(err) => return handle_render_error(event.source_id, &err, dispatch),
            }
        }

        // Image paths can contain variables too
        if let EventType::ImageRequested(m_event) = &event.etype {
            match self
                .renderer
                .render(m_event.match_id, m_event.trigger.as_deref(), HashMap::new())
            {
                Ok(image_path) => {
                    return Event::caused_by(
                        event.source_id,
                        EventType::ImageRequested(ImageRequestedEvent {
                            image_path,
                            ..m_event.clone()
                        }),
                    );
                }
                Err(err) => return handle_render_error(event.source_id, &err, dispatch),
            }
        }

        event
    }
}

fn handle_render_error(
    source_id: SourceId,
    err: &anyhow::Error,
    dispatch: &mut dyn FnMut(Event),
) -> Event {
    if matches!(
        err.downcast_ref::<RendererError>(),
        Some(RendererError::Aborted)
    ) {
        return Event::caused_by(source_id, EventType::NOOP);
    }
    error!("error during rendering: {err:?}");

    dispatch(Event::caused_by(
        source_id,
        EventType::TextInject(TextInjectRequest {
            text: "[Espanso]: An error occurred during rendering, please examine the logs for more information.".to_string(),
            ..Default::default()
        }),
    ));

    Event::caused_by(source_id, EventType::RenderingError)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRenderer;

    impl<'a> Renderer<'a> for MockRenderer {
        fn render(
            &'a self,
            match_id: i32,
            _: Option<&str>,
            _: HashMap<String, String>,
        ) -> Result<String> {
            assert_eq!(match_id, 1);
            Ok("/tmp/rendered.png".to_string())
        }
    }

    fn image_event() -> Event {
        Event::caused_by(
            0,
            EventType::ImageRequested(ImageRequestedEvent {
                match_id: 1,
                image_path: "/tmp/{{file}}".to_string(),
                trigger: None,
            }),
        )
    }

    #[test]
    fn image_path_is_rendered() {
        let middleware = RenderMiddleware::new(&MockRenderer);
        let event = middleware.next(image_event(), &mut |_| {});
        match event.etype {
            EventType::ImageRequested(m_event) => {
                assert_eq!(m_event.image_path, "/tmp/rendered.png");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
