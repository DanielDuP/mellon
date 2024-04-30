use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::ErrorKind;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;

use super::token::Token;
use anyhow::{anyhow, Result};
use uuid::Uuid;

pub struct TokenStore {
    file_path: PathBuf,
    tokens: Option<HashMap<String, Token>>, // Stores all token objects in memory
    token_lookup: Option<HashSet<String>>,  // Quickly checks if a token string is authorized
}

impl TokenStore {
    pub fn new(file_path: String) -> Result<Self> {
        let store_path = PathBuf::from(file_path);
        if let Some(dir_path) = store_path.parent() {
            if !dir_path.exists() {
                fs::create_dir_all(dir_path).expect("Failed to create directory");
            }
        }
        let mut token_store = TokenStore {
            file_path: store_path,
            tokens: None,
            token_lookup: None,
        };
        token_store.reload()?;
        Ok(token_store)
    }

    pub fn reload(&mut self) -> Result<()> {
        let file = match File::open(self.file_path.clone()) {
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
        let file = File::create(self.file_path.clone())?;
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
