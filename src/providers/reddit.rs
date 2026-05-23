use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};

pub struct RedditProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl RedditProvider {
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
impl Provider for RedditProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://www.reddit.com/api/v1/authorize?client_id={}&response_type=code&state=socialite&redirect_uri={}&duration=temporary&scope=identity",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let token_res = self.http_client.post("https://www.reddit.com/api/v1/access_token")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        let user_res = self.http_client.get("https://oauth.reddit.com/api/v1/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "rust-socialite/0.2.1")
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: None, // Reddit identity scope does not provide email by default
            avatar_url: user_res["icon_img"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}
