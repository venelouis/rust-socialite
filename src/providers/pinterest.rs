use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};

pub struct PinterestProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl PinterestProvider {
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
impl Provider for PinterestProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://www.pinterest.com/oauth/?client_id={}&redirect_uri={}&response_type=code&scope=user_accounts:read",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let token_res = self.http_client.post("https://api.pinterest.com/v5/oauth/token")
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

        let user_res = self.http_client.get("https://api.pinterest.com/v5/user_account")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["username"].as_str().unwrap_or("").to_string(),
            name: user_res["username"].as_str().unwrap_or("").to_string(), // Pinterest relies on username
            email: None,
            avatar_url: user_res["profile_image"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

