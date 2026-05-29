import os

providers_dir = "src/providers"

for filename in os.listdir(providers_dir):
    if not filename.endswith(".rs"):
        continue
    
    filepath = os.path.join(providers_dir, filename)
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()
    
    if ".post(&self.token_url())" in content:
        content = content.replace(".post(&self.token_url())", ".post(self.token_url())")
        
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Fixed clippy in {filename}")
