//! Error types for mx-lib

use thiserror::Error;

/// The main error type for mx-lib
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Not a MechCrate project")]
    NotAProject,

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Recipe not found: {0}")]
    RecipeNotFound(String),

    #[error("Invalid recipe: {0}")]
    InvalidRecipe(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Authentication required")]
    AuthRequired,

    #[error("API error: {0}")]
    Api(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for mx-lib
pub type Result<T> = std::result::Result<T, Error>;
