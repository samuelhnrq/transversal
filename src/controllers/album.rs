use axum::{
    Form,
    extract::{Path, State, rejection::FormRejection},
    response::{IntoResponse, Redirect},
};
use models::{
    Uuid,
    repositories::album::{
        create_album, delete_album, empty_album, get_album_by_id, list_albums, update_album,
    },
    state::AppState,
};
use reqwest::StatusCode;
use tower_sessions::Session;
use views::AlbumView;

use crate::controllers::auth::session_user;

#[axum_macros::debug_handler]
pub(crate) async fn album_details(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Redirect> {
    let album = get_album_by_id(&state.db, &id)
        .await
        .map_err(|_| Redirect::to("/album"))?
        .ok_or_else(|| Redirect::to("/album"))?;
    let user = session_user(&session).await;

    Ok(AlbumView {
        album: album.into(),
        albums: list_albums(&state.db).await.unwrap_or_default(),
        user,
    })
}

#[axum_macros::debug_handler]
pub(crate) async fn album_list(
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
        user,
    })
}

#[axum_macros::debug_handler]
pub(crate) async fn album_create(
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
pub(crate) async fn album_update(
    session: Session,
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
        user: session_user(&session).await,
    })
}

#[axum_macros::debug_handler]
pub(crate) async fn album_delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    delete_album(&state.db, &id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Redirect::to("/album"))
}
