import os
import re

files_with_warnings = [
    "src/providers/github.rs",
    "src/providers/reddit.rs",
    "src/providers/snapchat.rs",
]

for filepath in files_with_warnings:
    if os.path.exists(filepath):
        with open(filepath, "r") as f:
            content = f.read()

        content = re.sub(r'use url::form_urlencoded;\n?', '', content)

        with open(filepath, "w") as f:
            f.write(content)
        print(f"Fixed {filepath}")
