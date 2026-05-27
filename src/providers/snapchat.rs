use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct SnapchatProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl SnapchatProvider {
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
impl Provider for SnapchatProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://accounts.snapchat.com/login/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=snapchat-api.read",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://accounts.snapchat.com/login/oauth2/access_token")
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

        // Need to use POST to fetch user details with GraphQL equivalent query in Snapchat API
        let query = "{ me { externalId displayName bitmoji { avatar } } }";
        let user_res = self.http_client.post("https://kit.snapchat.com/v1/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&serde_json::json!({ "query": query }))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let me = &user_res["data"]["me"];

        Ok(SocialiteUser {
            id: me["externalId"].as_str().unwrap_or("").to_string(),
            name: me["displayName"].as_str().unwrap_or("").to_string(),
            email: None,
            avatar_url: me["bitmoji"]["avatar"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

