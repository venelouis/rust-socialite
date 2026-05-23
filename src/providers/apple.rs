use crate::provider::Provider;
use crate::user::SocialiteUser;
use async_trait::async_trait;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::Engine;

pub struct AppleProvider {
    client_id: String,
    team_id: String,
    key_id: String,
    private_key_pem: String,
    redirect_url: String,
    http_client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppleClaims {
    iss: String,
    iat: u64,
    exp: u64,
    aud: String,
    sub: String,
}

impl AppleProvider {
    /// Apple requires a Team ID, a Key ID, and the contents of a .p8 Private Key file
    /// to dynamically generate the client_secret JWT on every login.
    pub fn new(
        client_id: String,
        team_id: String,
        key_id: String,
        private_key_pem: String,
        redirect_url: String,
    ) -> Self {
        Self {
            client_id,
            team_id,
            key_id,
            private_key_pem,
            redirect_url,
            http_client: Client::new(),
        }
    }

    fn generate_client_secret(&self) -> Result<String, crate::error::SocialiteError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let claims = AppleClaims {
            iss: self.team_id.clone(),
            iat: now,
            exp: now + 86400 * 30, // 30 days expiration
            aud: "https://appleid.apple.com".to_string(),
            sub: self.client_id.clone(),
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        let encoding_key = EncodingKey::from_ec_pem(self.private_key_pem.as_bytes())?;
        let token = encode(&header, &claims, &encoding_key)?;

        Ok(token)
    }
}

#[async_trait]
impl Provider for AppleProvider {
    fn redirect_url(&self) -> String {
        format!(
            "https://appleid.apple.com/auth/authorize?client_id={}&redirect_uri={}&response_type=code&response_mode=form_post&scope=name%20email",
            self.client_id, self.redirect_url
        )
    }

    async fn get_user(&self, auth_code: &str) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let client_secret = self.generate_client_secret()?;

        let token_res = self.http_client.post("https://appleid.apple.com/auth/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", auth_code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;

        // Apple returns user data inside an "id_token" (JWT)
        let id_token_str = token_res["id_token"].as_str().ok_or_else(|| crate::error::SocialiteError::Token("Failed to get id_token from Apple".to_string()))?;
        
        // Decode the JWT (unsafe decode is fine here since it comes directly over TLS from Apple)
        let parts: Vec<&str> = id_token_str.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::error::SocialiteError::Provider("Invalid id_token format".to_string()));
        }
        
        let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])?;
        let payload: Value = serde_json::from_slice(&payload_bytes)?;

        // Note: Apple only sends 'name' and 'email' in the FIRST login POST body (form_post), 
        // not in the token response. The id_token only guarantees the sub (id) and email.
        Ok(SocialiteUser {
            id: payload["sub"].as_str().unwrap_or("").to_string(),
            name: String::new(), // Developer needs to extract this from the form_post on first login
            email: payload["email"].as_str().map(|s| s.to_string()),
            avatar_url: None, // Apple does not provide avatars
            raw_data: payload,
        })
    }
}
