use crate::error::SocialiteError;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

crate::define_provider!(NotionProvider);

#[async_trait]
impl Provider for NotionProvider {
    fn redirect_url(&self) -> String {
        let mut url = url::Url::parse("https://api.notion.com/v1/oauth/authorize").expect("Invalid authorization URL");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        url.query_pairs_mut().append_pair("response_type", "code");
        url.query_pairs_mut().append_pair("owner", "user");
        url.query_pairs_mut()
            .append_pair("redirect_uri", &self.redirect_url);
        crate::utils::append_auth_params(
            &mut url.query_pairs_mut(),
            &self.scopes,
            &self.state,
            &self.pkce_challenge,
        );

        url.into()
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let token_res = self
            .http_client
            .post("https://api.notion.com/v1/oauth/token")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .json(&serde_json::json!({
                "grant_type": "authorization_code",
                "code": auth_code,
                "redirect_uri": self.redirect_url.as_str()
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let owner = &token_res["owner"]["user"];

        let access_token = token_res["access_token"].as_str().unwrap_or("").to_string();

        Ok(SocialiteUser {
            id: owner["id"].as_str().unwrap_or("").to_string(),
            name: owner["name"].as_str().unwrap_or("").to_string(),
            email: owner["person"]["email"].as_str().map(|s| s.to_string()),
            avatar_url: owner["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: token_res.clone(), // Notion returns user data right in the token response
            access_token,
            refresh_token: token_res["refresh_token"].as_str().map(|s| s.to_string()),
            expires_in: token_res["expires_in"]
                .as_u64()
                .or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64)),
        })
    }

    async fn get_user_from_token(
        &self,
        access_token: &str,
    ) -> Result<SocialiteUser, SocialiteError> {
        let user_res = self
            .http_client
            .get("https://api.notion.com/v1/users/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await?
            .json::<Value>()
            .await?;

        let user = if user_res["type"].as_str() == Some("bot") {
            &user_res["bot"]["owner"]["user"]
        } else {
            &user_res
        };

        Ok(SocialiteUser {
            id: user["id"].as_str().unwrap_or("").to_string(),
            name: user["name"].as_str().unwrap_or("").to_string(),
            email: user["person"]["email"].as_str().map(|s| s.to_string()),
            avatar_url: user["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
