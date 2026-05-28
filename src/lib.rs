pub mod client;
pub mod error;
#[cfg(any(feature = "axum", feature = "actix"))]
pub mod extractors;
pub mod pkce;
pub mod prelude;
pub mod provider;
pub mod providers;
pub mod user;

#[macro_use]
pub mod macros;

pub use error::SocialiteError;

pub use provider::Provider;
pub use user::SocialiteUser;

/// The main entry point for the Socialite library.
pub struct Socialite;

impl Socialite {
    // In the future, this will act as a factory:
    // pub fn driver(name: &str) -> Box<dyn Provider> { ... }
}
