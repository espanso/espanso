use serde::{Serialize, Deserialize, Deserializer};
use crate::event::{KeyEvent, KeyModifier};
use crate::event::KeyEventReceiver;
use serde_yaml::Mapping;
use regex::Regex;

pub(crate) mod scrolling;

#[derive(Debug, Serialize, Clone)]
pub struct Match {
    pub trigger: String,
    pub replace: String,
    pub vars: Vec<MatchVariable>,

    #[serde(skip_serializing)]
    pub _has_vars: bool,
}

impl <'de> serde::Deserialize<'de> for Match {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {

        let auto_match = AutoMatch::deserialize(deserializer)?;
        Ok(Match::from(&auto_match))
    }
}

impl<'a> From<&'a AutoMatch> for Match{
    fn from(other: &'a AutoMatch) -> Self {
        lazy_static! {
            static ref VarRegex: Regex = Regex::new("\\{\\{\\s*(\\w+)\\s*\\}\\}").unwrap();
        }

        // Check if the match contains variables
        let has_vars = VarRegex.is_match(&other.replace);

        Self {
            trigger: other.trigger.clone(),
            replace: other.replace.clone(),
            vars: other.vars.clone(),
            _has_vars: has_vars,
        }
    }
}

/// Used to deserialize the Match struct before applying some custom elaboration.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AutoMatch {
    pub trigger: String,
    pub replace: String,

    #[serde(default = "default_vars")]
    pub vars: Vec<MatchVariable>,
}

fn default_vars() -> Vec<MatchVariable> {Vec::new()}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchVariable {
    pub name: String,

    #[serde(rename = "type")]
    pub var_type: String,

    pub params: Mapping,
}

pub trait MatchReceiver {
    fn on_match(&self, m: &Match);
    fn on_enable_update(&self, status: bool);
}

pub trait Matcher : KeyEventReceiver {
    fn handle_char(&self, c: char);
    fn handle_modifier(&self, m: KeyModifier);
}

impl <M: Matcher> KeyEventReceiver for M {
    fn on_key_event(&self, e: KeyEvent) {
        match e {
            KeyEvent::Char(c) => {
                self.handle_char(c);
            },
            KeyEvent::Modifier(m) => {
                self.handle_modifier(m);
            },
        }
    }
}