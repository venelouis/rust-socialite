use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct LineProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl LineProvider {
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
impl Provider for LineProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://access.line.me/oauth2/v2.1/authorize?response_type=code&client_id={}&redirect_uri={}&state=socialite&scope=profile%20openid%20email",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://api.line.me/oauth2/v2.1/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", self.client_id.as_str()),
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

        let user_res = self.http_client.get("https://api.line.me/v2/profile")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["userId"].as_str().unwrap_or("").to_string(),
            name: user_res["displayName"].as_str().unwrap_or("").to_string(),
            email: None, // Line email requires parsing the 'id_token' JWT payload. We omit it here for simplicity.
            avatar_url: user_res["pictureUrl"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

