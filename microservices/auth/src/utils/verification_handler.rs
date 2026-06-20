//! # Password Verification
//!
//! This module provides functionality for verifying passwords against Argon2 hashes.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};

/// Verifies a plain-text string against a hashed string using Argon2.
///
/// Returns `Ok(true)` when the password matches, `Ok(false)` when it does not (including cases where verification fails and is converted via `.is_ok()`), and only returns `Err` on hash parsing failures.
pub async fn verification_handler(
    string_to_compare: &str,
    hashed_string: &str,
) -> Result<bool, argon2::password_hash::Error> {
    // Parse the stored hash
    let parsed_hash = PasswordHash::new(hashed_string)?;

    // Verify the password
    let is_valid = Argon2::default()
        .verify_password(string_to_compare.as_bytes(), &parsed_hash)
        .is_ok(); // returns true if verification succeeded

    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::hashing_handler::hashing_handler;

    #[tokio::test]
    async fn test_verification_handler_success() {
        let password = "my_secure_password";
        let hash = hashing_handler(password).await.unwrap();

        let result = verification_handler(password, &hash).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_verification_handler_failure() {
        let password = "my_secure_password";
        let wrong_password = "wrong_password";
        let hash = hashing_handler(password).await.unwrap();

        let result = verification_handler(wrong_password, &hash).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
