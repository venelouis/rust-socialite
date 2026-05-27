use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct XProvider {
    client_id: String,
    redirect_url: String,
    http_client: Client,
}

impl XProvider {
    /// Note: X (Twitter) OAuth v2 uses only a Client ID for public/confidential clients doing PKCE.
    pub fn new(client_id: String, redirect_url: String) -> Self {
        Self {
            client_id,
            redirect_url,
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Provider for XProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://twitter.com/i/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope=users.read%20tweet.read&state=state",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, _auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        Err(SocialiteError::Provider("X (Twitter) requires PKCE. Use get_user_with_pkce instead.".to_string()))
    }

    async fn get_user_with_pkce(&self, auth_code: &str, code_verifier: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://api.twitter.com/2/oauth2/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", self.client_id.as_str()),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
                ("code_verifier", code_verifier),
            ])
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        let user_res = self.http_client.get("https://api.twitter.com/2/users/me?user.fields=profile_image_url")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let data = &user_res["data"];

        Ok(SocialiteUser {
            id: data["id"].as_str().unwrap_or("").to_string(),
            name: data["name"].as_str().unwrap_or("").to_string(),
            email: None, // X v2 does not return email via this endpoint by default
            avatar_url: data["profile_image_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

