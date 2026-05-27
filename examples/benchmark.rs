use async_trait::async_trait;
use rust_socialite::{Provider, SocialiteUser, SocialiteError};
use std::time::Instant;

struct MockProvider;

#[async_trait]
impl Provider for MockProvider {
    fn redirect_url(&self) -> String {
        // Simulate an expensive redirect_url call
        std::thread::sleep(std::time::Duration::from_millis(10));
        "https://example.com/oauth".to_string()
    }

    async fn get_user(&self, _auth_code: &str) -> Result<SocialiteUser, SocialiteError> {
        unimplemented!()
    }

    async fn get_user_from_token(&self, _access_token: &str) -> Result<SocialiteUser, SocialiteError> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() {
    let provider = MockProvider;
    let iterations = 50;

    println!("Starting benchmark with {} iterations...", iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = provider.redirect_url_with_state("my-state");
    }
    let duration_state = start.elapsed();
    println!("redirect_url_with_state: {:?}", duration_state);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = provider.redirect_url_with_pkce("my-challenge");
    }
    let duration_pkce = start.elapsed();
    println!("redirect_url_with_pkce: {:?}", duration_pkce);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = provider.redirect_url_with_pkce_and_state("my-challenge", "my-state");
    }
    let duration_pkce_state = start.elapsed();
    println!("redirect_url_with_pkce_and_state: {:?}", duration_pkce_state);

    println!("Total time: {:?}", duration_state + duration_pkce + duration_pkce_state);
}
