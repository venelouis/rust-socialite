/// Defines a standard OAuth2 provider struct and its builder methods.
/// 
/// This macro generates the boilerplate struct definition, the `new` constructor, 
/// and the `with_scopes` / `with_state` builder methods.
#[macro_export]
macro_rules! define_provider {
    ($name:ident) => {
        $crate::define_provider!($name, );
    };
    ($name:ident, $($default_scope:expr),*) => {
        pub struct $name {
            client_id: String,
            client_secret: String,
            redirect_url: String,
            http_client: reqwest::Client,
            scopes: Vec<String>,
            state: Option<String>,
        }

        impl $name {
            pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
                Self {
                    client_id,
                    client_secret,
                    redirect_url,
                    http_client: reqwest::Client::new(),
                    scopes: vec![$($default_scope.to_string()),*],
                    state: None,
                }
            }

            /// Overrides the default scopes for this provider.
            pub fn with_scopes(mut self, scopes: &[&str]) -> Self {
                self.scopes = scopes.iter().map(|s| s.to_string()).collect();
                self
            }

            /// Sets the state parameter for CSRF protection.
            pub fn with_state(mut self, state: &str) -> Self {
                self.state = Some(state.to_string());
                self
            }
        }
    };
}
