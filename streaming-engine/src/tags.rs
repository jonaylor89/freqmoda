use std::collections::HashMap;

const MAX_TAG_VALUE_LENGTH: usize = 256;
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),

    #[error("Invalid tag value: {name}={value}")]
    InvalidTagValue { name: String, value: String },
}

pub fn create_tags(
    custom_tags: HashMap<String, String>,
) -> Result<HashMap<String, String>, TagError> {
    let mut tags = HashMap::with_capacity(10);

    // Add default tags
    tags.insert("processor".into(), "Streaming Engine".into());
    tags.insert("timestamp".into(), chrono::Utc::now().to_rfc3339());
    tags.insert(
        "host".into(),
        gethostname::gethostname().to_string_lossy().into(),
    );
    tags.insert("version".into(), VERSION.into());

    // Add provided custom tags
    for (k, v) in custom_tags {
        // Simple validation
        if !k.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(TagError::InvalidTagName(format!("Invalid tag name: {}", k)));
        }
        if v.len() > MAX_TAG_VALUE_LENGTH {
            return Err(TagError::InvalidTagValue { name: k, value: v });
        }
        tags.insert(k, v);
    }

    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tags_with_valid_tags() {
        let mut custom_tags = HashMap::new();
        custom_tags.insert("artist".to_string(), "Test Artist".to_string());
        custom_tags.insert("album".to_string(), "Test Album".to_string());

        let result = create_tags(custom_tags).unwrap();

        // Check default tags are present
        assert!(result.contains_key("processor"));
        assert!(result.contains_key("timestamp"));
        assert!(result.contains_key("host"));
        assert!(result.contains_key("version"));

        // Check custom tags are present
        assert_eq!(result.get("artist").unwrap(), "Test Artist");
        assert_eq!(result.get("album").unwrap(), "Test Album");
    }

    #[test]
    fn test_create_tags_with_invalid_tag_name() {
        let mut custom_tags = HashMap::new();
        custom_tags.insert("invalid-tag".to_string(), "Value".to_string());

        let result = create_tags(custom_tags);
        assert!(matches!(result, Err(TagError::InvalidTagName(_))));
    }

    #[test]
    fn test_create_tags_with_long_value() {
        let mut custom_tags = HashMap::new();
        custom_tags.insert(
            "long_value".to_string(),
            "a".repeat(MAX_TAG_VALUE_LENGTH + 1),
        );

        let result = create_tags(custom_tags);
        assert!(matches!(result, Err(TagError::InvalidTagValue { .. })));
    }

    #[test]
    fn test_create_tags_empty_custom_tags() {
        let custom_tags = HashMap::new();
        let result = create_tags(custom_tags).unwrap();

        // Should only contain default tags
        assert_eq!(result.len(), 4);
        assert!(result.contains_key("processor"));
        assert!(result.contains_key("timestamp"));
        assert!(result.contains_key("host"));
        assert!(result.contains_key("version"));
    }
}
