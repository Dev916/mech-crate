//! User Service
//!
//! Pure business logic for user operations.
//! Validates inputs and applies transformations.

use crate::domain::models::{CreateUser, UpdateUser, User, UserError};

/// Validate user creation input
pub fn validate_create_user(cmd: &CreateUser) -> Result<(), UserError> {
    if cmd.name.len() < 2 {
        return Err(UserError::NameTooShort);
    }
    if cmd.name.len() > 100 {
        return Err(UserError::NameTooLong);
    }
    if !cmd.email.contains('@') {
        return Err(UserError::InvalidEmail);
    }
    Ok(())
}

/// Validate user update input
pub fn validate_update_user(cmd: &UpdateUser) -> Result<(), UserError> {
    if let Some(ref name) = cmd.name {
        if name.len() < 2 {
            return Err(UserError::NameTooShort);
        }
        if name.len() > 100 {
            return Err(UserError::NameTooLong);
        }
    }
    if let Some(ref email) = cmd.email {
        if !email.contains('@') {
            return Err(UserError::InvalidEmail);
        }
    }
    Ok(())
}

/// Check if email is a valid format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// Normalize email (lowercase, trim)
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Create a display name from email if name is not provided
pub fn email_to_display_name(email: &str) -> String {
    email
        .split('@')
        .next()
        .unwrap_or("User")
        .replace(['.', '_', '-'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_email() {
        assert_eq!(normalize_email("  TEST@EXAMPLE.COM  "), "test@example.com");
    }

    #[test]
    fn test_email_to_display_name() {
        assert_eq!(email_to_display_name("john.doe@example.com"), "John Doe");
        assert_eq!(email_to_display_name("jane_smith@example.com"), "Jane Smith");
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("missing@dot"));
    }
}
