use crate::error::SocialiteError;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

crate::define_provider!(RedditProvider, "identity");

#[async_trait]
impl Provider for RedditProvider {
    fn redirect_url(&self) -> String {
        let mut url = url::Url::parse("https://www.reddit.com/api/v1/authorize")
            .expect("Invalid redirect URL");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        url.query_pairs_mut().append_pair("response_type", "code");
        url.query_pairs_mut().append_pair("state", "socialite");
        url.query_pairs_mut()
            .append_pair("redirect_uri", &self.redirect_url);
        url.query_pairs_mut().append_pair("duration", "temporary");
        if !self.scopes.is_empty() {
            url.query_pairs_mut()
                .append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut().append_pair("state", state);
        }

        if let Some(pkce) = &self.pkce_challenge {
            url.query_pairs_mut().append_pair("code_challenge", pkce);
            url.query_pairs_mut()
                .append_pair("code_challenge_method", "S256");
        }
        url.into()
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let token_res = self
            .http_client
            .post("https://www.reddit.com/api/v1/access_token")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"]
            .as_u64()
            .or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }

    async fn get_user_from_token(
        &self,
        access_token: &str,
    ) -> Result<SocialiteUser, SocialiteError> {
        let user_res = self
            .http_client
            .get("https://oauth.reddit.com/api/v1/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "rust-socialite/0.2.1")
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: None, // Reddit identity scope does not provide email by default
            avatar_url: user_res["icon_img"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
