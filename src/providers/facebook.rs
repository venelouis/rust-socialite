use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct FacebookProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl FacebookProvider {
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
impl Provider for FacebookProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&scope=email,public_profile",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self.http_client.post("https://graph.facebook.com/v19.0/oauth/access_token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"].as_str().ok_or_else(|| crate::error::SocialiteError::Token("Failed to get access_token".to_string()))?;

        let user_res = self.http_client.get(format!(
            "https://graph.facebook.com/v19.0/me?fields=id,name,email,picture.width(500).height(500)&access_token={}",
            access_token
        ))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let avatar = user_res["picture"]["data"]["url"].as_str().map(|s| s.to_string());

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: avatar,
            raw_data: user_res,
        })
    }
}
