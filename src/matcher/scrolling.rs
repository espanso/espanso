use crate::matcher::{Match, MatchReceiver};
use std::cell::RefCell;
use crate::keyboard::KeyModifier;
use crate::config::Configs;
use crate::keyboard::KeyModifier::BACKSPACE;
use std::time::SystemTime;

pub struct ScrollingMatcher<'a, R> where R: MatchReceiver{
    configs: Configs,
    receiver: R,
    current_set: RefCell<Vec<MatchEntry<'a>>>,
    toggle_press_time: RefCell<SystemTime>
}

struct MatchEntry<'a> {
    start: usize,
    _match: &'a Match
}

impl <'a, R> super::Matcher<'a> for ScrollingMatcher<'a, R> where R: MatchReceiver+Send{
    fn handle_char(&'a self, c: char) {
        let mut current_set = self.current_set.borrow_mut();

        let new_matches: Vec<MatchEntry> = self.configs.matches.iter()
            .filter(|&x| x.trigger.chars().nth(0).unwrap() == c)
            .map(|x | MatchEntry{start: 1, _match: &x})
            .collect();
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let old_matches: Vec<MatchEntry> = (*current_set).iter()
            .filter(|&x| {
                x._match.trigger[x.start..].chars().nth(0).unwrap() == c
            })
            .map(|x | MatchEntry{start: x.start+1, _match: &x._match})
            .collect();

        (*current_set) = old_matches;
        (*current_set).extend(new_matches);

        let mut found_match = None;

        for entry in (*current_set).iter() {
            if entry.start == entry._match.trigger.len() {
                found_match = Some(entry._match);
                break;
            }
        }

        if let Some(_match) = found_match {
            (*current_set).clear();
            self.receiver.on_match(_match);
        }
    }

    fn handle_modifier(&'a self, m: KeyModifier) {
        if m == self.configs.toggle_key {
            let mut toggle_press_time = self.toggle_press_time.borrow_mut();
            if let Ok(elapsed) = toggle_press_time.elapsed() {
                if elapsed.as_millis() < self.configs.toggle_interval as u128 {
                    println!("Disable! {}", elapsed.as_millis());
                }
            }

            (*toggle_press_time) = SystemTime::now();
        }
    }
}

impl <'a, R> ScrollingMatcher<'a, R> where R: MatchReceiver {
    pub fn new(configs: Configs, receiver: R) -> ScrollingMatcher<'a, R> {
        let current_set = RefCell::new(Vec::new());
        let toggle_press_time = RefCell::new(SystemTime::now());
        ScrollingMatcher{ configs, receiver, current_set, toggle_press_time }
    }
}