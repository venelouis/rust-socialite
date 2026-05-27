use crate::error::SocialiteError;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(StripeProvider, "read_write");

#[async_trait]
impl Provider for StripeProvider {
    fn redirect_url(&self) -> String {
        let mut params = url::form_urlencoded::Serializer::new(String::new());
        params.append_pair("response_type", "code");
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
            "https://connect.stripe.com/oauth/authorize?{}",
            params.finish()
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self
            .http_client
            .post("https://connect.stripe.com/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let stripe_user_id = token_res["stripe_user_id"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get stripe_user_id".to_string()))?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        let mut user = self.get_user_from_token(access_token).await?;
        if user.id.is_empty() {
            user.id = stripe_user_id.to_string();
        }
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
        // Fetch account details using the connected account ID (or just /v1/account for the current token owner)
        let user_res = self
            .http_client
            .get("https://api.stripe.com/v1/account")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let name = user_res["business_profile"]["name"]
            .as_str()
            .or_else(|| user_res["settings"]["dashboard"]["display_name"].as_str())
            .unwrap_or("");

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: name.to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: None, // Stripe does not expose an avatar URL via this endpoint
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
