use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(GitlabProvider, "read_user");

#[async_trait]
impl Provider for GitlabProvider {
    fn redirect_url(&self) -> Result<String, crate::error::SocialiteError> {
        let mut url = url::Url::parse("https://gitlab.com/oauth/authorize")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        url.query_pairs_mut()
            .append_pair("redirect_uri", &self.redirect_url);
        url.query_pairs_mut().append_pair("response_type", "code");
        if !self.scopes.is_empty() {
            url.query_pairs_mut()
                .append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut().append_pair("state", state);
        }

        if let Some(pkce) = &self.pkce_challenge {
            url.query_pairs_mut().append_pair("code_challenge", pkce);
            url.query_pairs_mut()
                .append_pair("code_challenge_method", "S256");
        }
        Ok(url.into())
    }

    async fn get_user(
        &self,
        auth_code: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self
            .http_client
            .post("https://gitlab.com/oauth/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let access_token = token_res["access_token"].as_str().ok_or_else(|| {
            crate::error::SocialiteError::Token("Failed to get access_token".to_string())
        })?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"]
            .as_u64()
            .or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }

    async fn get_user_from_token(
        &self,
        access_token: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let user_res = self
            .http_client
            .get("https://gitlab.com/api/v4/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["id"].as_i64().unwrap_or(0).to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
