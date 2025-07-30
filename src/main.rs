use crate::model::{AppConfig, AppState};
use axum::{
    Form, Router, ServiceExt,
    extract::{Path, Request, State, rejection::FormRejection},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
};
use models::{
    ActiveValue,
    generated::user::{self, *},
    get_database,
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
async fn user_details(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Redirect> {
    let user = user::Entity::find()
        .filter(user::Column::Id.eq(id))
        .one(&state.db)
        .await
        .map_err(|_| Redirect::to("/user"))?
        .ok_or_else(|| Redirect::to("/user"))?;
    Ok(UserDetailsPage {
        user: user.into(),
        users: user::Entity::find()
            .paginate(&state.db, 10)
            .fetch()
            .await
            .unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn user_create(
    State(state): State<AppState>,
    user: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(user) = user.map_err(|_| StatusCode::BAD_REQUEST)?;
    let user = user::ActiveModel::from_json(user)
        .inspect_err(|e| {
            log::error!("Failed to deserialize user: {e:?}");
        })
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    user.insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(UserDetailsPage {
        user: user::ActiveModel::new(),
        users: user::Entity::find()
            .paginate(&state.db, 10)
            .fetch()
            .await
            .unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn list_users(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let users = user::Entity::find()
        .paginate(&state.db, 10)
        .fetch()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(UserDetailsPage {
        user: user::ActiveModel::new(),
        users,
    })
}

#[axum_macros::debug_handler]
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(user) = user.map_err(|_| StatusCode::BAD_REQUEST)?;
    let user: user::ActiveModel =
        user::ActiveModel::from_json(user).map_err(|_| StatusCode::BAD_REQUEST)?;
    log::info!("Updating user with ID: {id} and {user:?}");
    let user = user
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(UserDetailsPage {
        user: user.into(),
        users: user::Entity::find()
            .paginate(&state.db, 10)
            .fetch()
            .await
            .unwrap_or_default(),
    })
}

#[axum_macros::debug_handler]
async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user::ActiveModel {
        id: ActiveValue::Set(id),
        ..Default::default()
    };
    user.delete(&state.db)
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
    let app = Router::new()
        .route("/", get(home))
        .route("/user/{id}", get(user_details))
        .route("/user/{id}", post(update_user))
        .route("/user/{id}", delete(delete_user))
        .route("/user", post(user_create))
        .route("/user", get(list_users))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let app = NormalizePath::trim_trailing_slash(app);
    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("Listening on port {port}");
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await?;
    Ok(())
}
