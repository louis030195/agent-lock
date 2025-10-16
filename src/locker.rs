use anyhow::Result;
use std::io::{self, Write};

pub fn prompt_for_unlock() -> Result<bool> {
    print!("Enter PIN to unlock: ");
    io::stdout().flush()?;

    let pin = rpassword::read_password()?;

    match crate::auth::verify_pin_internal(&pin) {
        true => {
            println!("✓ Unlocked successfully!");
            Ok(true)
        }
        false => {
            println!("✗ Incorrect PIN. Try again.");
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_for_unlock_compiles() {
        assert!(true);
    }
}
