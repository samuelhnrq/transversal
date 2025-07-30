use auth::load_openid_config;
use axum::{
    Form, Router, ServiceExt,
    extract::{Path, Request, State, rejection::FormRejection},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
};
use axum_login::{
    AuthManagerLayerBuilder,
    tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration},
};
use models::{
    Uuid, get_database,
    oauth::{OAUTH_CALLBACK_ENDPOINT, OAUTH_LOGIN_ENDPOINT},
    session::SeaSessionBackend,
    state::{AppConfig, AppState},
    user_auth::SeaAuthBackend,
    user_repository::{delete_user, empty_user, get_user_by_id, list_users, update_user},
};
use reqwest::Url;
use std::env::var;
use tokio::net::TcpListener;
use tower_http::{normalize_path::NormalizePath, trace::TraceLayer};
use tracing_subscriber::{filter::LevelFilter, prelude::*};
use views::{IndexPage, UserDetailsPage};

mod auth;
mod axum_auth;

#[axum_macros::debug_handler]
async fn home(State(state): State<AppState>) -> impl IntoResponse {
    state.db.ping().await.ok();
    IndexPage {
        name: format!("Hello world from port {}", state.config.port),
    }
}

#[axum_macros::debug_handler]
async fn user_details(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Redirect> {
    let user = get_user_by_id(&state.db, &id)
        .await
        .map_err(|_| Redirect::to("/user"))?
        .ok_or_else(|| Redirect::to("/user"))?;
    Ok(UserDetailsPage {
        user: user.into(),
        users: list_users(&state.db).await.unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn user_list(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let users = list_users(&state.db).await.unwrap_or_default();
    Ok(UserDetailsPage {
        user: empty_user(),
        users,
    })
}

#[axum_macros::debug_handler]
async fn user_update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(form) = user.map_err(|_| StatusCode::BAD_REQUEST)?;
    update_user(&state.db, &id, form)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(UserDetailsPage {
        user: empty_user(),
        users: list_users(&state.db).await.unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn user_delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    delete_user(&state.db, &id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Redirect::to("/user"))
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

    let auth_layer =
        AuthManagerLayerBuilder::new(SeaAuthBackend::new(state.clone()), session_layer).build();

    let app = Router::new()
        .route("/", get(home))
        .route("/user/{id}", get(user_details))
        .route("/user/{id}", post(user_update))
        .route("/user/{id}", delete(user_delete))
        .route("/user", get(user_list))
        .route(OAUTH_CALLBACK_ENDPOINT, get(axum_auth::redirect_handler))
        .route(OAUTH_LOGIN_ENDPOINT, get(axum_auth::login_handler))
        .layer(auth_layer)
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
