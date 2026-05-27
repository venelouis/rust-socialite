use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};

crate::define_provider!(ZoomProvider);

#[async_trait]
impl Provider for ZoomProvider {
    fn redirect_url(&self) -> String {
        let mut url = url::Url::parse("https://zoom.us/oauth/authorize").unwrap();
        url.query_pairs_mut().append_pair("response_type", "code");
        url.query_pairs_mut().append_pair("client_id", &self.client_id);
        url.query_pairs_mut().append_pair("redirect_uri", &self.redirect_url);
        if !self.scopes.is_empty() {
            url.query_pairs_mut().append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut().append_pair("state", state);
        }
        url.into()
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let token_res = self.http_client.post("https://zoom.us/oauth/token")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"].as_u64().or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }


    async fn get_user_from_token(&self, access_token: &str) -> Result<SocialiteUser, SocialiteError> {
        let user_res = self.http_client.get("https://api.zoom.us/v2/users/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let first_name = user_res["first_name"].as_str().unwrap_or("");
        let last_name = user_res["last_name"].as_str().unwrap_or("");
        let name = format!("{} {}", first_name, last_name).trim().to_string();

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name,
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["pic_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }}
