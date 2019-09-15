use serde_yaml::{Mapping, Value};
use chrono::{DateTime, Utc};

pub struct DateExtension {}

impl DateExtension {
    pub fn new() -> DateExtension {
        DateExtension{}
    }
}

impl super::Extension for DateExtension {
    fn name(&self) -> String {
        String::from("date")
    }

    fn calculate(&self, params: &Mapping) -> Option<String> {
        let now: DateTime<Utc> = Utc::now();

        let format = params.get(&Value::from("format"));

        let date = if let Some(format) = format {
            now.format(format.as_str().unwrap()).to_string()
        }else{
            now.to_rfc2822()
        };

        Some(date)
    }
}