use jsonwebtoken::{
    DecodingKey,
    jwk::{Jwk, JwkSet, PublicKeyUse},
};
use models::oauth::{OAUTH_CALLBACK_ENDPOINT, TokenExchangePayload};
use models::{
    oauth::{AuthRedirectQuery, AuthorizationParams, OpenIdConfiguration, TokenResponse},
    state::{AppConfig, AppState},
};
use reqwest::{Client, Url};
use serde::Serialize;
use std::io::Error as IOError;
use std::{collections::HashMap, error::Error};

pub fn generate_auth_url(
    params: AuthorizationParams,
    config: &AppConfig,
) -> Result<String, IOError> {
    let encoded_params = serde_urlencoded::to_string(params).map_err(IOError::other)?;
    let mut url = Url::parse(&config.oauth.authorization_endpoint).map_err(IOError::other)?;
    url.set_query(Some(&encoded_params));
    Ok(url.to_string())
}

// fn build_validation() -> Validation {
//     let mut val = Validation::new(Algorithm::RS256);
//     val.validate_aud = false;
//     val
// }

/// # Panics
/// if it fails to fetch the config remotly
pub async fn load_openid_config(url: &str) -> OpenIdConfiguration {
    let trimmed = url.strip_suffix('/').unwrap_or(url);
    let issuer_url = Url::parse(&format!("{trimmed}/.well-known/openid-configuration"))
        .expect("Invalid oauth config URL");
    log::info!("Fetching oauth config at {issuer_url}");
    reqwest::get(issuer_url)
        .await
        .expect("Failed to fetch oauth config URL")
        .json::<OpenIdConfiguration>()
        .await
        .expect("Failed to deserialized oauth response")
}

/// # Panics
/// if cant get the JWKS
pub async fn fetch_remote_jwk(request: &Client, config: &OpenIdConfiguration) -> DecodingKey {
    log::info!("Fetching JWKS remotely");
    let resp = request
        .get(&config.jwks_uri)
        .send()
        .await
        .expect("Failed to rearch clerk, invalid URL?")
        .json::<JwkSet>()
        .await
        .expect("Failed to deserialize JWKS response");
    log::info!("Fetched JWKS successfully");
    let jwk = resp
        .keys
        .iter()
        .find(|&x| is_sig_key(x))
        .expect("JWKS without any sig keys?!?!");
    DecodingKey::from_jwk(jwk).unwrap()
}

#[must_use]
pub fn from_redirect_to_token_payload(
    config: &AppConfig,
    value: AuthRedirectQuery,
    pkce: String,
) -> TokenExchangePayload {
    TokenExchangePayload {
        code: value.code,
        client_id: config.oauth_client_id.clone(),
        client_secret: config.oauth_client_secret.clone(),
        code_verifier: pkce,
        grant_type: "authorization_code".to_string(),
        redirect_uri: build_redirect_url(config),
    }
}

pub async fn exchange_token<T: Serialize>(
    state: &AppState,
    payload: &T,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client_id = state.config.oauth_client_id.clone();
    let client_secret = state.config.oauth_client_secret.clone();
    let response = state
        .requests
        .post(&state.config.oauth.token_endpoint)
        .form(&payload)
        .basic_auth(client_id, Some(client_secret))
        .send()
        .await
        .map_err(Box::new)?;
    response.json::<TokenResponse>().await.map_err(|x| {
        let err = x.source();
        log::error!("Failed to deserialize token response: {err:?}");
        Box::new(x) as Box<dyn std::error::Error>
    })
}

// fn from_refresh_to_token_payload(token: String) -> RefreshPayload {
//     RefreshPayload {
//         refresh_token: token,
//         client_id: LOADED_CONFIG.oauth_client_id.clone(),
//         client_secret: LOADED_CONFIG.oauth_client_secret.clone(),
//         grant_type: "refresh_token".to_string(),
//         redirect_uri: build_redirect_url(),
//     }
// }

#[must_use]
pub fn build_redirect_url(config: &AppConfig) -> String {
    let mut redirect_url = config.self_url.clone();
    redirect_url.set_path(OAUTH_CALLBACK_ENDPOINT);
    redirect_url.to_string()
}

#[must_use]
pub fn generate_logout_url(config: &AppConfig) -> String {
    let Ok(mut end_session_url) = Url::parse(&config.oauth.end_session_endpoint) else {
        return "Failed to generate logout URL".to_string();
    };
    let mut logout_params = HashMap::new();
    logout_params.insert("client_id", config.oauth_client_id.clone());
    end_session_url.set_query(serde_urlencoded::to_string(logout_params).ok().as_deref());
    end_session_url.to_string()
}

fn is_sig_key(key: &Jwk) -> bool {
    key.common
        .public_key_use
        .as_ref()
        .is_some_and(|k| *k == PublicKeyUse::Signature)
}
