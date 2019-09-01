use crate::matcher::{Match, MatchReceiver};

pub struct ScrollingMatcher<'a>{
    matches: &'a Vec<Match>,
    receiver: &'a dyn MatchReceiver,
    current_set: Vec<MatchEntry<'a>>
}

struct MatchEntry<'a> {
    remaining: &'a str,
    _match: &'a Match
}

impl <'a> super::Matcher for ScrollingMatcher<'a> {
    fn handle_char(&mut self, c: char) {
        let mut new_matches: Vec<MatchEntry> = self.matches.iter()
            .filter(|&x| x.trigger.chars().nth(0).unwrap() == c)
            .map(|x | MatchEntry{remaining: &x.trigger[1..], _match: &x})
            .collect();
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let old_matches = self.current_set.iter()
            .filter(|&x| x.remaining.chars().nth(0).unwrap() == c)
            .map(|x | MatchEntry{remaining: &x.remaining[1..], _match: &x._match})
            .collect();

        self.current_set = old_matches;
        self.current_set.append(&mut new_matches);

        let mut found_match = None;

        for entry in self.current_set.iter_mut() {
            if entry.remaining.len() == 0 {
                found_match = Some(entry._match);
                break;
            }
        }

        if let Some(_match) = found_match {
            self.current_set.clear();
            self.receiver.on_match(_match);
        }
    }
}

impl <'a> ScrollingMatcher<'a> {
    pub fn new(matches:&'a Vec<Match>, receiver: &'a dyn MatchReceiver) -> ScrollingMatcher<'a> {
        let current_set = Vec::new();
        ScrollingMatcher{ matches, receiver, current_set }
    }
}