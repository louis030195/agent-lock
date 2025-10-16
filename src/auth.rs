use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    pub pin_hash: String,
}

impl AuthConfig {
    pub fn new(pin: &str) -> Self {
        Self {
            pin_hash: Self::hash_pin(pin),
        }
    }

    fn hash_pin(pin: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn verify(&self, pin: &str) -> bool {
        self.pin_hash == Self::hash_pin(pin)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).context("Failed to save auth config")?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let json = fs::read_to_string(path).context("Failed to read auth config")?;
        let config: AuthConfig = serde_json::from_str(&json)?;
        Ok(config)
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("screen-locker");

    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("auth.json"))
}

pub fn setup_pin() -> Result<()> {
    println!("Set up your PIN (4-8 digits):");
    let pin = rpassword::read_password()?;

    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_numeric()) {
        anyhow::bail!("PIN must be 4-8 digits");
    }

    println!("Confirm your PIN:");
    let confirm = rpassword::read_password()?;

    if pin != confirm {
        anyhow::bail!("PINs do not match");
    }

    let config = AuthConfig::new(&pin);
    let path = get_config_path()?;
    config.save(&path)?;

    println!("PIN configured successfully!");
    Ok(())
}

pub fn verify_pin() -> Result<bool> {
    let pin = rpassword::read_password()?;
    Ok(verify_pin_internal(&pin))
}

pub fn verify_pin_internal(pin: &str) -> bool {
    match get_config_path() {
        Ok(path) => match AuthConfig::load(&path) {
            Ok(config) => config.verify(pin),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let pin = "1234";
        let hash1 = AuthConfig::hash_pin(pin);
        let hash2 = AuthConfig::hash_pin(pin);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_correct_pin() {
        let config = AuthConfig::new("1234");
        assert!(config.verify("1234"));
    }

    #[test]
    fn test_verify_incorrect_pin() {
        let config = AuthConfig::new("1234");
        assert!(!config.verify("5678"));
    }
}
