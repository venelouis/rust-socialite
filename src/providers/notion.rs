use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};

pub struct NotionProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl NotionProvider {
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
impl Provider for NotionProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://api.notion.com/v1/oauth/authorize?client_id={}&response_type=code&owner=user&redirect_uri={}",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let token_res = self.http_client.post("https://api.notion.com/v1/oauth/token")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .json(&serde_json::json!({
                "grant_type": "authorization_code",
                "code": auth_code,
                "redirect_uri": self.redirect_url.as_str()
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let owner = &token_res["owner"]["user"];

        Ok(SocialiteUser {
            id: owner["id"].as_str().unwrap_or("").to_string(),
            name: owner["name"].as_str().unwrap_or("").to_string(),
            email: owner["person"]["email"].as_str().map(|s| s.to_string()),
            avatar_url: owner["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: token_res, // Notion returns user data right in the token response
        })
    }
}
