use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct Token(pub String, pub String);

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Unable to parse token from string! Improperly segmented."
            )); // Replace with a more appropriate error
        }
        Ok(Token(
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
        ))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

