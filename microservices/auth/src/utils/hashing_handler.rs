//! # Password Hashing
//!
//! This module provides functionality for hashing passwords using the Argon2 algorithm.

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

/// Hashes a plain-text string using Argon2 with a random salt.
///
/// Returns the hashed string in PHC format, or an `Err` if hashing fails.
pub async fn hashing_handler(string_to_hash: &str) -> Result<String, argon2::password_hash::Error> {
    let password = string_to_hash.to_string();

    tokio::task::spawn_blocking(move || {
        // Generate a random 16-byte salt
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

        Ok(password_hash.to_string())
    })
    .await
    .map_err(|_| argon2::password_hash::Error::Password)?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hashing_handler_success() {
        let password = "my_secure_password";
        let result = hashing_handler(password).await;
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(hash.contains("$argon2id$"));
    }

    #[tokio::test]
    async fn test_hashing_produces_unique_hashes() {
        let password = "same_password";
        let hash1 = hashing_handler(password).await.unwrap();
        let hash2 = hashing_handler(password).await.unwrap();
        assert_ne!(hash1, hash2, "Each hash should use a unique salt");
    }
}
