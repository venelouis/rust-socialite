use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct GitlabProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl GitlabProvider {
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
impl Provider for GitlabProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://gitlab.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read_user",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self.http_client.post("https://gitlab.com/oauth/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"].as_str().ok_or_else(|| crate::error::SocialiteError::Token("Failed to get access_token".to_string()))?;

        let user_res = self.http_client.get("https://gitlab.com/api/v4/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["id"].as_i64().unwrap_or(0).to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

