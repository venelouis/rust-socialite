# 🗺️ rust-socialite Roadmap

Welcome to the official roadmap for `rust-socialite`! The library currently supports 33 providers, dynamic token parsing, `get_user_from_token` extraction, CSRF protection (via `state`), and `scopes` customization.

For our journey towards the **`v1.0.0`** release (Enterprise Level), we have planned the following major additions which are actively being worked on:

## ⏳ In Progress (Phases 3 & 4)

- [x] **Boilerplate Reduction Macros:** Use `define_provider!` to drastically cut down repetitive code in provider structs, making it easier for the community to contribute new providers.
- [x] **Native Framework Integration:** Create the optional `axum` and `actix` features, providing Extractors (like `AuthCallback`) so URL parsing of codes, states, and errors works magically.
- [x] **Token Revocation (Logout):** Add the `revoke_token` method to the trait to allow direct logout on the supported providers' servers.
- [x] **Mocking Tools (TDD):** A `MockProvider` to facilitate writing unit tests for end users of the library.
- [x] **OIDC Support:** Automatic and ultra-fast `id_token` validation for Google and Apple using JWT (without extra HTTP calls).
- [x] **Security Audit & PKCE:** Implement strict URL parsing, eliminate panics, and provide native `.with_pkce()` support across all providers.

## 🔮 Not So Distant Future

- **HTTP Client Agnostic:** Generic client support (via `HttpClient` trait) allowing the use of `surf`, `reqwest-middleware`, or others instead of forcing `reqwest`.
- **Database Integration (rust-eloquent, SQLx, Diesel):** Helper traits (e.g., `IntoDatabaseUser`) to seamlessly save the user into the database, with special focus on `rust-eloquent` to keep the Laravel ecosystem vibe in Rust!
- **HTTP Proxy Support:** Allow the configuration of corporate proxies for locked-down environments.
- [x] **Refresh Token Module:** Automated `refresh_token` support in case the primary token expires. (Done in v5.1.0)
- **Universal Avatar Standardization:** Advanced parsing to guarantee optimal resolutions for returned profile pictures.
- **Leptos & Dioxus Integration:** Extractors for Fullstack / WebAssembly Rust frameworks.
- **Integration Tests with Mock Servers (`wiremock`):** Cover the real HTTP flow to guarantee that the parser correctly handles incomplete responses, expired tokens, or network failures.
- **Rate Limiting & Advanced Retry Policies:** Offer integrated wrappers (e.g., via `reqwest-middleware` and `reqwest-retry`) to perform native exponential backoff when providers reject requests due to rate limits (HTTP 429).
- **Unified Provider Error Extraction:** Map error responses from providers (like "invalid_grant") into structured enums within `SocialiteError` to drastically improve debugging experience.

---

Want to help implement any of these features? Feel free to open a PR!
