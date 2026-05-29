use thiserror::Error;

/// Official errors of the Rust Socialite library
#[derive(Error, Debug)]
pub enum SocialiteError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Failed to decode Base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("JWT processing failed: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("System time error: {0}")]
    Time(#[from] std::time::SystemTimeError),

    #[error("Missing token or unexpected response: {0}")]
    Token(String),

    #[error("Provider specific error: {0}")]
    Provider(String),

    #[error("Invalid CSRF state: {0}")]
    InvalidState(String),
}
