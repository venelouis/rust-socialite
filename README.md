# Rust Socialite 🦀

[![Crates.io](https://img.shields.io/crates/v/rust-socialite.svg)](https://crates.io/crates/rust-socialite)
[![Documentation](https://docs.rs/rust-socialite/badge.svg)](https://docs.rs/rust-socialite)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> 🇧🇷 *Para a versão em Português, veja o [README_pt.md](README_pt.md).*

**Rust Socialite** is an elegant, async-first, and Developer Experience (DX) focused OAuth2 authentication library for Rust, heavily inspired by Laravel Socialite. It simplifies the integration of social logins into your Rust web applications, providing a standardized interface across multiple providers.

## ✨ Features

- 🚀 **Async & Fast**: Built on top of `tokio` and `reqwest`.
- 🧩 **Standardized**: All providers return a unified `SocialiteUser` struct.
- 🛡️ **Type-Safe**: Robust error handling using `thiserror` (`SocialiteError`).
- 🔌 **Framework Agnostic**: Works seamlessly with Axum, Actix, Leptos, Dioxus, or any other framework.

> 📚 **Important Documents:**
> - [CHANGELOG.md](CHANGELOG.md): See what's new in v5.0.0.
> - [ROADMAP.md](ROADMAP.md): Discover our path to v1.0.0.
> - [SECURITY_AUDIT.md](SECURITY_AUDIT.md): Detailed report on the v5.0.0 Enterprise Security Audit.

## 📦 Supported Providers (v5.0.0)

Official support for 33 major providers:

1. **Google**
2. **GitHub**
3. **X (Twitter)** (with PKCE support)
4. **Apple** (Sign in with Apple)
5. **Microsoft / Azure AD**
6. **AWS Cognito**
7. **Auth0**
8. **Okta**
9. **Facebook**
10. **LinkedIn**
11. **Discord**
12. **Spotify**
13. **Twitch**
14. **GitLab**
15. **Bitbucket**
16. **Slack**
17. **Patreon**
18. **Zoom**
19. **Reddit**
20. **Dropbox**
21. **Notion**
22. **Stripe**
23. **DigitalOcean**
24. **TikTok**
25. **Pinterest**
26. **Snapchat**
27. **Instagram**
28. **Line**
29. **VK (VKontakte)**
30. **Yahoo**
31. **Basecamp**
32. **Linear**
33. **Asana**

## 🛠️ Installation

Add the package to your `Cargo.toml`:

```toml
[dependencies]
rust-socialite = "0.4.1"
tokio = { version = "1.52", features = ["full"] }
```

## 🚀 Quick Start

### 1. Initialize the Provider
Choose your provider and pass your credentials and callback URL:

```rust
use rust_socialite::prelude::*;

let github = GithubProvider::new(
    "YOUR_CLIENT_ID".to_string(),
    "YOUR_CLIENT_SECRET".to_string(),
    "http://localhost:3000/auth/github/callback".to_string(),
);
```

### 2. Redirect the User
Get the authorization URL and redirect your user:

```rust
let url = github.redirect_url();
// Example in Axum: return Redirect::temporary(&url);
```

### 3. Handle the Callback & Get User
When the user returns to your callback URL with a `code` query parameter, exchange it for a `SocialiteUser`:

```rust
match github.get_user(code).await {
    Ok(user) => {
        println!("Welcome, {}!", user.name);
        println!("Email: {:?}", user.email);
        println!("Avatar: {:?}", user.avatar_url);
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user".to_string()),
    }
}
```

### 🛡️ CSRF Protection (State Parameter)

To prevent Cross-Site Request Forgery (CSRF) attacks, you should generate a secure random string, save it in a session/cookie, and pass it to the provider.

```rust
// 1. Generate a random state string and save it in the session
let state = "random_secure_string";

// 2. Get the authorization URL with the state parameter using the builder
let url = github.with_state(state).redirect_url();
// return Redirect::temporary(&url);

// 3. In the callback route, verify if the query param `state` matches your session!
```

### 🔒 PKCE Support (e.g. X / Twitter)

Some providers like **X (Twitter) v2** strictly require PKCE (Proof Key for Code Exchange). We provide a built-in helper for this.

```rust
use rust_socialite::pkce::generate_pkce;

// 1. Generate challenge and verifier
let (code_verifier, code_challenge) = generate_pkce();

// 2. Save `code_verifier` in the user's session or a secure HttpOnly cookie!

// 3. Get the URL with PKCE natively using the builder pattern
let auth_url = provider.with_pkce(&code_challenge).redirect_url();

// 4. In the callback route, fetch the user using the saved verifier:
let user = provider.get_user_with_pkce(&code, &code_verifier).await.unwrap();
```

## 🧑‍💻 Full Example with Axum

You can find a complete working server using the **Axum** framework in the examples directory. Just run:

```bash
cargo run --example axum_server
```

## 🤝 Contributing

Feel free to open Issues and submit Pull Requests! Want to add a new provider? It's easy! Just implement the `Provider` trait.

## 📄 License

This project is licensed under the MIT License.
