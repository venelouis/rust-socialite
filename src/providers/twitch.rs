use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct TwitchProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl TwitchProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for TwitchProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=user:read:email",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self.http_client.post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"].as_str().ok_or_else(|| crate::error::SocialiteError::Token("Failed to get access_token".to_string()))?;

        let user_res = self.http_client.get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Client-Id", self.client_id.as_str())
            .send()
            .await?
            .json::<Value>()
            .await?;

        let user_data = user_res["data"].as_array().and_then(|arr| arr.first()).ok_or_else(|| crate::error::SocialiteError::Provider("No user data returned".to_string()))?;

        Ok(SocialiteUser {
            id: user_data["id"].as_str().unwrap_or("").to_string(),
            name: user_data["display_name"].as_str().unwrap_or("").to_string(),
            email: user_data["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_data["profile_image_url"].as_str().map(|s| s.to_string()),
            raw_data: user_data.clone(),
        })
    }
}
