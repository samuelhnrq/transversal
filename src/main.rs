use crate::model::{AppConfig, AppState};
use axum::{
    Form, Router, ServiceExt,
    extract::{Path, Request, State, rejection::FormRejection},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
};
use axum_login::{AuthManagerLayerBuilder, tower_sessions::SessionManagerLayer};
use models::{
    Uuid, get_database,
    session::SeaSessionBackend,
    user::{
        AuthBackend, create_user, delete_user, empty_user, get_user_by_id, list_users, update_user,
    },
};
use std::env::var;
use tokio::net::TcpListener;
use tower_http::{normalize_path::NormalizePath, trace::TraceLayer};
use tracing_subscriber::{filter::LevelFilter, prelude::*};
use views::{IndexPage, UserDetailsPage};

mod model;

#[axum_macros::debug_handler]
async fn home(State(state): State<AppState>) -> impl IntoResponse {
    state.db.ping().await.ok();
    IndexPage {
        name: format!("Hello world from port {}", state.config.port),
    }
}

#[axum_macros::debug_handler]
async fn user_create(
    State(state): State<AppState>,
    user: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(form) = user.map_err(|_| StatusCode::BAD_REQUEST)?;
    create_user(&state.db, form)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(UserDetailsPage {
        user: empty_user(),
        users: list_users(&state.db).await.unwrap_or_default(),
    })
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

fn get_app_config() -> model::AppConfig {
    AppConfig {
        db_url: var("DATABASE_URL").unwrap_or_default(),
        port: var("PORT")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(3000),
    }
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    setup_tracing();
    let config = get_app_config();
    let port = config.port;
    let state = AppState {
        db: get_database(&config.db_url).await?,
        config,
    };
    let session_layer = SessionManagerLayer::new(SeaSessionBackend::new(state.db.clone()));
    let auth_layer =
        AuthManagerLayerBuilder::new(AuthBackend::new(state.db.clone()), session_layer).build();

    let app = Router::new()
        .route("/", get(home))
        .route("/user/{id}", get(user_details))
        .route("/user/{id}", post(user_update))
        .route("/user/{id}", delete(user_delete))
        .route("/user", post(user_create))
        .route("/user", get(user_list))
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
