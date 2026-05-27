use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(VkProvider);

#[async_trait]
impl Provider for VkProvider {
    fn redirect_url(&self) -> String {
        let mut url = url::Url::parse("https://oauth.vk.com/authorize").unwrap();
        url.query_pairs_mut().append_pair("client_id", &self.client_id);
        url.query_pairs_mut().append_pair("display", "page");
        url.query_pairs_mut().append_pair("redirect_uri", &self.redirect_url);
        url.query_pairs_mut().append_pair("response_type", "code");
        url.query_pairs_mut().append_pair("v", "5.131");
        if !self.scopes.is_empty() {
            url.query_pairs_mut().append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut().append_pair("state", state);
        }
        url.into()
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

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"].as_u64().or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        
        // Overwrite email if it was returned in the token exchange (VK specific)
        if let Some(email) = token_res["email"].as_str() {
            user.email = Some(email.to_string());
        }
        
        // Overwrite ID if it was returned in the token exchange just to be safe
        if let Some(user_id) = token_res["user_id"].as_i64() {
            user.id = user_id.to_string();
        }

        Ok(user)
    }

    async fn get_user_from_token(&self, access_token: &str) -> Result<SocialiteUser, SocialiteError> {
        let user_res = self.http_client.get(format!(
            "https://api.vk.com/method/users.get?fields=photo_200&v=5.131&access_token={}",
            access_token
        ))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let user_data = &user_res["response"][0];
        let first_name = user_data["first_name"].as_str().unwrap_or("");
        let last_name = user_data["last_name"].as_str().unwrap_or("");
        let name = format!("{} {}", first_name, last_name).trim().to_string();

        Ok(SocialiteUser {
            id: user_data["id"].as_i64().map(|i| i.to_string()).unwrap_or_else(|| "".to_string()),
            name,
            email: None, // Email is generally not available in users.get unless specified and granted
            avatar_url: user_data["photo_200"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}

