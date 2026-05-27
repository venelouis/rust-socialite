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

## 📦 Provedores Suportados (v5.0.0)

Já oferecemos suporte oficial para incríveis 33 dos maiores provedores do mercado:

1. **Google**
2. **GitHub**
3. **X (Twitter)** (com suporte a PKCE)
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

## 🛠️ Instalação

Adicione o pacote ao seu `Cargo.toml`:

```toml
[dependencies]
rust-socialite = "5.0.0"
tokio = { version = "1.52", features = ["full"] }
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
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao autenticar usuário".to_string()),
    }
}
```

### 🛡️ Proteção contra CSRF (Parâmetro State)

Para prevenir ataques de falsificação de solicitações (CSRF), você deve gerar uma string aleatória segura, salvá-la em uma sessão/cookie e passá-la para o provedor.

```rust
// 1. Gere uma string aleatória para o state e salve na sessão
let state = "string_aleatoria_segura";

// 2. Pegue a URL de autorização com o parâmetro state
let url = github.redirect_url_with_state(state);
// return Redirect::temporary(&url);

// 3. Na rota de callback, verifique se o `state` recebido na query string é igual ao da sua sessão!
```

### 🔒 Suporte a PKCE (ex: X / Twitter)

Alguns provedores como o **X (Twitter) v2** exigem estritamente o PKCE (Proof Key for Code Exchange). Fornecemos um ajudante nativo para isso.

```rust
use rust_socialite::pkce::generate_pkce;

// 1. Gere o challenge e o verifier
let (code_verifier, code_challenge) = generate_pkce();

// 2. Salve o `code_verifier` na sessão do usuário ou num cookie HttpOnly!

// 3. Pegue a URL de redirecionamento já com o PKCE
let auth_url = provider.redirect_url_with_pkce(&code_challenge);

// 4. Na rota de callback, busque o usuário usando o verifier salvo:
let user = provider.get_user_with_pkce(&code, &code_verifier).await.unwrap();
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
