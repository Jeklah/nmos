use std::collections::{HashMap, HashSet};

// Define the API version structure
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
struct ApiVersion {
    major: u32,
    minor: u32,
}

// Define the resource type enumeration
#[derive(Debug, PartialEq, Eq, Hash)]
enum ResourceType {
    Node,
    Device,
    Source,
    Flow,
    Sender,
    Receiver,
    Subscription,
}

// Define the Resource structure
struct Resource {
    version: ApiVersion,
    downgrade_version: ApiVersion,
    // Assume other fields are present here... (?)
}

// Define the ApiDowngradeError enumeration
#[derive(Debug)]
enum ApiDowngradeError {
    NotPermitted(&'static str),
}

// Define the downgrade function
fn downgrade(
    resource: &Resource,
    version: ApiVersion,
    downgrade_version: ApiVersion,
) -> Result<(), ApiDowngradeError> {
    if !is_permitted_downgrade(&resource, version, downgrade_version) {
        return Err(ApiDowngradeError::NotPermitted("Downgrade not permitted"));
    }

    // Downgrade resource data here...
    // Here, we simply return Ok(()) as a placeholder.
    Ok(())
}

// Define the is_permitted_downgrade function
fn is_permitted_downgrade(
    resource: &Resource,
    version: ApiVersion,
    downgrade_version: ApiVersion,
) -> bool {
    if resource.downgrade_version > downgrade_version {
        return false;
    }

    if version.major != downgrade_version.major {
        return false;
    }

    if resource.version.major != version.major {
        return false;
    }

    if resource.version.minor < downgrade_version.minor {
        return false;
    }

    if ResourceType::Subscription == ResourceType::Subscription && version != resource.version {
        return false;
    }

    true
}

// Define the downgrade function for resource data
fn downgrade_data(resource_data: &HashMap<String, String> resource_type: ResourceType, version: ApiVersion, downgrade_version: ApiVersion) -> HashMap<String, String> {
    let mut result = HashMap::new();

    // This is a placeholder for the downgrade logic
    // In a real implementation, this would handle doengrading the resource data based on versions.
    // Here, we simply copy the original data to the result
    for (key, value) in resource_data {
        result.insert(key.clone(), value.clone());
    }
    result
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_permitted_downgrade() {
        let resource = Resource {
            version: ApiVersion { major: 1, minor: 2 },
            downgrade_version: ApiVersion { major: 1, minor: 1 },
        };
        let version = ApiVersion { major: 1, minor: 2 };
        let downgrade_version = ApiVersion { major: 1, minor: 1 };

        assert!(is_permitted_downgrade(&resource, version, downgrade_version));
    }

    #[test]
    fn test_downgrade_data() {
        let mut resource_data = HashMap::new();
        resource_data.insert("key1".to_string(), "value1".to_string());
        resource_data.insert("key2".to_string(), "value2".to_string());
        let resource_type = ResourceType::Node;
        let version = ApiVersion { major: 1, minor: 2 };
        let downgrade_version = ApiVersion { major: 1, minor: 1 };

        let result = downgrade_data(&resource_data, resource_type, version, downgrade_version);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }
}

