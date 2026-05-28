use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;
use url::form_urlencoded;

crate::define_provider!(GoogleProvider, "openid", "profile", "email");

#[async_trait]
impl Provider for GoogleProvider {
    fn redirect_url(&self) -> String {
        let mut params = form_urlencoded::Serializer::new(String::with_capacity(256));
        params
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_url)
            .append_pair("response_type", "code")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");
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
            "https://accounts.google.com/o/oauth2/v2/auth?{}",
            params.finish()
        )
    }

    async fn get_user(
        &self,
        auth_code: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        // Exchange code for token
        let token_res = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
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

        let mut user = if let Some(id_token) = token_res["id_token"].as_str() {
            // OIDC FAST PATH: Decode id_token directly without making a second HTTP request!
            let parts: Vec<&str> = id_token.split('.').collect();
            if parts.len() == 3 {
                use base64::Engine;
                if let Ok(payload_bytes) =
                    base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
                {
                    if let Ok(payload) = serde_json::from_slice::<Value>(&payload_bytes) {
                        SocialiteUser {
                            id: payload["sub"].as_str().unwrap_or("").to_string(),
                            name: payload["name"].as_str().unwrap_or("").to_string(),
                            email: payload["email"].as_str().map(|s| s.to_string()),
                            avatar_url: payload["picture"].as_str().map(|s| s.to_string()),
                            raw_data: payload,
                            access_token: access_token.to_string(),
                            refresh_token: None,
                            expires_in: None,
                        }
                    } else {
                        self.get_user_from_token(access_token).await?
                    }
                } else {
                    self.get_user_from_token(access_token).await?
                }
            } else {
                self.get_user_from_token(access_token).await?
            }
        } else {
            self.get_user_from_token(access_token).await?
        };

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
        // Fetch user profile
        let user_res = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["sub"].as_str().unwrap_or("").to_string(),
            name: user_res["name"].as_str().unwrap_or("").to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: user_res["picture"].as_str().map(|s| s.to_string()),
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }

    async fn revoke_token(&self, token: &str) -> Result<(), crate::error::SocialiteError> {
        self.http_client
            .post("https://oauth2.googleapis.com/revoke")
            .form(&[("token", token)])
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
