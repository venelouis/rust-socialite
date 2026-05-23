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

## 📦 Supported Providers (v0.1.0)

Official support for 12 major providers:

1. **Google**
2. **GitHub**
3. **Apple** (Sign in with Apple)
4. **Microsoft / Azure AD**
5. **Facebook**
6. **LinkedIn**
7. **Discord**
8. **Spotify**
9. **Twitch**
10. **GitLab**
11. **Bitbucket**
12. **Slack**

## 🛠️ Installation

Add the package to your `Cargo.toml`:

```toml
[dependencies]
rust-socialite = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 🚀 Quick Start

### 1. Initialize the Provider
Choose your provider and pass your credentials and callback URL:

```rust
use rust_socialite::providers::github::GithubProvider;
use rust_socialite::Provider;

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
    },
    Err(e) => {
        eprintln!("Authentication failed: {}", e);
    }
}
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
