use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a standardized user profile returned from any OAuth2 provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialiteUser {
    /// The unique identifier of the user in the provider's system.
    pub id: String,
    
    /// The full name or display name of the user.
    pub name: String,
    
    /// The email address of the user, if available and granted.
    pub email: Option<String>,
    
    /// The URL to the user's avatar/profile picture, if available.
    pub avatar_url: Option<String>,
    
    /// The raw JSON response received from the provider's user endpoint.
    /// Useful for extracting provider-specific fields not covered by this struct.
    pub raw_data: Value,
    
    /// The access token retrieved during the OAuth2 flow.
    pub access_token: String,
    
    /// The refresh token retrieved during the OAuth2 flow (if provided).
    pub refresh_token: Option<String>,
    
    /// The token expiration time in seconds from the time it was granted (if provided).
    pub expires_in: Option<u64>,
}
