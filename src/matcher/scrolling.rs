use crate::matcher::{Match, MatchReceiver};
use std::cell::RefCell;
use crate::keyboard::KeyModifier;

pub struct ScrollingMatcher<'a, R> where R: MatchReceiver{
    matches: Vec<Match>,
    receiver: R,
    current_set: RefCell<Vec<MatchEntry<'a>>>
}

struct MatchEntry<'a> {
    remaining: &'a str,
    _match: &'a Match
}

impl <'a, R> super::Matcher<'a> for ScrollingMatcher<'a, R> where R: MatchReceiver+Send{
    fn handle_char(&'a self, c: char) {
        let mut current_set = self.current_set.borrow_mut();

        let new_matches: Vec<MatchEntry> = self.matches.iter()
            .filter(|&x| x.trigger.chars().nth(0).unwrap() == c)
            .map(|x | MatchEntry{remaining: &x.trigger[1..], _match: &x})
            .collect();
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let old_matches: Vec<MatchEntry> = (*current_set).iter()
            .filter(|&x| x.remaining.chars().nth(0).unwrap() == c)
            .map(|x | MatchEntry{remaining: &x.remaining[1..], _match: &x._match})
            .collect();

        (*current_set) = old_matches;
        (*current_set).extend(new_matches);

        let mut found_match = None;

        for entry in (*current_set).iter() {
            if entry.remaining.len() == 0 {
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

    }
}

impl <'a, R> ScrollingMatcher<'a, R> where R: MatchReceiver {
    pub fn new(matches:Vec<Match>, receiver: R) -> ScrollingMatcher<'a, R> {
        let current_set = RefCell::new(Vec::new());
        ScrollingMatcher{ matches, receiver, current_set }
    }
}