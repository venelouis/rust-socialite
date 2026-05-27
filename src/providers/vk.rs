use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct VkProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl VkProvider {
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
impl Provider for VkProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://oauth.vk.com/authorize?client_id={}&display=page&redirect_uri={}&response_type=code&v=5.131",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.get(format!(
            "https://oauth.vk.com/access_token?client_id={}&client_secret={}&redirect_uri={}&code={}",
            self.client_id, self.client_secret, self.redirect_url, auth_code
        ))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        // VK often returns email directly in the token response
        let email = token_res["email"].as_str().map(|s| s.to_string());
        
        let user_id = token_res["user_id"]
            .as_i64()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "".to_string());

        let user_res = self.http_client.get(format!(
            "https://api.vk.com/method/users.get?user_ids={}&fields=photo_200&v=5.131&access_token={}",
            user_id, access_token
        ))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let user_data = &user_res["response"][0];
        let first_name = user_data["first_name"].as_str().unwrap_or("");
        let last_name = user_data["last_name"].as_str().unwrap_or("");
        let name = format!("{} {}", first_name, last_name).trim().to_string();

        Ok(SocialiteUser {
            id: user_data["id"].as_i64().map(|i| i.to_string()).unwrap_or(user_id),
            name,
            email,
            avatar_url: user_data["photo_200"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
        })
    }
}

