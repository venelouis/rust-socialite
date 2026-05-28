import os
import re

files_with_conflicts = [
    "src/providers/auth0.rs",
    "src/providers/bitbucket.rs",
    "src/providers/cognito.rs",
    "src/providers/github.rs",
    "src/providers/linkedin.rs",
    "src/providers/okta.rs",
    "src/providers/pinterest.rs",
    "src/providers/reddit.rs",
    "src/providers/snapchat.rs",
    "src/providers/stripe.rs",
    "src/providers/tiktok.rs",
    "src/providers/vk.rs",
    "src/providers/x.rs",
    "src/providers/yahoo.rs",
]

for filepath in files_with_conflicts:
    if os.path.exists(filepath):
        with open(filepath, "r") as f:
            content = f.read()

        # We want to keep HEAD which has our param finish change.
        # But wait, looking at origin/main, someone changed url parsing to handle Ok(u) => u, Err(_) => return String::new().
        # But our params formulation does not need Ok or Err.

        # Let's extract the part from HEAD
        pattern = r"<<<<<<< HEAD(.*?)=======(.*?)>>>>>>> origin/main"

        def replace_func(match):
            return match.group(1).strip() + "\n"

        new_content = re.sub(pattern, replace_func, content, flags=re.DOTALL)

        with open(filepath, "w") as f:
            f.write(new_content)
        print(f"Resolved {filepath}")
