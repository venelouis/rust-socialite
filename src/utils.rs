use url::form_urlencoded;

pub fn append_auth_params<T: form_urlencoded::Target>(
    params: &mut form_urlencoded::Serializer<'_, T>,
    scopes: &[String],
    state: &Option<String>,
    pkce_challenge: &Option<String>,
) {
    if !scopes.is_empty() {
        params.append_pair("scope", &scopes.join(" "));
    }
    if let Some(state) = state {
        params.append_pair("state", state);
    }
    if let Some(pkce) = pkce_challenge {
        params.append_pair("code_challenge", pkce);
        params.append_pair("code_challenge_method", "S256");
    }
}
