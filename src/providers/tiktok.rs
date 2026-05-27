use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct TiktokProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl TiktokProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Provider for TiktokProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://www.tiktok.com/v2/auth/authorize?client_key={}&response_type=code&scope=user.info.basic&redirect_uri={}",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://open.tiktokapis.com/v2/oauth/token/")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_key", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

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
        })
    }
}

