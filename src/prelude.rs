//! The Rullst Connect Prelude
//!
//! A convenient module to import everything you need to authenticate users via OAuth2.
//!
//! ```rust,ignore
//! use rullst_connect::prelude::*;
//! ```

pub use crate::error::ConnectError;
pub use crate::provider::Provider;
pub use crate::providers::*;
pub use crate::user::ConnectUser;

#[cfg(any(feature = "axum", feature = "actix"))]
pub use crate::extractors::AuthCallback;
