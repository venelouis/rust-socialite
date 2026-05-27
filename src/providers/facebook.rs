use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(FacebookProvider, "email", "public_profile");

#[async_trait]
impl Provider for FacebookProvider {
    fn redirect_url(&self) -> String {
        let mut params = url::form_urlencoded::Serializer::new(String::new());
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
        format!(
            "https://www.facebook.com/v19.0/dialog/oauth?{}",
            params.finish()
        )
    }

    async fn get_user(
        &self,
        auth_code: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self
            .http_client
            .post("https://graph.facebook.com/v19.0/oauth/access_token")
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
        let user_res = self.http_client.get(format!(
            "https://graph.facebook.com/v19.0/me?fields=id,name,email,picture.width(500).height(500)&access_token={}",
            access_token
        ))
            .send().await?.error_for_status()?
            .json::<Value>()
            .await?;

        let avatar = user_res["picture"]["data"]["url"]
            .as_str()
            .map(|s| s.to_string());

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: avatar,
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
