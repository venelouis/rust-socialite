use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct BitbucketProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl BitbucketProvider {
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
impl Provider for BitbucketProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://bitbucket.org/site/oauth2/authorize?client_id={}&response_type=code",
            self.client_id
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self.http_client.post("https://bitbucket.org/site/oauth2/access_token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("grant_type", "authorization_code"),
            ])
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"].as_str().ok_or_else(|| crate::error::SocialiteError::Token("Failed to get access_token".to_string()))?;

        let user_res = self.http_client.get("https://api.bitbucket.org/2.0/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let emails_res = self.http_client.get("https://api.bitbucket.org/2.0/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let email = emails_res["values"].as_array()
            .and_then(|vals| vals.iter().find(|v| v["is_primary"].as_bool().unwrap_or(false)))
            .and_then(|v| v["email"].as_str())
            .map(|s| s.to_string());

        Ok(SocialiteUser {
            id: user_res["account_id"].as_str().unwrap_or("").to_string(),
            name: user_res["display_name"].as_str().unwrap_or("").to_string(),
            email,
            avatar_url: user_res["links"]["avatar"]["href"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

