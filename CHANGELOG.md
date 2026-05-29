# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [5.1.0] - 2026-05-28

### Added
- **Automatic CSRF Validation**: The `AuthCallback` extractor now includes a `verify_state(&self, session_state: &str)` method to automatically secure OAuth flows against CSRF attacks.
- **Refresh Token Support**: Added `token_url()` and `refresh_token(token: &str)` methods to the `Provider` trait and implemented them across all 35 providers, allowing developers to automatically renew expired tokens natively.

### Fixed
- **Testing Dependencies**: Added `serde_urlencoded` to `dev-dependencies` to fix a major compilation bug running the test suite on `main`.

### Security
- Maintained zero `unsafe` footprint while ensuring standard parameter passing for URL token generation and token revocation across providers.

## [5.0.2] - 2026-05-27

### Performance
- **Optimized String allocations**: Replaced `String::new()` with `String::with_capacity(256)` in all 33 providers' `redirect_url` methods, reducing unnecessary reallocations and improving performance.

### Developer Experience
- **Fixed README example**: Corrected compilation bug in the README.md code example (line 103) where the `Err` branch was incorrectly placed inside the `Ok` block.

### Maintenance
- **Removed dead code**: Deleted `src/utils.rs` file which was no longer used after the PR-29 refactoring that inlined URL parameter serialization logic directly into providers.

### Internal
- **Code cleanup**: Removed unused module imports and references to the deleted `utils.rs` module from `lib.rs`.

### Compatibility
- **Breaking Changes**: None
- **API Changes**: None
- **Migration Guide**: No migration required - fully backward compatible with v5.0.1

## [5.0.1] - 2026-05-27

### Added
- **Tokens returned on User**: `SocialiteUser` now contains `access_token`, `refresh_token`, and `expires_in` fields so you can interact with the provider's API immediately.
- **Frontend/Mobile Integrations**: Added `get_user_from_token(access_token)` to all providers. This allows your backend to securely fetch the user profile when the OAuth flow is handled natively on the frontend (e.g. mobile apps, React, Vue).
- **Framework Integrations (Axum & Actix)**: Added `axum` and `actix-web` optional features in `Cargo.toml`. Provides native extractors (`AuthCallback`) for seamless URL parsing in route handlers.
- **Token Revocation**: New `revoke_token` method on the `Provider` trait for direct logout at the provider level (reference implementation added for Google).
- **Mocking Tools (TDD)**: Added `MockProvider` to the library to facilitate offline unit testing.
- **Continuous Integration**: Added GitHub Actions (Publish to Crates.io) to automate new version deployments via Tags.
- **Native OIDC Support**: OIDC providers (like Google and Apple) now feature a "Fast Path" that decodes the `id_token` directly via base64, extracting name, email, and photo instantly without making a secondary HTTP request! A massive performance boost.
- **PKCE Support (Proof Key for Code Exchange)**: All providers now have native support for modern PKCE security via the `.with_pkce(code_challenge)` builder method.
- **Prelude Module (`rust_socialite::prelude::*`)**: Added a prelude module for unified imports (ideal for developers and AI assistants).

### Changed
- **Architectural Macros**: All providers now use the internal `define_provider!` macro, which centralizes constructors, state, PKCE, scopes, and reduces hundreds of lines of boilerplate.
- All dependencies have been updated to their latest compatible versions.
- Cleaned up compiler warnings related to unused variables across providers.

## [0.4.0] - Previous stable version
- Initial open-source release with 33 OAuth2 providers supported.
- Standardized `SocialiteUser` and async support.
