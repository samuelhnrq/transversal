use base64ct::{Base64Url, Encoding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const OAUTH_LOGIN_ENDPOINT: &str = "/auth/login";
pub const OAUTH_LOGOUT_ENDPOINT: &str = "/auth/logout";
pub const OAUTH_CALLBACK_ENDPOINT: &str = "/auth/callback";

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub preferred_username: Option<String>,
    pub picture: Option<String>,
    pub email: String,
    pub birthdate: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenIdConfiguration {
    pub jwks_uri: String,
    pub authorization_endpoint: String,
    pub end_session_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    // pub issuer: String,
    // pub backchannel_logout_supported: bool,
    // pub frontchannel_logout_supported: bool,
    // pub grant_types_supported: Vec<String>,
    // pub response_modes_supported: Vec<String>,
    // pub response_types_supported: Vec<String>,
    // pub token_endpoint_auth_methods_supported: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthRedirectQuery {
    pub state: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenExchangePayload {
    pub code: String,
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshPayload {
    pub grant_type: String,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    // pub expires_in: i64,
    // pub refresh_expires_in: Option<i64>,
    // pub token_type: String,
    // pub id_token: Option<String>,
    // pub session_state: String,
    // pub scope: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub audience: String,
    pub response_mode: String,
    pub response_type: String,
    pub scope: String,
    #[serde(skip)]
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub pkce: String,
    pub csrf: String,
}

impl From<AuthorizationParams> for LoginAttempt {
    fn from(params: AuthorizationParams) -> Self {
        LoginAttempt {
            pkce: params.code_verifier,
            csrf: params.state,
        }
    }
}

fn generate_pkce() -> String {
    let pkce_verifier = Uuid::new_v4();
    format!("{pkce_verifier}-{pkce_verifier}")
}

impl AuthorizationParams {
    #[must_use]
    pub fn new(oauth_client_id: String, redirect_uri: String) -> Self {
        let pkce_verifier = generate_pkce();
        let mut hasher = Sha256::new();
        hasher.update(&pkce_verifier);
        let padded_pkce_challenge = Base64Url::encode_string(&hasher.finalize());
        let pkce_challenge = padded_pkce_challenge
            .strip_suffix('=')
            .unwrap_or(&padded_pkce_challenge);
        let crsf = Uuid::new_v4();
        AuthorizationParams {
            client_id: oauth_client_id,
            redirect_uri,
            state: crsf.to_string(),
            response_mode: "query".to_string(),
            audience: "hyper-tarot".to_string(),
            response_type: "code".to_string(),
            scope: "offline_access openid email profile".to_string(),
            code_verifier: pkce_verifier.to_string(),
            code_challenge: pkce_challenge.to_string(),
            code_challenge_method: "S256".to_string(),
        }
    }
}
