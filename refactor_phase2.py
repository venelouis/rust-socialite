import os
import re
import urllib.parse

DIR = 'src/providers'

def refactor():
    for filename in os.listdir(DIR):
        if not filename.endswith('.rs') or filename == 'mod.rs':
            continue
        filepath = os.path.join(DIR, filename)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        if 'pub fn with_scopes' in content:
            continue
            
        # 1. Add scopes and state to the struct definition
        # Find struct ProviderName { ... }
        struct_match = re.search(r'(pub struct \w+ \{.*?)(^\})', content, re.MULTILINE | re.DOTALL)
        if not struct_match:
            print(f"Failed to find struct in {filename}")
            continue
            
        struct_def = struct_match.group(1)
        # Ensure it doesn't already have scopes
        if 'scopes: Vec<String>' not in struct_def:
            new_struct_def = struct_def + "    scopes: Vec<String>,\n    state: Option<String>,\n}"
            content = content.replace(struct_match.group(0), new_struct_def)

        # 2. Extract redirect_url logic and default scope
        # Find fn redirect_url(&self) -> String { format!("...", self.client_id, self.redirect_url) }
        redirect_match = re.search(r'fn redirect_url\(&self\) -> String \{\s*format!\(\s*"(.*?)",\s*(.*?)\s*\)\s*\}', content, re.DOTALL)
        
        default_scope_list = []
        if redirect_match:
            url_str = redirect_match.group(1)
            args_str = redirect_match.group(2)
            
            # Extract arguments
            args = [a.strip() for a in args_str.split(',') if a.strip()]
            
            # Split url_str by '?'
            if '?' in url_str:
                base_url, query_str = url_str.split('?', 1)
                query_params = query_str.split('&')
                
                url_builder_lines = [f'let mut url = url::Url::parse("{base_url}").unwrap();']
                
                arg_idx = 0
                for param in query_params:
                    if not param: continue
                    if '=' in param:
                        key, val = param.split('=', 1)
                    else:
                        key, val = param, ""
                        
                    if key == 'scope':
                        # This is the default scope!
                        # Decode %20 and split by space or comma
                        decoded = urllib.parse.unquote(val)
                        if ',' in decoded:
                            default_scope_list = [s.strip() for s in decoded.split(',') if s.strip()]
                        else:
                            default_scope_list = [s.strip() for s in decoded.split(' ') if s.strip()]
                        continue # Don't append scope here, we will do it dynamically
                        
                    # If val has {}, replace with the arg
                    if '{}' in val:
                        # Find which arg it maps to
                        if arg_idx < len(args):
                            mapped_arg = args[arg_idx]
                            arg_idx += 1
                            url_builder_lines.append(f'url.query_pairs_mut().append_pair("{key}", &{mapped_arg});')
                        else:
                            print(f"Arg mismatch in {filename}")
                    else:
                        url_builder_lines.append(f'url.query_pairs_mut().append_pair("{key}", "{val}");')
                
                # Dynamic appends
                url_builder_lines.append('if !self.scopes.is_empty() {')
                url_builder_lines.append('    url.query_pairs_mut().append_pair("scope", &self.scopes.join(" "));')
                url_builder_lines.append('}')
                url_builder_lines.append('if let Some(state) = &self.state {')
                url_builder_lines.append('    url.query_pairs_mut().append_pair("state", state);')
                url_builder_lines.append('}')
                url_builder_lines.append('url.into()')
                
                new_redirect_body = "\n        ".join(url_builder_lines)
                
                new_redirect_fn = f"fn redirect_url(&self) -> String {{\n        {new_redirect_body}\n    }}"
                
                content = content.replace(redirect_match.group(0), new_redirect_fn)
            else:
                print(f"No query params in {filename}")
        else:
            print(f"Could not find redirect format in {filename}")

        # 3. Update Provider::new to initialize scopes and state
        new_match = re.search(r'(pub fn new\(.*?\) -> Self \{\s*Self \{)(.*?)(\s*\})', content, re.DOTALL)
        if new_match:
            init_start = new_match.group(1)
            init_body = new_match.group(2)
            init_end = new_match.group(3)
            
            scope_init_str = "vec![" + ", ".join(f'"{s}".to_string()' for s in default_scope_list) + "]"
            
            if 'scopes:' not in init_body:
                new_init_body = init_body
                if not new_init_body.strip().endswith(','):
                    new_init_body += ","
                new_init_body += f"\n            scopes: {scope_init_str},\n            state: None,"
                
                content = content.replace(new_match.group(0), init_start + new_init_body + init_end)
        
        # 4. Add Builder Methods with_scopes and with_state
        # Find where the impl block ends
        impl_match = re.search(r'(impl \w+Provider \{\s*pub fn new.*?)(^\})', content, re.MULTILINE | re.DOTALL)
        if impl_match:
            builder_methods = """
    /// Overrides the default scopes for this provider.
    pub fn with_scopes(mut self, scopes: &[&str]) -> Self {
        self.scopes = scopes.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Sets the state parameter for CSRF protection.
    pub fn with_state(mut self, state: &str) -> Self {
        self.state = Some(state.to_string());
        self
    }
"""
            content = content.replace(impl_match.group(0), impl_match.group(1) + builder_methods + "}")

        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print(f"Refactored Phase 2 for {filename}")

if __name__ == "__main__":
    refactor()
