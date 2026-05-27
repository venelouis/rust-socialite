use rust_socialite::providers::{CognitoProvider, OktaProvider};
use rust_socialite::provider::Provider;

#[test]
fn test_okta_no_panic() {
    let okta = OktaProvider::new(
        "client".to_string(),
        "secret".to_string(),
        "redirect".to_string(),
        "not a url::/invalid".to_string(),
    );
    let url = okta.redirect_url();
    assert!(url.contains("not a url::/invalid"));
}

#[test]
fn test_cognito_no_panic() {
    let cognito = CognitoProvider::new(
        "client".to_string(),
        "secret".to_string(),
        "redirect".to_string(),
        "not a url::/invalid".to_string(),
    );
    let url = cognito.redirect_url();
    assert!(url.contains("not a url::/invalid"));
}
