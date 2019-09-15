use serde_yaml::Mapping;

mod date;

pub trait Extension {
    fn name(&self) -> String;
    fn calculate(&self, params: &Mapping) -> Option<String>;
}

pub fn get_extensions() -> Vec<Box<dyn Extension>> {
    vec![
        Box::new(date::DateExtension::new()),
    ]
}