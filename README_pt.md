# Rust Socialite 🦀

[![Crates.io](https://img.shields.io/crates/v/rust-socialite.svg)](https://crates.io/crates/rust-socialite)
[![Documentation](https://docs.rs/rust-socialite/badge.svg)](https://docs.rs/rust-socialite)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Rust Socialite** é uma biblioteca de autenticação OAuth2 elegante, assíncrona e focada em Developer Experience (DX), inspirada no Laravel Socialite. Ela facilita a integração de logins sociais na sua aplicação web Rust, suportando múltiplos provedores de forma padronizada.

## ✨ Características

- 🚀 **Assíncrona e Rápida**: Construída em cima de `tokio` e `reqwest`.
- 🧩 **Padronizada**: Todos os provedores retornam uma struct única `SocialiteUser`.
- 🛡️ **Tipagem Segura**: Tratamento de erros robusto com `thiserror` (`SocialiteError`).
- 🔌 **Agnóstica a Framework**: Funciona com Axum, Actix, Leptos, Dioxus, ou qualquer outro framework.

## 📦 Provedores Suportados (v0.1.0)

Já oferecemos suporte oficial para 12 dos maiores provedores do mercado:

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

## 🛠️ Instalação

Adicione o pacote ao seu `Cargo.toml`:

```toml
[dependencies]
rust-socialite = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 🚀 Como Usar

### 1. Inicialize o Provedor
Escolha o provedor e passe suas credenciais e URL de callback:

```rust
use rust_socialite::providers::github::GithubProvider;
use rust_socialite::Provider;

let github = GithubProvider::new(
    "SEU_CLIENT_ID".to_string(),
    "SEU_CLIENT_SECRET".to_string(),
    "http://localhost:3000/auth/github/callback".to_string(),
);
```

### 2. Redirecione o Usuário
Pegue a URL de autorização e mande o usuário para lá:

```rust
let url = github.redirect_url();
// Exemplo em Axum: return Redirect::temporary(&url);
```

### 3. Receba o Callback e Pegue o Usuário
Quando o usuário voltar para a sua URL com o código na query string (`?code=...`), basta trocá-lo pelo `SocialiteUser`:

```rust
match github.get_user(code).await {
    Ok(user) => {
        println!("Bem-vindo, {}!", user.name);
        println!("Email: {:?}", user.email);
        println!("Avatar: {:?}", user.avatar_url);
    },
    Err(e) => {
        eprintln!("Falha na autenticação: {}", e);
    }
}
```

## 🧑‍💻 Exemplo Completo com Axum

Você pode ver um servidor completo usando o framework **Axum** no nosso repositório de exemplos. Basta rodar:

```bash
cargo run --example axum_server
```

## 🤝 Contribuindo

Sinta-se à vontade para abrir Issues e enviar Pull Requests! Quer adicionar um novo provedor? É muito fácil! Basta implementar a trait `Provider`.

## 📄 Licença

Este projeto está licenciado sob a licença MIT.
