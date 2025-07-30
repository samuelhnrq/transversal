use crate::auth_utils::{
    build_redirect_url, exchange_token, from_redirect_to_token_payload, generate_auth_url,
};
use axum::{
    extract::{Query, Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use models::{
    generated::user,
    oauth::{AuthRedirectQuery, AuthorizationParams, LoginAttempt},
    state::AppState,
    user_auth::authenticate,
};
use tower_sessions::Session;

const USER_SESSION_KEY: &str = "user";
const AUTH_PARAMS_KEY: &str = "auth_params";

#[axum_macros::debug_middleware]
pub(crate) async fn login_required(session: Session, request: Request, next: Next) -> Response {
    if session_user(&session).await.is_none() {
        log::warn!("Unauthorized access attempt, redirecting to homepage");
        return Redirect::to("/").into_response();
    }
    next.run(request).await
}

#[axum_macros::debug_handler]
pub(crate) async fn login_handler(
    session: Session,
    State(state): State<AppState>,
) -> Result<Redirect, String> {
    log::info!("starting login");
    let config = &state.config;

    let redirect_uri = build_redirect_url(config);
    let params = AuthorizationParams::new(config.oauth_client_id.clone(), redirect_uri);
    log::debug!("Generating auth URL with params: {params:?}");
    let attempt = LoginAttempt::from(params.clone());
    session.insert(AUTH_PARAMS_KEY, attempt).await.ok();
    let url =
        generate_auth_url(params, &state.config).map_err(|_| "Failed to generate auth URL")?;
    log::info!("generated auth url, redirecting to {url}");
    Ok(Redirect::temporary(&url))
}

#[axum_macros::debug_handler]
pub(crate) async fn redirect_handler(
    session: Session,
    State(state): State<AppState>,
    Query(query): Query<AuthRedirectQuery>,
) -> Result<Redirect, String> {
    log::info!("Got oauth2 redirect, reading cookies");
    let attempt = session
        .get::<LoginAttempt>(AUTH_PARAMS_KEY)
        .await
        .map_err(|_| "Failed to deserialize login attempt from session")?
        .ok_or("Failed to find login attempt from session")?;
    let crsf_token = attempt.csrf;
    let pkce = attempt.pkce;
    if query.state != crsf_token {
        log::error!(
            "CRSF attack?! state '{}', stored cookie '{}'",
            query.state,
            crsf_token
        );
        return Ok(Redirect::to("/"));
    }
    log::info!("cookies pass, converting to token exchage payload");
    let token_payload = from_redirect_to_token_payload(&state.config, query, pkce);
    let code = match exchange_token(&state, &token_payload).await {
        Ok(code) => code,
        Err(err) => {
            log::error!("Failed to exchange token {err:?}");
            return Ok(Redirect::to("/"));
        }
    };
    let user = authenticate(&state, code)
        .await
        .map_err(|err| {
            log::error!("Failed to authenticate session: {err:?}");
            "Failed to authenticate session".to_string()
        })?
        .ok_or_else(|| {
            log::error!("Failed to authenticate session, no user found");
            "Failed to authenticate session, no user found".to_string()
        })?;
    session.insert(USER_SESSION_KEY, user).await.ok();
    session.remove::<LoginAttempt>(AUTH_PARAMS_KEY).await.ok();
    Ok(Redirect::to("/"))
}

pub(crate) async fn session_user(session: &Session) -> Option<user::Model> {
    session
        .get::<user::Model>(USER_SESSION_KEY)
        .await
        .ok()
        .flatten()
}

#[axum_macros::debug_handler]
pub(crate) async fn logout_handler(session: Session) -> Redirect {
    session.clear().await;
    Redirect::to("/")
}
