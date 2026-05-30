use crate::provider::Provider;
use crate::user::ConnectUser;
use async_trait::async_trait;

/// A mock provider specifically designed for testing and TDD.
/// It returns a pre-configured `ConnectUser` without performing any HTTP requests.
pub struct MockProvider {
    mocked_user: ConnectUser,
    mocked_url: String,
    expect_revoke_success: bool,
}

impl MockProvider {
    /// Creates a new `MockProvider` with a static user and login URL.
    pub fn new(user: ConnectUser, url: String) -> Self {
        Self {
            mocked_user: user,
            mocked_url: url,
            expect_revoke_success: true,
        }
    }

    /// Sets whether the `revoke_token` method should succeed or fail.
    pub fn with_revoke_success(mut self, success: bool) -> Self {
        self.expect_revoke_success = success;
        self
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn redirect_url(&self) -> String {
        self.mocked_url.clone()
    }

    fn token_url(&self) -> String {
        "https://mock.provider/token".to_string()
    }

    async fn get_user(
        &self,
        _auth_code: &str,
    ) -> Result<ConnectUser, crate::error::ConnectError> {
        Ok(self.mocked_user.clone())
    }

    async fn get_user_from_token(
        &self,
        _access_token: &str,
    ) -> Result<ConnectUser, crate::error::ConnectError> {
        Ok(self.mocked_user.clone())
    }

    async fn revoke_token(&self, _token: &str) -> Result<(), crate::error::ConnectError> {
        if self.expect_revoke_success {
            Ok(())
        } else {
            Err(crate::error::ConnectError::Token(
                "Mocked revocation failure".to_string(),
            ))
        }
    }
}
