//! User Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User creation command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
}

/// User update command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// User-related errors
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    
    #[error("Email already exists")]
    EmailExists,
    
    #[error("Invalid email format")]
    InvalidEmail,
    
    #[error("Name too short (min 2 characters)")]
    NameTooShort,
    
    #[error("Name too long (max 100 characters)")]
    NameTooLong,
}

impl User {
    /// Create a new user (factory method)
    pub fn new(cmd: CreateUser) -> Result<Self, UserError> {
        // Validate
        if cmd.name.len() < 2 {
            return Err(UserError::NameTooShort);
        }
        if cmd.name.len() > 100 {
            return Err(UserError::NameTooLong);
        }
        if !cmd.email.contains('@') {
            return Err(UserError::InvalidEmail);
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            email: cmd.email,
            name: cmd.name,
            created_at: now,
            updated_at: now,
        })
    }

    /// Apply update to user (pure transformation)
    pub fn apply_update(mut self, cmd: UpdateUser) -> Result<Self, UserError> {
        if let Some(name) = cmd.name {
            if name.len() < 2 {
                return Err(UserError::NameTooShort);
            }
            if name.len() > 100 {
                return Err(UserError::NameTooLong);
            }
            self.name = name;
        }

        if let Some(email) = cmd.email {
            if !email.contains('@') {
                return Err(UserError::InvalidEmail);
            }
            self.email = email;
        }

        self.updated_at = Utc::now();
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_valid() {
        let user = User::new(CreateUser {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
        });
        
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
    }

    #[test]
    fn test_create_user_invalid_email() {
        let result = User::new(CreateUser {
            email: "invalid".to_string(),
            name: "Test".to_string(),
        });
        
        assert!(matches!(result, Err(UserError::InvalidEmail)));
    }

    #[test]
    fn test_create_user_name_too_short() {
        let result = User::new(CreateUser {
            email: "test@example.com".to_string(),
            name: "A".to_string(),
        });
        
        assert!(matches!(result, Err(UserError::NameTooShort)));
    }
}
