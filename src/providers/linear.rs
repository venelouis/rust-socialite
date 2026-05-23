use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct LinearProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl LinearProvider {
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
impl Provider for LinearProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://linear.app/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://api.linear.app/oauth/token")
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

        // Linear exposes user info via GraphQL
        let query = "{ viewer { id name email avatarUrl } }";
        let user_res = self.http_client.post("https://api.linear.app/graphql")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let viewer = &user_res["data"]["viewer"];

        Ok(SocialiteUser {
            id: viewer["id"].as_str().unwrap_or("").to_string(),
            name: viewer["name"].as_str().unwrap_or("").to_string(),
            email: viewer["email"].as_str().map(|s| s.to_string()),
            avatar_url: viewer["avatarUrl"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}
