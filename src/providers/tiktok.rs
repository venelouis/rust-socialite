use crate::error::SocialiteError;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(TiktokProvider, "user.info.basic");

#[async_trait]
impl Provider for TiktokProvider {
    fn redirect_url(&self) -> String {
        let mut params = url::form_urlencoded::Serializer::new(String::new());
        params
            .append_pair("client_key", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &self.redirect_url);

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
        format!(
            "https://www.tiktok.com/v2/auth/authorize?{}",
            params.finish()
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self
            .http_client
            .post("https://open.tiktokapis.com/v2/oauth/token/")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_key", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
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
        let user_res = self.http_client.get("https://open.tiktokapis.com/v2/user/info/?fields=open_id,union_id,avatar_url,display_name")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let data = &user_res["data"];

        Ok(SocialiteUser {
            id: data["open_id"].as_str().unwrap_or("").to_string(),
            name: data["display_name"].as_str().unwrap_or("").to_string(),
            email: None, // TikTok API v2 does not expose email publicly
            avatar_url: data["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
