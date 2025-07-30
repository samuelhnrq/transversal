use axum::{extract::State, response::IntoResponse};
use models::state::AppState;
use views::IndexPage;

pub(crate) mod album;

#[axum_macros::debug_handler]
pub(crate) async fn home(State(state): State<AppState>) -> impl IntoResponse {
    state.db.ping().await.ok();
    IndexPage {
        name: format!("Running on port {}", state.config.port),
    }
}
