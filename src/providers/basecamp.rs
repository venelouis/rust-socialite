use crate::error::SocialiteError;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(BasecampProvider);

#[async_trait]
impl Provider for BasecampProvider {
    fn redirect_url(&self) -> String {
        let mut params = url::form_urlencoded::Serializer::new(String::with_capacity(256));
        params.append_pair("type", "web_server");
        params
            .append_pair("client_id", &self.client_id);
        params
            .append_pair("redirect_uri", &self.redirect_url);
        if !self.scopes.is_empty() {
            params
                .append_pair("scope", &self.scopes.join(" "));
        }
        if let Some(state) = &self.state {
            params.append_pair("state", state);
        }

        if let Some(pkce) = &self.pkce_challenge {
            params.append_pair("code_challenge", pkce);
            params
                .append_pair("code_challenge_method", "S256");
        }
        format!("https://launchpad.37signals.com/authorization/new?{}", params.finish())
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self
            .http_client
            .post("https://launchpad.37signals.com/authorization/token?type=web_server")
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

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

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
    ) -> Result<SocialiteUser, SocialiteError> {
        let user_res = self
            .http_client
            .get("https://launchpad.37signals.com/authorization.json")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let identity = &user_res["identity"];
        let first_name = identity["first_name"].as_str().unwrap_or("");
        let last_name = identity["last_name"].as_str().unwrap_or("");
        let name = format!("{} {}", first_name, last_name).trim().to_string();

        Ok(SocialiteUser {
            id: identity["id"]
                .as_i64()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "".to_string()),
            name,
            email: identity["email_address"].as_str().map(|s| s.to_string()),
            avatar_url: None, // Basecamp API doesn't standardly expose an avatar via launchpad
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
