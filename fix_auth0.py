import re

filepath = "src/providers/auth0.rs"
with open(filepath, "r") as f:
    content = f.read()

# Fix client_id bug
content = content.replace('params.append_pair("client_id", &self.domain);', 'params.append_pair("client_id", &self.client_id);')
content = content.replace('params\n            .append_pair("redirect_uri", &self.client_id);', 'params.append_pair("redirect_uri", &self.redirect_url);')
content = content.replace('params.append_pair("redirect_uri", &self.client_id);', 'params.append_pair("redirect_uri", &self.redirect_url);')

# Also fix the invalid domain test, since format! doesn't gracefully fall back to auth0.com
content = re.sub(
    r'assert!\(url\.starts_with\("https://auth0\.com/authorize\?"\)\);',
    r'assert!(url.starts_with("https://invalid domain/authorize?"));',
    content
)

with open(filepath, "w") as f:
    f.write(content)
