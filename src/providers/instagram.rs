use crate::error::SocialiteError;
use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use serde_json::Value;

crate::define_provider!(InstagramProvider, "user_profile");

#[async_trait]
impl Provider for InstagramProvider {
    fn redirect_url(&self) -> String {
        let mut url = url::Url::parse("https://api.instagram.com/oauth/authorize").expect("Invalid authorization URL");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id);
        url.query_pairs_mut()
            .append_pair("redirect_uri", &self.redirect_url);
        url.query_pairs_mut().append_pair("response_type", "code");
        crate::utils::append_auth_params(
            &mut url.query_pairs_mut(),
            &self.scopes,
            &self.state,
            &self.pkce_challenge,
        );

        url.into()
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self
            .http_client
            .post("https://api.instagram.com/oauth/access_token")
            .form(&[
                ("grant_type", "authorization_code"),
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
            .get(format!(
                "https://graph.instagram.com/me?fields=id,username&access_token={}",
                access_token
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or("").to_string(),
            name: user_res["username"].as_str().unwrap_or("").to_string(),
            email: None,
            avatar_url: None, // Instagram Basic Display does not provide profile picture
            raw_data: user_res,
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_in: None,
        })
    }
}
