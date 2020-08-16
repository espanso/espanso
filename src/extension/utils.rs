use crate::extension::ExtensionResult;
use std::collections::HashMap;

pub fn convert_to_env_variables(
    original_vars: &HashMap<String, ExtensionResult>,
) -> HashMap<String, String> {
    let mut output = HashMap::new();

    for (key, result) in original_vars.iter() {
        match result {
            ExtensionResult::Single(value) => {
                let name = format!("ESPANSO_{}", key.to_uppercase());
                output.insert(name, value.clone());
            }
            ExtensionResult::Multiple(values) => {
                for (sub_key, sub_value) in values.iter() {
                    let name = format!("ESPANSO_{}_{}", key.to_uppercase(), sub_key.to_uppercase());
                    output.insert(name, sub_value.clone());
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::Extension;

    #[test]
    fn test_convert_to_env_variables() {
        let mut vars: HashMap<String, ExtensionResult> = HashMap::new();
        let mut subvars = HashMap::new();
        subvars.insert("name".to_owned(), "John".to_owned());
        subvars.insert("lastname".to_owned(), "Snow".to_owned());
        vars.insert("form1".to_owned(), ExtensionResult::Multiple(subvars));
        vars.insert(
            "var1".to_owned(),
            ExtensionResult::Single("test".to_owned()),
        );

        let output = convert_to_env_variables(&vars);
        assert_eq!(output.get("ESPANSO_FORM1_NAME").unwrap(), "John");
        assert_eq!(output.get("ESPANSO_FORM1_LASTNAME").unwrap(), "Snow");
        assert_eq!(output.get("ESPANSO_VAR1").unwrap(), "test");
    }
}
