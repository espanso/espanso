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

use super::super::Middleware;
use crate::event::{Event, EventType, SourceId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

// A thin interface implemented by the host binary (espanso) to persist stats.
#[derive(Clone, Debug)]
pub struct StatsRecord {
    pub trigger: String,
}

pub trait StatsRecorder: Send + Sync {
    fn record(&self, record: StatsRecord);
}

struct NoOpRecorder;
impl StatsRecorder for NoOpRecorder {
    fn record(&self, _record: StatsRecord) {}
}

static GLOBAL_RECORDER: OnceLock<Arc<dyn StatsRecorder>> = OnceLock::new();

pub fn set_global_recorder(recorder: Box<dyn StatsRecorder>) {
    GLOBAL_RECORDER.get_or_init(|| Arc::from(recorder));
}

fn get_recorder() -> &'static Arc<dyn StatsRecorder> {
    GLOBAL_RECORDER.get_or_init(|| Arc::new(NoOpRecorder))
}

// Minimal middleware to attribute a trigger to an actual injection.
struct PendingEntry {
    trigger: String,
}

pub struct StatsMiddleware {
    pending: RefCell<HashMap<SourceId, PendingEntry>>,
}

impl StatsMiddleware {
    pub fn new() -> Self {
        Self {
            pending: RefCell::new(HashMap::new()),
        }
    }
}

impl Middleware for StatsMiddleware {
    fn name(&self) -> &'static str {
        "stats"
    }

    fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
        match &event.etype {
            EventType::RenderingRequested(r_event) => {
                if let Some(trigger) = &r_event.trigger {
                    let mut pend = self.pending.borrow_mut();
                    if pend.len() > 4096 {
                        // simple capacity cap
                        pend.clear();
                    }
                    pend.insert(
                        event.source_id,
                        PendingEntry {
                            trigger: trigger.clone(),
                        },
                    );
                }
            }
            EventType::ImageRequested(i_event) => {
                if let Some(trigger) = &i_event.trigger {
                    let mut pend = self.pending.borrow_mut();
                    if pend.len() > 4096 {
                        pend.clear();
                    }
                    pend.insert(
                        event.source_id,
                        PendingEntry {
                            trigger: trigger.clone(),
                        },
                    );
                }
            }
            EventType::DiscardPrevious(e) => {
                let mut pend = self.pending.borrow_mut();
                let min_id = e.minimum_source_id;
                pend.retain(|&k, _| k >= min_id);
            }
            EventType::DiscardBetween(e) => {
                let mut pend = self.pending.borrow_mut();
                let start = e.start_id;
                let end = e.end_id;
                pend.retain(|&k, _| k < start || k >= end);
            }
            EventType::MatchInjected => {
                if let Some(entry) = self.pending.borrow_mut().remove(&event.source_id) {
                    let rec = StatsRecord {
                        trigger: entry.trigger,
                    };
                    get_recorder().record(rec);
                }
            }
            _ => {}
        }

        event
    }
}
