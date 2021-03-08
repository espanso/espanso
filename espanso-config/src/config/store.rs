use super::{Config, ConfigStore};

pub(crate) struct DefaultConfigStore {
  default: Box<dyn Config>,
  customs: Vec<Box<dyn Config>>,
}

impl<'a> ConfigStore<'a> for DefaultConfigStore {
  fn default(&'a self) -> &'a dyn super::Config {
    self.default.as_ref()
  }

  fn active(&'a self, app: &super::AppProperties) -> &'a dyn super::Config {
    // Find a custom config that matches or fallback to the default one
    for custom in self.customs.iter() {
      if custom.is_match(app) {
        return custom.as_ref();
      }
    }
    self.default.as_ref()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct MockConfig {
    label: String,
    is_match: bool,
  }

  impl MockConfig {
    pub fn new(label: &str, is_match: bool) -> Self {
      Self {
        label: label.to_string(),
        is_match,
      }
    }
  }

  impl Config for MockConfig {
    fn label(&self) -> &str {
      &self.label
    }

    fn match_paths(&self) -> &std::collections::HashSet<String> {
      unimplemented!()
    }

    fn is_match(&self, _: &crate::config::AppProperties) -> bool {
      self.is_match
    }
  }

  #[test]
  fn config_store_selects_correctly() {
    let default = MockConfig::new("default", false);
    let custom1 = MockConfig::new("custom1", false);
    let custom2 = MockConfig::new("custom2", true);

    let store = DefaultConfigStore {
      default: Box::new(default),
      customs: vec![Box::new(custom1), Box::new(custom2)],
    };

    assert_eq!(store.default().label(), "default");
    assert_eq!(
      store
        .active(&crate::config::AppProperties {
          title: None,
          class: None,
          exec: None,
        })
        .label(),
      "custom2"
    );
  }

  #[test]
  fn config_store_active_fallback_to_default_if_no_match() {
    let default = MockConfig::new("default", false);
    let custom1 = MockConfig::new("custom1", false);
    let custom2 = MockConfig::new("custom2", false);

    let store = DefaultConfigStore {
      default: Box::new(default),
      customs: vec![Box::new(custom1), Box::new(custom2)],
    };

    assert_eq!(store.default().label(), "default");
    assert_eq!(
      store
        .active(&crate::config::AppProperties {
          title: None,
          class: None,
          exec: None,
        })
        .label(),
      "default"
    );
  }
}
