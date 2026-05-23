use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct PatreonProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl PatreonProvider {
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
impl Provider for PatreonProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://www.patreon.com/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope=identity%20identity[email]",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://www.patreon.com/api/oauth2/token")
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

        let user_res = self.http_client.get("https://www.patreon.com/api/oauth2/v2/identity?fields[user]=email,full_name,image_url")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let user_data = &user_res["data"];
        let attributes = &user_data["attributes"];

        Ok(SocialiteUser {
            id: user_data["id"].as_str().unwrap_or("").to_string(),
            name: attributes["full_name"].as_str().unwrap_or("").to_string(),
            email: attributes["email"].as_str().map(|s| s.to_string()),
            avatar_url: attributes["image_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}
