use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct InstagramProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl InstagramProvider {
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
impl Provider for InstagramProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://api.instagram.com/oauth/authorize?client_id={}&redirect_uri={}&scope=user_profile&response_type=code",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://api.instagram.com/oauth/access_token")
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

        let user_res = self.http_client.get(format!("https://graph.instagram.com/me?fields=id,username&access_token={}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: user_res["username"].as_str().unwrap_or("").to_string(),
            email: None,
            avatar_url: None, // Instagram Basic Display does not provide profile picture
            raw_data: user_res,
        })
    }
}
