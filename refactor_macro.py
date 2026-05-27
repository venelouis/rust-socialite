import os
import re

DIR = 'src/providers'

def refactor():
    for filename in os.listdir(DIR):
        if not filename.endswith('.rs') or filename == 'mod.rs':
            continue
        filepath = os.path.join(DIR, filename)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # Extract name and scopes
        struct_match = re.search(r'pub struct (\w+Provider) \{', content)
        if not struct_match:
            continue
        provider_name = struct_match.group(1)

        # Skip providers with custom fields like Apple, Cognito, Auth0, Okta
        custom_fields = ['domain:', 'team_id:', 'domain: String']
        skip = False
        for c in custom_fields:
            if c in content.split('#[async_trait]')[0]:  # Only check struct definition area
                skip = True
        
        # Also skip Apple entirely due to custom key fields
        if provider_name in ['AppleProvider', 'CognitoProvider', 'Auth0Provider', 'OktaProvider']:
            skip = True

        if skip:
            print(f"Skipping {filename} due to custom fields")
            continue

        # Check if already refactored
        if 'define_provider!' in content:
            continue

        # Extract default scopes from pub fn new
        scopes_match = re.search(r'scopes:\s*vec!\[(.*?)\],', content)
        scopes_args = ""
        if scopes_match:
            scopes_str = scopes_match.group(1)
            # "openid".to_string(), "profile".to_string() -> "openid", "profile"
            scopes = re.findall(r'"([^"]+)"', scopes_str)
            if scopes:
                scopes_args = ", " + ", ".join(f'"{s}"' for s in scopes)

        macro_call = f"crate::define_provider!({provider_name}{scopes_args});\n"

        parts = re.split(r'#\[async_trait\]', content, maxsplit=1)
        if len(parts) != 2:
            continue
        
        header = parts[0]
        # Remove the struct and impl from header
        header = re.sub(r'pub struct \w+Provider \{.*?\n\}\n*', '', header, flags=re.DOTALL)
        header = re.sub(r'impl \w+Provider \{.*?\n\}\n*', '', header, flags=re.DOTALL)

        new_content = header.strip() + "\n\n" + macro_call + "\n#[async_trait]\n" + parts[1].strip() + "\n"

        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"Refactored {filename} to use macro")

if __name__ == '__main__':
    refactor()
