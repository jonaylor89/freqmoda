use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Deserializer};

const UPPER_HEX: &str = "0123456789ABCDEF";

trait SafeChars {
    fn should_escape(&self, c: u8) -> bool;
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SafeCharsType {
    #[default]
    Default,
    Custom(HashSet<u8>),
    Noop,
}

impl SafeChars for SafeCharsType {
    fn should_escape(&self, c: u8) -> bool {
        match self {
            SafeCharsType::Default => {
                !(c.is_ascii_alphanumeric()
                    || c == b'/'
                    || c == b'-'
                    || c == b'_'
                    || c == b'.'
                    || c == b'~')
            }
            SafeCharsType::Custom(safe_chars) => {
                !(c.is_ascii_alphanumeric()
                    || c == b'/'
                    || c == b'-'
                    || c == b'_'
                    || c == b'.'
                    || c == b'~'
                    || safe_chars.contains(&c))
            }
            SafeCharsType::Noop => false,
        }
    }
}

impl<'de> Deserialize<'de> for SafeCharsType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.into())
    }
}

impl From<&str> for SafeCharsType {
    fn from(s: &str) -> Self {
        if s == "--" {
            SafeCharsType::Noop
        } else if s.is_empty() {
            SafeCharsType::Default
        } else {
            SafeCharsType::Custom(s.bytes().collect())
        }
    }
}

impl From<String> for SafeCharsType {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

fn escape<F>(s: &str, should_escape: F) -> String
where
    F: Fn(u8) -> bool,
{
    let mut result = String::with_capacity(s.len());
    for &c in s.as_bytes() {
        if should_escape(c) {
            if c == b' ' {
                result.push('+');
            } else {
                result.push('%');
                result.push(UPPER_HEX.as_bytes()[(c >> 4) as usize] as char);
                result.push(UPPER_HEX.as_bytes()[(c & 15) as usize] as char);
            }
        } else {
            result.push(c as char);
        }
    }
    result
}

pub fn normalize(key: &str, safe_chars: &SafeCharsType) -> String {
    let cleaned = key.replace("\r\n", "").replace(
        [
            '\r', '\n', '\u{000B}', '\u{000C}', '\u{0085}', '\u{2028}', '\u{2029}',
        ],
        "",
    );

    let cleaned = cleaned.trim_matches('/');
    let path = Path::new(&cleaned).to_str().unwrap_or(cleaned);

    escape(path, |c| safe_chars.should_escape(c))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_safe_chars() {
        let safe_chars = SafeCharsType::Default;

        // Alphanumeric characters should not be escaped
        assert!(!safe_chars.should_escape(b'a'));
        assert!(!safe_chars.should_escape(b'Z'));
        assert!(!safe_chars.should_escape(b'0'));
        assert!(!safe_chars.should_escape(b'9'));

        // Special allowed characters
        assert!(!safe_chars.should_escape(b'/'));
        assert!(!safe_chars.should_escape(b'-'));
        assert!(!safe_chars.should_escape(b'_'));
        assert!(!safe_chars.should_escape(b'.'));
        assert!(!safe_chars.should_escape(b'~'));

        // Characters that should be escaped
        assert!(safe_chars.should_escape(b' '));
        assert!(safe_chars.should_escape(b'!'));
        assert!(safe_chars.should_escape(b'@'));
        assert!(safe_chars.should_escape(b'#'));
        assert!(safe_chars.should_escape(b'$'));
    }

    #[test]
    fn test_custom_safe_chars() {
        let mut set = HashSet::new();
        set.insert(b'!');
        set.insert(b'*');
        let safe_chars = SafeCharsType::Custom(set);

        // Default allowed characters
        assert!(!safe_chars.should_escape(b'a'));
        assert!(!safe_chars.should_escape(b'/'));
        assert!(!safe_chars.should_escape(b'-'));

        // Custom allowed characters
        assert!(!safe_chars.should_escape(b'!'));
        assert!(!safe_chars.should_escape(b'*'));

        // Characters that should still be escaped
        assert!(safe_chars.should_escape(b' '));
        assert!(safe_chars.should_escape(b'@'));
        assert!(safe_chars.should_escape(b'#'));
    }

    #[test]
    fn test_noop_safe_chars() {
        let safe_chars = SafeCharsType::Noop;

        // No characters should be escaped
        assert!(!safe_chars.should_escape(b'a'));
        assert!(!safe_chars.should_escape(b' '));
        assert!(!safe_chars.should_escape(b'!'));
        assert!(!safe_chars.should_escape(b'@'));
    }

    #[test]
    fn test_safe_chars_from_str() {
        // Empty string should create Default
        let safe_chars: SafeCharsType = "".into();
        assert_eq!(safe_chars, SafeCharsType::Default);

        // "--" should create Noop
        let safe_chars: SafeCharsType = "--".into();
        assert_eq!(safe_chars, SafeCharsType::Noop);

        // Any other string should create Custom with those characters
        let safe_chars: SafeCharsType = "!*".into();
        let mut expected = HashSet::new();
        expected.insert(b'!');
        expected.insert(b'*');
        assert_eq!(safe_chars, SafeCharsType::Custom(expected));
    }

    #[test]
    fn test_escape_function() {
        // Test with a simple escape function that escapes spaces
        let result = escape("hello world", |c| c == b' ');
        assert_eq!(result, "hello+world");

        // Test with a function that escapes vowels
        let result = escape("hello world", |c| {
            c == b'a' || c == b'e' || c == b'i' || c == b'o' || c == b'u'
        });
        assert_eq!(result, "h%65ll%6F w%6Frld");

        // Test with a function that escapes nothing
        let result = escape("hello world", |_| false);
        assert_eq!(result, "hello world");

        // Test with a function that escapes everything
        let result = escape("ab", |_| true);
        assert_eq!(result, "%61%62");
    }

    #[test]
    fn test_normalize() {
        // Test with default safe chars
        let result = normalize("hello world", &SafeCharsType::Default);
        assert_eq!(result, "hello+world");

        // Test with custom safe chars that include space
        let mut set = HashSet::new();
        set.insert(b' ');
        let result = normalize("hello world", &SafeCharsType::Custom(set));
        assert_eq!(result, "hello world");

        // Test with noop safe chars
        let result = normalize("hello world!@#", &SafeCharsType::Noop);
        assert_eq!(result, "hello world!@#");

        // Test trimming of leading and trailing slashes
        let result = normalize("/hello/world/", &SafeCharsType::Default);
        assert_eq!(result, "hello/world");

        // Test newline handling
        let result = normalize("hello\r\nworld", &SafeCharsType::Default);
        assert_eq!(result, "helloworld");

        let result = normalize("hello\nworld", &SafeCharsType::Default);
        assert_eq!(result, "helloworld");

        let result = normalize("hello\rworld", &SafeCharsType::Default);
        assert_eq!(result, "helloworld");

        // Test with other Unicode line endings
        let result = normalize("hello\u{000B}world", &SafeCharsType::Default);
        assert_eq!(result, "helloworld");

        let result = normalize("hello\u{2028}world", &SafeCharsType::Default);
        assert_eq!(result, "helloworld");
    }
}
