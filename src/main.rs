use crate::model::{AppConfig, AppState};
use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use models::{
    generated::user::{self, *},
    get_database,
};
use std::env::var;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
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
    Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let user = user::Entity::find()
        .filter(user::Column::Id.eq(id))
        .one(&state.db)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    Ok(UserDetailsPage {
        user: user.map(|user| user.into()).unwrap_or_default(),
    })
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
        config: config,
    };
    let app = Router::new()
        .route("/", get(home))
        .route("/user/{id}", get(user_details))
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("Listening on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
