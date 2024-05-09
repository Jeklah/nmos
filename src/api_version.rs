use std::fmt;

// Define the API version structure
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}

// Implement the Display trait for API version
impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

// Implement methods for parsing and making API versions
impl ApiVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        ApiVersion { major, minor }
    }

    pub fn parse(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 2 {
            return None;
        }
        if let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse()) {
            Some(ApiVersion::new(major, minor))
        } else {
            None
        }
    }
}

// Unit tests for the API version structure
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let version = ApiVersion::new(1, 2);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
    }

    #[test]
    fn test_parse() {
        assert_eq!(ApiVersion::parse("v1.2"), Some(ApiVersion::new(1, 2)));
        assert_eq!(ApiVersion::parse("v1.2.3"), None);
        assert_eq!(ApiVersion::parse("1.2"), None);
        assert_eq!(ApiVersion::parse("v1"), None);
    }
}
