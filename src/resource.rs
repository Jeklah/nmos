use serde_json::{json, Value};
use std::collections::HashMap;

// Define the namespace for NMOS
mod nmos {
    pub mod details {
        // Define a function to create resource core JSON value
        pub fn make_resource_core(
            id: &str,
            label: &str,
            description: &str,
            tags: HashMap<String, Value>,
        ) -> Value {
            json!({
                "id": id,
                "version": nmos::make_version(),
                "label": label,
                "description": description,
                "tags": tags,
            })
        }

        // Define a function to create resource core JSON value from settings
        pub fn make_resource_core_from_settings(id: &str, settings: &nmos::Settings) -> Value {
            let label = settings.get_label();
            let description = settings.get_description();

            make_resource_core(id, &label, &description, HashMap::new())
        }
    }

    // Define a function to make version JSON value
    fn make_version() -> Value {
        json!({"major": 1, "minor": 2})
    }

    // Define a structure for settings
    pub struct Settings {
        label: String,
        description: String,
    }

    // Implement methods for settings
    impl Settings {
        pub fn new(label: &str, description: &str) -> Self {
            Settings {
                label: label.to_string(),
                description: description.to_string(),
            }
        }

        pub fn get_label(&self) -> String {
            &self.label
        }

        pub fn get_description(&self) -> String {
            &self.description
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_resource_core() {
        let id = "resource_id";
        let label = "resource_label";
        let description = "resource_description";
        let tags = HashMap::new();
        let result = nmos::details::make_resource_core(id, label, description, tags);
        let expected = json!({
            "id": "resource_id",
            "version": {"major": 1, "minor": 2},
            "label": "resource_label",
            "description": "resource_description",
            "tags": {}
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_make_resource_core_from_settings() {
        let id = "resource_id";
        let settings = nmos::Settings::new("label_from_settings", "description_from_settings");
        let result = nmos::details::make_resource_core_from_settings(id, &settings);
        let expected = json!({
            "id": "resource_id",
            "version": {"major": 1, "minor": 2},
            "label": "label_from_settings",
            "description": "description_from_settings",
            "tags": {}
        });
        assert_eq!(result, expected);
    }
}
