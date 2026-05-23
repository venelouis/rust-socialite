use crate::user::SocialiteUser;
use async_trait::async_trait;

/// The core trait implemented by all OAuth2 providers in Rust Socialite.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Returns the authorization URL to redirect the user to the provider's login screen.
    fn redirect_url(&self) -> String;

    /// Returns the authorization URL with a PKCE `code_challenge` appended.
    /// Useful for providers that enforce PKCE (like Twitter/X v2).
    fn redirect_url_with_pkce(&self, code_challenge: &str) -> String {
        let separator = if self.redirect_url().contains('?') { "&" } else { "?" };
        format!(
            "{}{}code_challenge={}&code_challenge_method=S256",
            self.redirect_url(), separator, code_challenge
        )
    }

    /// Exchanges the authorization code for an access token and fetches the user's profile.
    /// Returns a standardized `SocialiteUser` or a `SocialiteError`.
    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError>;

    /// Exchanges the authorization code for an access token using a PKCE `code_verifier`.
    /// Fallbacks to standard `get_user` by default. Must be overridden by PKCE-enforcing providers.
    async fn get_user_with_pkce(&self, auth_code: &str, _code_verifier: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        self.get_user(auth_code).await
    }
}
