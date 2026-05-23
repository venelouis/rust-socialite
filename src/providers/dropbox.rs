use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct DropboxProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl DropboxProvider {
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
impl Provider for DropboxProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://www.dropbox.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://api.dropboxapi.com/oauth2/token")
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

        let user_res = self.http_client.post("https://api.dropboxapi.com/2/users/get_current_account")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["account_id"].as_str().unwrap_or("").to_string(),
            name: user_res["name"]["display_name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["profile_photo_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}
