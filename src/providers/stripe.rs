use crate::provider::Provider;
use crate::user::SocialiteUser;
use crate::error::SocialiteError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct StripeProvider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    http_client: Client,
}

impl StripeProvider {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for StripeProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://connect.stripe.com/oauth/authorize?response_type=code&client_id={}&scope=read_write",
            self.client_id
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        let token_res = self.http_client.post("https://connect.stripe.com/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_secret", self.client_secret.as_str()),
                ("code", auth_code),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;

        let stripe_user_id = token_res["stripe_user_id"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get stripe_user_id".to_string()))?;

        let access_token = token_res["access_token"]
            .as_str()
            .ok_or_else(|| SocialiteError::Token("Failed to get access_token".to_string()))?;

        // Fetch account details using the connected account ID
        let user_res = self.http_client.get(format!("https://api.stripe.com/v1/accounts/{}", stripe_user_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let name = user_res["business_profile"]["name"].as_str()
            .or_else(|| user_res["settings"]["dashboard"]["display_name"].as_str())
            .unwrap_or("");

        Ok(SocialiteUser {
            id: user_res["id"].as_str().unwrap_or(stripe_user_id).to_string(),
            name: name.to_string(),
            email: user_res["email"].as_str().map(|s| s.to_string()),
            avatar_url: None, // Stripe does not expose an avatar URL via this endpoint
            raw_data: user_res,
        })
    }
}
