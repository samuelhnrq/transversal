use auth::load_openid_config;
use axum::{
    Form, Router, ServiceExt,
    extract::{Path, Request, State, rejection::FormRejection},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
};
use models::{
    Uuid, get_database,
    oauth::{OAUTH_CALLBACK_ENDPOINT, OAUTH_LOGIN_ENDPOINT},
    repositories::album::{
        create_album, delete_album, empty_album, get_album_by_id, list_albums, update_album,
    },
    session::SeaSessionBackend,
    state::{AppConfig, AppState},
};
use reqwest::Url;
use std::env::var;
use tokio::net::TcpListener;
use tower_http::{normalize_path::NormalizePath, trace::TraceLayer};
use tower_sessions::{Expiry, Session, SessionManagerLayer, cookie::time::Duration};
use tracing_subscriber::{filter::LevelFilter, prelude::*};
use views::{AlbumView, IndexPage};

use crate::axum_auth::session_user;

mod auth;
mod axum_auth;

#[axum_macros::debug_handler]
async fn home(State(state): State<AppState>) -> impl IntoResponse {
    state.db.ping().await.ok();
    IndexPage {
        name: format!("Running on port {}", state.config.port),
    }
}

#[axum_macros::debug_handler]
async fn album_details(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Redirect> {
    let album = get_album_by_id(&state.db, &id)
        .await
        .map_err(|_| Redirect::to("/album"))?
        .ok_or_else(|| Redirect::to("/album"))?;

    Ok(AlbumView {
        album: album.into(),
        albums: list_albums(&state.db).await.unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn album_list(
    session: Session,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = session_user(&session).await;
    log::info!("Listing albums for session: {:?}", user.is_some());
    let albums = list_albums(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AlbumView {
        album: empty_album(),
        albums,
    })
}

#[axum_macros::debug_handler]
async fn album_create(
    State(state): State<AppState>,
    album: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(form) = album
        .inspect_err(|err| log::error!("Failed to parse album form: {err}"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    create_album(&state.db, form)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Redirect::to("/album"))
}

#[axum_macros::debug_handler]
async fn album_update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    album: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(form) = album.map_err(|_| StatusCode::BAD_REQUEST)?;
    update_album(&state.db, &id, form)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(AlbumView {
        album: empty_album(),
        albums: list_albums(&state.db).await.unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn album_delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    delete_album(&state.db, &id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Redirect::to("/album"))
}

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
        .route(OAUTH_CALLBACK_ENDPOINT, get(axum_auth::redirect_handler))
        .route(OAUTH_LOGIN_ENDPOINT, get(axum_auth::login_handler))
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
