use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct DigitaloceanProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl DigitaloceanProvider {
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
impl Provider for DigitaloceanProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://cloud.digitalocean.com/v1/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://cloud.digitalocean.com/v1/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
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

        let user_res = self.http_client.get("https://api.digitalocean.com/v2/account")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let account = &user_res["account"];

        Ok(SocialiteUser {
            id: account["uuid"].as_str().unwrap_or("").to_string(),
            name: String::new(), // DigitalOcean does not provide a display name via API
            email: account["email"].as_str().map(|s| s.to_string()),
            avatar_url: None, // No avatar provided
            raw_data: user_res,
        })
    }
}
