import os
import re

providers_dir = "src/providers"

refresh_template = """    fn token_url(&self) -> String {
        "{url}".to_string()
    }

    async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<SocialiteUser, crate::error::SocialiteError> {
        let token_res = self
            .http_client
            .post(&self.token_url())
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        if let Some(err) = token_res["error"].as_str() {
            let err_desc = token_res["error_description"].as_str().unwrap_or("");
            return Err(crate::error::SocialiteError::Token(format!(
                "Provider returned error: {} - {}",
                err, err_desc
            )));
        }

        let access_token = token_res["access_token"].as_str().ok_or_else(|| {
            crate::error::SocialiteError::Token("Failed to get access_token during refresh".to_string())
        })?;

        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"]
            .as_str()
            .map(|s| s.to_string());
        user.expires_in = token_res["expires_in"]
            .as_u64()
            .or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)
    }
}"""

for filename in os.listdir(providers_dir):
    if not filename.endswith(".rs") or filename in ["mod.rs", "mock.rs"]:
        continue
    
    filepath = os.path.join(providers_dir, filename)
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()
    
    if "fn token_url" in content:
        print(f"Skipping {filename}, already refactored.")
        continue
    
    # Try to find the token URL in get_user
    # Usually it's like: self.http_client.post("https://...")
    match = re.search(r'\.http_client\s*\.\s*post\(\s*"([^"]+)"\s*\)', content)
    if not match:
        print(f"Warning: Could not find token url in {filename}")
        continue
    
    url = match.group(1)
    
    # Replace the hardcoded URL with self.token_url() in get_user
    # Careful not to replace other POST requests like revoke_token if they exist, but normally get_user is the main one.
    # Actually, let's just replace the exact match.
    content = content.replace(f'.post("{url}")', '.post(&self.token_url())')
    
    # Insert token_url and refresh_token at the end of the Provider impl
    # The impl ends with `}`. So we replace the last `}`.
    last_brace_idx = content.rfind("}")
    if last_brace_idx != -1:
        before = content[:last_brace_idx]
        after = content[last_brace_idx+1:]
        
        injection = refresh_template.replace("{url}", url)
        content = before + "\n" + injection + after
        
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Refactored {filename}")
    else:
        print(f"Error finding closing brace in {filename}")
