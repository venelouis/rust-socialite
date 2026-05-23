use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct Auth0Provider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    domain: String,
    http_client: Client,
}

impl Auth0Provider {
    /// Note: domain should be the tenant domain, e.g., "dev-xxxx.us.auth0.com"
    pub fn new(client_id: String, client_secret: String, redirect_url: String, domain: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            domain,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for Auth0Provider {
    fn redirect_url(&self) -> String {
        format!(
            "https://{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email",
            self.domain, self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post(format!("https://{}/oauth/token", self.domain))
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

        let user_res = self.http_client.get(format!("https://{}/userinfo", self.domain))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
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
