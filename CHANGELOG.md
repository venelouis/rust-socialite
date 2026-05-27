# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Tokens returned on User**: `SocialiteUser` now contains `access_token`, `refresh_token`, and `expires_in` fields so you can interact with the provider's API immediately.
- **Frontend/Mobile Integrations**: Added `get_user_from_token(access_token)` to all providers. This allows your backend to securely fetch the user profile when the OAuth flow is handled natively on the frontend (e.g. mobile apps, React, Vue).

### Changed
- All dependencies have been updated to their latest compatible versions.
- Cleaned up compiler warnings related to unused variables across providers.

## [0.4.0] - Previous stable version
- Initial open-source release with 33 OAuth2 providers supported.
- Standardized `SocialiteUser` and async support.
