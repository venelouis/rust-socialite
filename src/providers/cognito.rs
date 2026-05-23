use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct CognitoProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    domain: String,
    http_client: Client,
}

impl CognitoProvider {
    /// Note: domain should be the full base url, e.g., "https://my-domain.auth.us-east-1.amazoncognito.com"
    pub fn new(client_id: String, client_secret: String, redirect_url: String, domain: String) -> Self {
        let clean_domain = domain.trim_end_matches('/').to_string();
        Self {
            client_id,
            client_secret,
            redirect_url,
            domain: clean_domain,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for CognitoProvider {
    fn redirect_url(&self) -> String {
        format!(
            "{}/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=email%20openid%20profile",
            self.domain, self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post(format!("{}/oauth2/token", self.domain))
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

        let user_res = self.http_client.get(format!("{}/oauth2/userInfo", self.domain))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["sub"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().or_else(|| user_res["username"].as_str()).unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["picture"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}
