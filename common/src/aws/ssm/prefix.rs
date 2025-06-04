use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct ParameterPrefix(String);

impl ParameterPrefix {
    fn trim_end_matches(prefix: &str) -> &str {
        prefix.trim_end_matches('/')
    }

    pub fn new(prefix: &str) -> Result<ParameterPrefix, ParameterPrefixError> {
        let lowercase = prefix.to_lowercase();

        // Parameters must include a leading forward slash character (/)
        if !lowercase.starts_with('/') {
            return Err(ParameterPrefixError::NoForwardSlash(prefix.into()));
        }

        // Parameter name can't be prefixed with "aws" or "ssm" (case-insensitive)
        if lowercase.contains("/aws") || lowercase.contains("/ssm") {
            return Err(ParameterPrefixError::ReservedKeyword(prefix.into()));
        }

        // Parameter names can't include spaces
        if lowercase.contains(char::is_whitespace) {
            return Err(ParameterPrefixError::Whitespace(prefix.into()));
        }

        // Parameter names can consist of the following symbols and letters only: a-zA-Z0-9_.-
        // In addition, the slash character ( / ) is used to delineate hierarchies in parameter names
        if lowercase
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '.' && c != '-' && c != '/')
        {
            return Err(ParameterPrefixError::IllegalCharacter(prefix.into()));
        }

        Ok(ParameterPrefix(Self::trim_end_matches(prefix).to_owned()))
    }

    /// Get a fully qualified parameter name from this prefix
    pub fn get_parameter(&self, parameter_name: &str) -> String {
        format!("{}/{}", self, parameter_name.trim_start_matches('/'))
    }
}

impl Display for ParameterPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for ParameterPrefix {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl AsRef<str> for ParameterPrefix {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl FromStr for ParameterPrefix {
    type Err = ParameterPrefixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParameterPrefix::new(s)
    }
}

/// Represents a validation error
#[derive(Error, Debug)]
pub enum ParameterPrefixError {
    #[error("{0} contains 'aws' or 'ssm', which are not allowed")]
    ReservedKeyword(String),
    #[error("{0} doesn't start with a /")]
    NoForwardSlash(String),
    #[error("{0} contains one or multiple whitespaces")]
    Whitespace(String),
    #[error("{0} contains illegal characters. Only a-zA-Z0-9_.- and / are allowed")]
    IllegalCharacter(String),
}

#[cfg(test)]
mod tests {

    use crate::aws::ssm::{parameters::ANCHOR_ORG_PK_SSM_PARAMETER, prefix::ParameterPrefixError};

    use super::ParameterPrefix;

    #[test]
    fn when_trailing_slash_then_remove() {
        let prefix = ParameterPrefix::new("/STAGE/stack/app/").unwrap();
        assert_eq!(*prefix, *"/STAGE/stack/app")
    }

    #[test]
    fn when_multiple_trailing_slashes_then_remove_all() {
        let prefix = ParameterPrefix::new("/STAGE/stack/app///").unwrap();
        assert_eq!(*prefix, *"/STAGE/stack/app")
    }

    #[test]
    fn when_no_trailing_slash_then_do_nothing() {
        let prefix = ParameterPrefix::new("/STAGE/stack/app").unwrap();
        assert_eq!(*prefix, *"/STAGE/stack/app")
    }

    #[test]
    fn when_illegal_prefix_then_return_error() {
        let prefix = ParameterPrefix::new("STAGE/stack/app");
        assert!(matches!(
            prefix,
            Err(ParameterPrefixError::NoForwardSlash(_))
        ));
    }

    #[test]
    fn when_reserved_keyword_then_return_error() {
        let prefix = ParameterPrefix::new("/STAGE/stack/aws/app");
        assert!(matches!(
            prefix,
            Err(ParameterPrefixError::ReservedKeyword(_))
        ));
    }

    #[test]
    fn when_contains_whitespace_then_return_error() {
        let prefix = ParameterPrefix::new("/STAGE/stack/ oops/app");
        assert!(matches!(prefix, Err(ParameterPrefixError::Whitespace(_))));
    }

    #[test]
    fn when_contains_invalid_character_then_return_error() {
        let prefix = ParameterPrefix::new("/STAGE/スタック/app");
        assert!(matches!(
            prefix,
            Err(ParameterPrefixError::IllegalCharacter(_))
        ));
    }

    #[test]
    fn ensure_newtype_display_trait_works() {
        let prefix = ParameterPrefix::new("/STAGE/stack/app").unwrap();
        let full_parameter = prefix.get_parameter(ANCHOR_ORG_PK_SSM_PARAMETER);
        assert_eq!(
            &full_parameter,
            "/STAGE/stack/app/keys/organization.pub.json"
        )
    }
}
