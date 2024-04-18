use std::collections::hash_map::Iter;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::{io::ErrorKind, path::Path};

use super::token::Token;
use anyhow::{anyhow, Result};
use uuid::Uuid;

pub struct TokenStore<'a> {
    file_path: &'a Path,
    tokens: Option<HashMap<String, Token>>, // Stores all token objects in memory
    token_lookup: Option<HashSet<String>>,  // Quickly checks if a token string is authorized
}

impl<'a> TokenStore<'a> {
    pub fn new(file_path: &'a Path) -> Result<Self> {
        let mut token_store = TokenStore {
            file_path,
            tokens: None,
            token_lookup: None,
        };
        token_store.reload()?;
        Ok(token_store)
    }

    pub fn reload(&mut self) -> Result<()> {
        let file = match File::open(self.file_path) {
            Ok(file) => file,
            Err(ref error) if error.kind() == ErrorKind::NotFound => {
                self.tokens = Some(HashMap::new());
                self.token_lookup = Some(HashSet::new());
                return Ok(());
            }
            Err(_) => {
                return Err(anyhow!(
                    "Unable to open keystore file at {}",
                    self.file_path.display()
                ))
            }
        };
        let reader = io::BufReader::new(file);

        let mut token_map = HashMap::new();
        for line_result in reader.lines() {
            let line = line_result.map_err(|e| anyhow!("Failed to read line: {}", e))?;
            let token = Token::from_str(&line)
                .map_err(|_| anyhow!("Failed to parse token from line: {}", line))?;
            token_map.insert(token.0.clone(), token);
        }

        self.tokens = Some(token_map);
        self.rebuild_token_lookup()?;
        Ok(())
    }

    fn persist_to_file(&self) -> io::Result<()> {
        let file = File::create(self.file_path)?;
        let mut writer = io::BufWriter::new(file);
        if let Some(tokens) = self.tokens.as_ref() {
            for token in tokens.values() {
                writeln!(writer, "{}", token)?;
            }
        }
        Ok(())
    }

    pub fn contains_token(&self, token_string: &str) -> Result<bool> {
        let token_store = self
            .token_lookup
            .as_ref()
            .ok_or_else(|| anyhow!("Token store not loaded!"))?;
        Ok(token_store.get(token_string).is_some())
    }

    fn rebuild_token_lookup(&mut self) -> Result<()> {
        let Some(token_map) = self.tokens.as_mut() else {
            return Err(anyhow!("Token store not yet loaded"));
        };
        let mut token_lookup = HashSet::new();
        token_map.values().for_each(|token| {
            token_lookup.insert(token.1.clone());
        });
        self.token_lookup = Some(token_lookup);
        Ok(())
    }

    pub fn create(&mut self, token_label: &str) -> Result<Token> {
        let Some(token_map) = self.tokens.as_mut() else {
            return Err(anyhow!("Token store not yet loaded"));
        };
        if token_map.contains_key(token_label) {
            return Err(anyhow!("Labels must be unique!"));
        }
        let new_token = Token(token_label.to_string(), Uuid::new_v4().to_string());
        token_map.insert(token_label.to_string(), new_token.clone());
        self.rebuild_token_lookup()?;
        self.persist_to_file()?;
        Ok(new_token)
    }

    pub fn rescind(&mut self, token_label: &str) -> Result<()> {
        let Some(token_map) = self.tokens.as_mut() else {
            return Err(anyhow!("Token store not yet loaded"));
        };
        if !token_map.contains_key(token_label) {
            return Err(anyhow!("No token associated with key!"));
        }
        token_map.remove(token_label);
        self.rebuild_token_lookup()?;
        self.persist_to_file()?;
        Ok(())
    }

    pub fn iter(&self) -> Result<impl Iterator<Item = &Token>> {
        self.tokens
            .as_ref()
            .ok_or_else(|| anyhow!("Token store not yet loaded"))
            .map(|token_map| token_map.values())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tempfile::tempdir;

    #[test]
    fn test_new_token_store() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("token_store.txt");
        let token_store = TokenStore::new(&file_path)?;
        assert!(token_store.tokens.unwrap().is_empty());
        assert!(token_store.token_lookup.unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn test_reload_empty_store() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("token_store.txt");
        let mut token_store = TokenStore::new(&file_path)?;
        token_store.reload()?;
        assert!(token_store.tokens.unwrap().is_empty());
        assert!(token_store.token_lookup.unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn test_reload_with_existing_tokens() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("token_store.txt");

        // Setup initial file with one token
        let mut file = File::create(&file_path)?;
        writeln!(file, "label1:uuid1")?;

        let mut token_store = TokenStore::new(&file_path)?;
        token_store.reload()?;
        assert_eq!(token_store.tokens.unwrap().len(), 1);
        assert!(token_store.token_lookup.unwrap().contains("uuid1"));
        Ok(())
    }

    #[test]
    fn test_persist_to_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("token_store.txt");

        let mut token_store = TokenStore::new(&file_path)?;
        let token_label = "label";
        let token = Token(token_label.to_string(), Uuid::new_v4().to_string());
        token_store.tokens = Some(HashMap::from([(token_label.to_string(), token.clone())]));
        token_store.persist_to_file()?;

        let mut file = File::open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert!(contents.contains(&token.0));
        assert!(contents.contains(&token.1));
        Ok(())
    }

    #[test]
    fn test_contains_token() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("token_store.txt");
        let mut token_store = TokenStore::new(&file_path)?;

        // Create a token and reload the store to simulate normal operation
        token_store.create("label")?;
        token_store.reload()?;

        // Check if the token is contained in the store
        let token = token_store
            .tokens
            .as_ref()
            .expect("works")
            .values()
            .next()
            .unwrap()
            .clone();
        assert!(token_store.contains_token(&token.1)?);
        assert!(!token_store.contains_token("nonexistent_token")?);
        Ok(())
    }

    #[test]
    fn test_create_token() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("token_store.txt");
        let mut token_store = TokenStore::new(&file_path)?;

        // Create a new token and check if it's added
        let token_label = "label";
        let new_token = token_store.create(token_label)?;
        print!("{}", new_token);
        assert_eq!(
            token_store.tokens.unwrap().get(token_label).unwrap().1,
            new_token.1
        );
        assert!(token_store.token_lookup.unwrap().contains(&new_token.1));
        Ok(())
    }
}
