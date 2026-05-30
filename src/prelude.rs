//! The Rust Socialite Prelude
//!
//! A convenient module to import everything you need to authenticate users via OAuth2.
//!
//! ```rust,ignore
//! use rullst_connect::prelude::*;
//! ```

pub use crate::error::SocialiteError;
pub use crate::provider::Provider;
pub use crate::providers::*;
pub use crate::user::SocialiteUser;

#[cfg(any(feature = "axum", feature = "actix"))]
pub use crate::extractors::AuthCallback;
