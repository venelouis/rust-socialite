use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, distributions::Alphanumeric};
use sha2::{Digest, Sha256};

/// Generates a (code_verifier, code_challenge) pair for OAuth2 PKCE.
///
/// - `code_verifier`: A high-entropy cryptographic random string. The developer MUST store this in the session/cookie.
/// - `code_challenge`: The base64-url-encoded SHA256 hash of the verifier. Sent in the authorization URL.
pub fn generate_pkce() -> (String, String) {
    // Generate a 64-character random string (verifier)
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    // SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let result = hasher.finalize();

    // Base64-url encoding without padding
    let code_challenge = general_purpose::URL_SAFE_NO_PAD.encode(result);

    (code_verifier, code_challenge)
}
