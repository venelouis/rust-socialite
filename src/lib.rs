pub mod error;
pub mod provider;
pub mod user;
pub mod providers;

pub use error::SocialiteError;

pub use provider::Provider;
pub use user::SocialiteUser;

/// The main entry point for the Socialite library.
pub struct Socialite;

impl Socialite {
    // In the future, this will act as a factory:
    // pub fn driver(name: &str) -> Box<dyn Provider> { ... }
}
