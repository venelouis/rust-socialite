use crate::user::SocialiteUser;
use async_trait::async_trait;

/// The core trait implemented by all OAuth2 providers in Rust Socialite.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Returns the authorization URL to redirect the user to the provider's login screen.
    fn redirect_url(&self) -> String;

    /// Returns the authorization URL with a `state` parameter appended.
    /// It is highly recommended to use this to prevent CSRF attacks.
    fn redirect_url_with_state(&self, state: &str) -> String {
        let separator = if self.redirect_url().contains('?') {
            "&"
        } else {
            "?"
        };
        format!("{}{separator}state={state}", self.redirect_url())
    }

    /// Returns the authorization URL with a PKCE `code_challenge` appended.
    /// Useful for providers that enforce PKCE (like Twitter/X v2).
    fn redirect_url_with_pkce(&self, code_challenge: &str) -> String {
        let separator = if self.redirect_url().contains('?') {
            "&"
        } else {
            "?"
        };
        format!(
            "{}{}code_challenge={}&code_challenge_method=S256",
            self.redirect_url(),
            separator,
            code_challenge
        )
    }

    /// Returns the authorization URL with a PKCE `code_challenge` and a `state` parameter appended.
    fn redirect_url_with_pkce_and_state(&self, code_challenge: &str, state: &str) -> String {
        let separator = if self.redirect_url().contains('?') {
            "&"
        } else {
            "?"
        };
        format!(
            "{}{}code_challenge={}&code_challenge_method=S256&state={}",
            self.redirect_url(),
            separator,
            code_challenge,
            state
        )
    }

    /// Exchanges the authorization code for an access token and fetches the user's profile.
    /// Returns a standardized `SocialiteUser` or a `SocialiteError`.
    async fn get_user(
        &self,
        auth_code: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError>;

    /// Exchanges the authorization code for an access token using a PKCE `code_verifier`.
    /// Fallbacks to standard `get_user` by default. Must be overridden by PKCE-enforcing providers.
    async fn get_user_with_pkce(
        &self,
        auth_code: &str,
        _code_verifier: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        self.get_user(auth_code).await
    }

    /// Fetches the user's profile using an existing access token.
    /// This bypasses the authorization code exchange step.
    async fn get_user_from_token(
        &self,
        access_token: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError>;

    /// Revokes an access token (or refresh token) directly on the provider's authorization server.
    /// By default, this returns a `Token` error since not all providers support token revocation.
    async fn revoke_token(&self, _token: &str) -> Result<(), crate::error::SocialiteError> {
        Err(crate::error::SocialiteError::Token(
            "Token revocation is not supported by this provider".to_string(),
        ))
    }
}
