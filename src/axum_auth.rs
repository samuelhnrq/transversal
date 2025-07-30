// use std::net::{IpAddr, SocketAddr};

use auth::{build_redirect_url, exchange_token, from_redirect_to_token_payload, generate_auth_url};
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use models::{
    oauth::{AuthRedirectQuery, AuthorizationParams, LoginAttempt},
    state::AppState,
    user_auth::SeaAuthSession,
};

const AUTH_PARAMS_KEY: &str = "params";

#[allow(clippy::unused_async)]
#[axum_macros::debug_handler]
pub async fn login_handler(
    session: SeaAuthSession,
    State(state): State<AppState>,
) -> Result<Redirect, String> {
    log::info!("starting login");
    let config = &state.config;

    let redirect_uri = build_redirect_url(config);
    let params = AuthorizationParams::new(config.oauth_client_id.clone(), redirect_uri);
    log::debug!("Generating auth URL with params: {params:?}");
    let attempt = LoginAttempt::from(params.clone());
    session.session.insert(AUTH_PARAMS_KEY, attempt).await.ok();
    let url =
        generate_auth_url(params, &state.config).map_err(|_| "Failed to generate auth URL")?;
    log::info!("generated auth url, redirecting to {url}");
    Ok(Redirect::temporary(&url))
}

#[axum_macros::debug_handler]
pub async fn handle_oauth_redirect(
    session: SeaAuthSession,
    State(state): State<AppState>,
    Query(query): Query<AuthRedirectQuery>,
) -> Result<Redirect, String> {
    log::info!("Got oauth2 redirect, reading cookies");
    let config = &state.config;
    let attempt = session
        .session
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
        return Ok(Redirect::to(config.self_url.as_ref()));
    }
    log::info!("cookies pass, converting to token exchage payload");
    let token_payload = from_redirect_to_token_payload(config, query, pkce);
    let code = match exchange_token(&state, &token_payload).await {
        Ok(code) => code,
        Err(err) => {
            log::error!("Failed to exchange token {err:?}");
            return Ok(Redirect::to(config.self_url.as_ref()));
        }
    };
    session.authenticate(code).await.ok();
    Ok(Redirect::to("/"))
}

// #[axum_macros::debug_handler]
// pub async fn logout_handler(
//     State(state): State<AppState>,
//     jar: PrivateCookieJar,
// ) -> impl IntoResponse {
//     log::info!("starting logout");
//     let mut params = HashMap::<&str, String>::new();
//     params.insert("client_id", LOADED_CONFIG.oauth_client_id.to_string());
//     if let Some(refresh_token) = jar.get("refresh_token") {
//         log::info!("has refresh adding to logout payload");
//         params.insert("refresh_token", refresh_token.value_trimmed().to_string());
//     }
//     let resp = state
//         .requests
//         .post(state.oauth_config.end_session_endpoint)
//         .basic_auth(
//             &LOADED_CONFIG.oauth_client_id,
//             Some(&LOADED_CONFIG.oauth_client_secret),
//         )
//         .form(&params)
//         .send()
//         .await
//         .inspect_err(|err| log::error!("Failed to revoke endpoint {:?}", err))
//         .ok();
//     let jar = if let Some(x) = resp {
//         let body = x.text().await.unwrap_or_default();
//         log::info!("Got successfull answer! {body}");
//         jar.remove(safe_cookie("token", ""))
//             .remove(safe_cookie("refresh_token", ""))
//     } else {
//         jar
//     };
//     (jar, Redirect::to("/"))
// }

// fn is_safe_requester(addr: SocketAddr) -> bool {
//     match addr.ip() {
//         IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback(),
//         IpAddr::V6(ipv6) => ipv6.is_loopback(),
//     }
// }

// fn build_unauthorized_response(cause: &'static str) -> Response {
//     let error = UnauthorizedError::new(cause);
//     let mut unauthorized = Json(error).into_response();
//     *unauthorized.status_mut() = StatusCode::FORBIDDEN;
//     unauthorized
// }

// pub async fn required_login_middleware(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     request: Request,
//     next: Next,
// ) -> Response {
//     let user_option = request.extensions().get::<Claims>();
//     if is_safe_requester(addr) || user_option.is_some() {
//         next.run(request).await
//     } else {
//         build_unauthorized_response("Unauthorized")
//     }
// }

// pub async fn user_data_extension(
//     mut jar: PrivateCookieJar,
//     State(state): State<AppState>,
//     mut request: Request,
//     next: Next,
// ) -> (PrivateCookieJar, Response) {
//     if let Some(user_data) = validate_cookie(&mut jar, &state).await {
//         let db_conn = &state.connection;
//         if let Some(user) = find_by_sub(db_conn, &user_data.sub).await {
//             request.extensions_mut().insert(user);
//             request.extensions_mut().insert(user_data);
//             log::debug!("Inserted extensions successfully");
//         } else {
//             log::error!("JWT valid bug sub not found in database, not trusting cookie");
//         }
//     } else {
//         log::debug!("Cookie did not pass validation");
//     }
//     (jar, next.run(request).await)
// }
