use crate::client::HttpClientExt;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(TwitchProvider, "user:read:email");

#[async_trait]
impl Provider for TwitchProvider {
    fn redirect_url(&self) -> String {
        let mut params = url::form_urlencoded::Serializer::new(String::with_capacity(256));
        params.append_pair("client_id", &self.client_id);
        params.append_pair("redirect_uri", &self.redirect_url);
        params.append_pair("response_type", "code");
        if !self.scopes.is_empty() {
            params.append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            params.append_pair("state", state);
        }

        if let Some(pkce) = &self.pkce_challenge {
            params.append_pair("code_challenge", pkce);
            params.append_pair("code_challenge_method", "S256");
        }
        format!("https://id.twitch.tv/oauth2/authorize?{}", params.finish())
    }

    async fn get_user(
        &self,
        auth_code: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self
            .http_client
            .post(self.token_url())
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"].as_str().ok_or_else(|| {
            crate::error::SocialiteError::Token("Failed to get access_token".to_string())
        })?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"]
            .as_str()
            .map(|s: &str| s.to_string());
        user.expires_in = token_res["expires_in"]
            .as_u64()
            .or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }

    async fn get_user_from_token(
        &self,
        access_token: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let user_res = self
            .http_client
            .get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Client-Id", self.client_id.as_str())
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let user_data = user_res["data"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                crate::error::SocialiteError::Provider("No user data returned".to_string())
            })?;

        Ok(SocialiteUser {
            id: user_data["id"].as_str().unwrap_or("").to_string(),
            name: user_data["display_name"].as_str().unwrap_or("").to_string(),
            email: user_data["email"].as_str().map(|s: &str| s.to_string()),
            avatar_url: user_data["profile_image_url"]
                .as_str()
                .map(|s: &str| s.to_string()),
            raw_data: user_data.clone(),
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }

    fn token_url(&self) -> String {
        "https://id.twitch.tv/oauth2/token".to_string()
    }

    async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self
            .http_client
            .post(self.token_url())
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        if let Some(err) = token_res["error"].as_str() {
            let err_desc = token_res["error_description"].as_str().unwrap_or("");
            return Err(crate::error::SocialiteError::Token(format!(
                "Provider returned error: {} - {}",
                err, err_desc
            )));
        }

        let access_token = token_res["access_token"].as_str().ok_or_else(|| {
            crate::error::SocialiteError::Token(
                "Failed to get access_token during refresh".to_string(),
            )
        })?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"]
            .as_u64()
            .or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }
}
