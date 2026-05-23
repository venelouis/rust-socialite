use crate::user::SocialiteUser;
use async_trait::async_trait;

/// The core trait implemented by all OAuth2 providers in Rust Socialite.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Returns the authorization URL to redirect the user to the provider's login screen.
    fn redirect_url(&self) -> String;

    /// Exchanges the authorization code for an access token and fetches the user's profile.
    /// Returns a standardized `SocialiteUser` or a `SocialiteError`.
    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError>;
}
