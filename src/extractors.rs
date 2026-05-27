use serde::Deserialize;

/// Standard OAuth2 callback query parameters.
/// 
/// Most web frameworks (like Axum, Actix, Leptos, Rocket) can automatically
/// deserialize URL query strings into this struct.
/// 
/// # Example (Axum)
/// ```rust,ignore
/// async fn auth_callback(Query(params): Query<AuthCallback>) -> impl IntoResponse {
///     if let Some(error) = params.error {
///         return format!("Auth failed: {}", error);
///     }
///     
///     let code = params.code.unwrap();
///     // Handle token exchange...
/// }
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct AuthCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}
