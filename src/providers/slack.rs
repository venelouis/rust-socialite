use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(SlackProvider);

#[async_trait]
impl Provider for SlackProvider {
    fn redirect_url(&self) -> String {
        let mut url = url::Url::parse("https://slack.com/oauth/v2/authorize").unwrap();
        url.query_pairs_mut().append_pair("client_id", &self.client_id);
        url.query_pairs_mut().append_pair("redirect_uri", &self.redirect_url);
        url.query_pairs_mut().append_pair("user_scope", "identity.basic,identity.email,identity.avatar");
        if !self.scopes.is_empty() {
            url.query_pairs_mut().append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut().append_pair("state", state);
        }
        url.into()
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self.http_client.post("https://slack.com/api/oauth.v2.access")
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

        let access_token = token_res["authed_user"]["access_token"].as_str().ok_or_else(|| crate::error::SocialiteError::Token("Failed to get authed_user access_token".to_string()))?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"].as_u64().or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }


    async fn get_user_from_token(&self, access_token: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let user_res = self.http_client.get("https://slack.com/api/users.identity")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let user_data = &user_res["user"];

        Ok(SocialiteUser {
            id: user_data["id"].as_str().unwrap_or("").to_string(),
            name: user_data["name"].as_str().unwrap_or("").to_string(),
            email: user_data["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_data["image_512"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }}
