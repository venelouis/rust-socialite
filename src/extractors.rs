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

impl AuthCallback {
    /// Helper to verify the CSRF state parameter.
    pub fn verify_state(&self, session_state: &str) -> Result<(), crate::error::SocialiteError> {
        match &self.state {
            Some(state) if state == session_state => Ok(()),
            Some(_) => Err(crate::error::SocialiteError::InvalidState(
                "CSRF state mismatch".into(),
            )),
            None => Err(crate::error::SocialiteError::InvalidState(
                "State missing in callback".into(),
            )),
        }
    }
}

#[cfg(feature = "axum")]
impl<S> axum::extract::FromRequestParts<S> for AuthCallback
where
    S: Send + Sync,
{
    type Rejection = axum::extract::rejection::QueryRejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(callback) =
            axum::extract::Query::<AuthCallback>::from_request_parts(parts, state).await?;
        Ok(callback)
    }
}

#[cfg(feature = "actix")]
impl actix_web::FromRequest for AuthCallback {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match actix_web::web::Query::<AuthCallback>::from_query(req.query_string()) {
            Ok(query) => std::future::ready(Ok(query.into_inner())),
            Err(e) => std::future::ready(Err(e.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_callback_success_deserialization() {
        let query = "code=auth_code_123&state=state_xyz";
        let callback: AuthCallback = serde_urlencoded::from_str(query).unwrap();

        assert_eq!(callback.code.as_deref(), Some("auth_code_123"));
        assert_eq!(callback.state.as_deref(), Some("state_xyz"));
        assert_eq!(callback.error, None);
        assert_eq!(callback.error_description, None);
    }

    #[test]
    fn test_auth_callback_error_deserialization() {
        let query = "error=access_denied&error_description=User%20denied%20access&state=state_xyz";
        let callback: AuthCallback = serde_urlencoded::from_str(query).unwrap();

        assert_eq!(callback.code, None);
        assert_eq!(callback.state.as_deref(), Some("state_xyz"));
        assert_eq!(callback.error.as_deref(), Some("access_denied"));
        assert_eq!(
            callback.error_description.as_deref(),
            Some("User denied access")
        );
    }

    #[test]
    fn test_auth_callback_empty_deserialization() {
        let query = "";
        let callback: AuthCallback = serde_urlencoded::from_str(query).unwrap();

        assert_eq!(callback.code, None);
        assert_eq!(callback.state, None);
        assert_eq!(callback.error, None);
        assert_eq!(callback.error_description, None);
    }
}
