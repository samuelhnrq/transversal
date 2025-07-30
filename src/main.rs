use auth_utils::load_openid_config;
use axum::{
    Router, ServiceExt,
    extract::Request,
    routing::{delete, get, post},
};
use models::{
    get_database,
    oauth::{OAUTH_CALLBACK_ENDPOINT, OAUTH_LOGIN_ENDPOINT, OAUTH_LOGOUT_ENDPOINT},
    session::SeaSessionBackend,
    state::{AppConfig, AppState},
};
use reqwest::Url;
use std::env::var;
use tokio::net::TcpListener;
use tower_http::{normalize_path::NormalizePath, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};
use tracing_subscriber::{filter::LevelFilter, prelude::*};

use crate::controllers::{
    album::{album_create, album_delete, album_details, album_list, album_update},
    auth, home,
};

mod auth_utils;
mod controllers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    setup_tracing();
    let config = load_app_config().await;
    let port = config.port;
    let state = AppState {
        db: get_database(&config.db_url).await?,
        requests: reqwest::Client::new(),
        config,
    };
    let session_layer = SessionManagerLayer::new(SeaSessionBackend::new(state.db.clone()))
        .with_secure(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let app = Router::new()
        .route("/", get(home))
        .route("/album/{id}", get(album_details))
        .route("/album/{id}", post(album_update))
        .route("/album/{id}", delete(album_delete))
        .route("/album", get(album_list))
        .route("/album", post(album_create))
        .route(OAUTH_CALLBACK_ENDPOINT, get(auth::redirect_handler))
        .route(OAUTH_LOGOUT_ENDPOINT, get(auth::logout_handler))
        .route(OAUTH_LOGIN_ENDPOINT, get(auth::login_handler))
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let app = NormalizePath::trim_trailing_slash(app);
    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("Listening on port {port}");
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await?;
    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// # Panics
/// if environment variables are not set correctly
async fn load_app_config() -> AppConfig {
    let oauth_url = var("OAUTH_DISCOVER_URL").expect("OAUTH_DISCOVER_URL must be set");
    let self_url = var("SELF_URL").expect("SELF_URL must be set");
    let self_url = Url::parse(&self_url).expect("Invalid SELF_URL format");
    AppConfig {
        db_url: var("DATABASE_URL").expect("DATABASE_URL must be set"),
        port: var("PORT")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(8889),
        oauth: load_openid_config(&oauth_url).await,
        oauth_client_id: var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set"),
        oauth_autodiscover_url: oauth_url,
        oauth_client_secret: var("OAUTH_CLIENT_SECRET").expect("OAUTH_CLIENT_SECRET must be set"),
        self_url,
    }
}
