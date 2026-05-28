use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;
use url::form_urlencoded;

crate::define_provider!(GithubProvider, "user:email");

#[async_trait]
impl Provider for GithubProvider {
    fn redirect_url(&self) -> Result<String, crate::error::SocialiteError> {
        let mut params = form_urlencoded::Serializer::new(String::new());
        params.append_pair("client_id", &self.client_id);
        params.append_pair("redirect_uri", &self.redirect_url);

        if !self.scopes.is_empty() {
            params.append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            params.append_pair("state", state);
        }

        if let Some(pkce) = &self.pkce_challenge {
            params.append_pair("code_challenge", pkce);
            params.append_pair("code_challenge_method", "S256");
        }

        Ok(format!(
            "https://github.com/login/oauth/authorize?{}",
            params.finish()
        ))
    }

    async fn get_user(
        &self,
        auth_code: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        // 1. Exchange authorization code for access token
        let token_res = self
            .http_client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        if let Some(err) = token_res["error"].as_str() {
            let err_desc = token_res["error_description"].as_str().unwrap_or("");
            return Err(crate::error::SocialiteError::Token(format!(
                "Provider returned error: {} - {}",
                err, err_desc
            )));
        }

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
        // 2. Fetch user profile
        let user_res = self
            .http_client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "rust-socialite") // GitHub API requires User-Agent
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        // 3. Map to generic SocialiteUser
        Ok(SocialiteUser {
            id: user_res["id"].as_i64().unwrap_or(0).to_string(),
            name: user_res["name"]
                .as_str()
                .unwrap_or(user_res["login"].as_str().unwrap_or(""))
                .to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["avatar_url"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
