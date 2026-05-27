import os

directory = 'src/providers'

for filename in os.listdir(directory):
    if filename.endswith(".rs") and filename not in ["mod.rs", "mock.rs"]:
        filepath = os.path.join(directory, filename)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            
        pkce_url = """
        if let Some(pkce) = &self.pkce_challenge {
            url.query_pairs_mut().append_pair("code_challenge", pkce);
            url.query_pairs_mut().append_pair("code_challenge_method", "S256");
        }"""
        
        pkce_params = """
        if let Some(pkce) = &self.pkce_challenge {
            params.append_pair("code_challenge", pkce);
            params.append_pair("code_challenge_method", "S256");
        }"""
        
        if "url.query_pairs_mut().append_pair(\"state\", state);\n        }" in content:
            content = content.replace(
                "url.query_pairs_mut().append_pair(\"state\", state);\n        }",
                "url.query_pairs_mut().append_pair(\"state\", state);\n        }\n" + pkce_url
            )
        elif "params.append_pair(\"state\", state);\n        }" in content:
            content = content.replace(
                "params.append_pair(\"state\", state);\n        }",
                "params.append_pair(\"state\", state);\n        }\n" + pkce_params
            )
            
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
