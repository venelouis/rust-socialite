import os
import re

DIR = 'src/providers'

for filename in os.listdir(DIR):
    if not filename.endswith('.rs') or filename == 'mod.rs':
        continue
    filepath = os.path.join(DIR, filename)
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Check if already refactored
    if 'async fn get_user_from_token' in content:
        continue
    
    # We want to find the get_user function.
    match = re.search(r'(    async fn get_user\(&self, auth_code: &str\) -> Result<SocialiteUser, (.*?SocialiteError)> \{\n)(.*?)(\n    \}\n)', content, re.DOTALL)
    if not match:
        print(f"Could not find get_user in {filename}")
        continue
        
    sig = match.group(1)
    error_type = match.group(2)
    body = match.group(3)
    end = match.group(4)
    
    # Find the access_token variable extraction
    # It usually ends with `?;`
    # Let's search for `let access_token = ...?;`
    split_match = re.search(r'(\s+let access_token = .*?\?;)\n', body, re.DOTALL)
    if not split_match:
        # Fallback: some might not have `?` if they unwrap, but let's see
        print(f"Could not find access_token split in {filename}")
        continue
        
    part1 = body[:split_match.end()]
    part2 = body[split_match.end():]
    
    # In part2, we need to modify the Ok(SocialiteUser { ... }) to include the tokens
    # We replace `raw_data: user_res,\n        })` with `raw_data: user_res,\n            access_token: access_token.to_string(),\n            refresh_token: None,\n            expires_in: None,\n        })`
    # Note: `raw_data: .*?,` might vary (sometimes it's just user_res, sometimes other things).
    part2 = re.sub(
        r'(raw_data:\s*.*?)(,\n\s*\})',
        r'\1,\n            access_token: access_token.to_string(),\n            refresh_token: None,\n            expires_in: None\2',
        part2
    )
    
    # Now build the new get_user body
    new_get_user_body = part1 + f"""
        let mut user = self.get_user_from_token(access_token).await?;
        user.refresh_token = token_res["refresh_token"].as_str().map(|s| s.to_string());
        user.expires_in = token_res["expires_in"].as_u64().or_else(|| token_res["expires_in"].as_i64().map(|v| v as u64));
        Ok(user)"""
    
    # Now build get_user_from_token
    new_get_user_from_token = f"""
    async fn get_user_from_token(&self, access_token: &str) -> Result<SocialiteUser, {error_type}> {{{part2}
    }}"""
    
    new_content = content[:match.start()] + sig + new_get_user_body + end + "\n" + new_get_user_from_token + content[match.end():]
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(new_content)
    
    print(f"Refactored {filename}")
