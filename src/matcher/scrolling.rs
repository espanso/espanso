use crate::matcher::{Match, MatchReceiver};

pub struct ScrollingMatcher<'a>{
    receiver: &'a MatchReceiver
}

impl <'a> super::Matcher for ScrollingMatcher<'a> {
    fn handle_char(&self, c: char) {
        println!("Scroll {}",c);

        if c == 'a' {
            self.receiver.on_match(Match{trigger: "a".to_owned(), result: "ciao".to_owned()});
        }
    }
}

impl <'a> ScrollingMatcher<'a> {
    pub fn new(receiver: &'a MatchReceiver) -> ScrollingMatcher<'a> {
        ScrollingMatcher{ receiver }
    }
}