use axum::{extract::State, response::IntoResponse};
use models::state::AppState;
use tower_sessions::Session;
use views::IndexPage;

use crate::controllers::auth::session_user;

pub(crate) mod album;
pub(crate) mod auth;

#[axum_macros::debug_handler]
pub(crate) async fn home(session: Session, State(state): State<AppState>) -> impl IntoResponse {
    state.db.ping().await.ok();
    let user = session_user(&session).await;
    log::info!("Rendering home page for user: {user:?}");
    IndexPage { user }
}
