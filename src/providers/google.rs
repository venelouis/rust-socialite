use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use url::form_urlencoded;

pub struct GoogleProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl GoogleProvider {
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
impl Provider for GoogleProvider {
    fn redirect_url(&self) -> String {
        let params = form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_url)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid profile email")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .finish();

        format!("https://accounts.google.com/o/oauth2/v2/auth?{}", params)
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        // Exchange code for token
        let token_res = self.http_client.post("https://oauth2.googleapis.com/token")
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

        // Fetch user profile
        let user_res = self.http_client.get("https://www.googleapis.com/oauth2/v3/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["sub"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["picture"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

