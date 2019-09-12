use crate::matcher::{Match, MatchReceiver};
use std::cell::RefCell;
use crate::event::KeyModifier;
use crate::config::ConfigManager;
use crate::event::KeyModifier::BACKSPACE;
use std::time::SystemTime;
use std::collections::VecDeque;

pub struct ScrollingMatcher<'a, R: MatchReceiver, M: ConfigManager<'a>> {
    config_manager: &'a M,
    receiver: &'a R,
    current_set_queue: RefCell<VecDeque<Vec<MatchEntry<'a>>>>,
    toggle_press_time: RefCell<SystemTime>,
    is_enabled: RefCell<bool>,
}

struct MatchEntry<'a> {
    start: usize,
    _match: &'a Match
}

impl <'a, R: MatchReceiver, M: ConfigManager<'a>> super::Matcher for ScrollingMatcher<'a, R, M> where{
    fn handle_char(&self, c: char) {
        // if not enabled, avoid any processing
        if !*(self.is_enabled.borrow()) {
            return;
        }

        let mut current_set_queue = self.current_set_queue.borrow_mut();

        let new_matches: Vec<MatchEntry> = self.config_manager.matches().iter()
            .filter(|&x| x.trigger.chars().nth(0).unwrap() == c)
            .map(|x | MatchEntry{start: 1, _match: &x})
            .collect();
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let combined_matches: Vec<MatchEntry> = match current_set_queue.back() {
            Some(last_matches) => {
                let mut updated: Vec<MatchEntry> = last_matches.iter()
                    .filter(|&x| {
                        x._match.trigger[x.start..].chars().nth(0).unwrap() == c
                    })
                    .map(|x | MatchEntry{start: x.start+1, _match: &x._match})
                    .collect();

                updated.extend(new_matches);
                updated
            },
            None => {new_matches},
        };

        let mut found_match = None;

        for entry in combined_matches.iter() {
            if entry.start == entry._match.trigger.len() {
                found_match = Some(entry._match);
                break;
            }
        }

        current_set_queue.push_back(combined_matches);

        if current_set_queue.len() as i32 > (self.config_manager.default_config().backspace_limit + 1) {
            current_set_queue.pop_front();
        }

        if let Some(_match) = found_match {
            if let Some(last) = current_set_queue.back_mut() {
                last.clear();
            }
            self.receiver.on_match(_match);
        }
    }

    fn handle_modifier(&self, m: KeyModifier) {
        let config = self.config_manager.default_config();

        if m == config.toggle_key {
            let mut toggle_press_time = self.toggle_press_time.borrow_mut();
            if let Ok(elapsed) = toggle_press_time.elapsed() {
                if elapsed.as_millis() < config.toggle_interval as u128 {
                    let mut is_enabled = self.is_enabled.borrow_mut();
                    *is_enabled = !(*is_enabled);

                    if !*is_enabled {
                        self.current_set_queue.borrow_mut().clear();
                    }

                    self.receiver.on_toggle(*is_enabled);
                }
            }

            (*toggle_press_time) = SystemTime::now();
        }

        // Backspace handling, basically "rewinding history"
        if m == BACKSPACE {
            let mut current_set_queue = self.current_set_queue.borrow_mut();
            current_set_queue.pop_back();
        }
    }
}
impl <'a, R: MatchReceiver, M: ConfigManager<'a>> ScrollingMatcher<'a, R, M> {
    pub fn new(config_manager: &'a M, receiver: &'a R) -> ScrollingMatcher<'a, R, M> {
        let current_set_queue = RefCell::new(VecDeque::new());
        let toggle_press_time = RefCell::new(SystemTime::now());

        ScrollingMatcher{
            config_manager,
            receiver,
            current_set_queue,
            toggle_press_time,
            is_enabled: RefCell::new(true)
        }
    }
}