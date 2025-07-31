use axum::{
    Extension, Form,
    extract::{Path, State, rejection::FormRejection},
    response::{IntoResponse, Redirect},
};
use models::{
    Uuid,
    generated::user,
    repositories::album::{
        create_album, delete_album, empty_album, get_album_by_id, list_albums, update_album,
    },
    state::AppState,
};
use reqwest::StatusCode;
use views::AlbumView;

const ALBUM_PATH: &str = "/album";

#[axum_macros::debug_handler]
pub(crate) async fn album_details(
    State(state): State<AppState>,
    Extension(user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Redirect> {
    let album = get_album_by_id(&state.db, &user, &id)
        .await
        .inspect_err(|err| log::error!("Failed to get album by id: {err}"))
        .map_err(|_| Redirect::to(ALBUM_PATH))?
        .ok_or_else(|| Redirect::to(ALBUM_PATH))?;
    Ok(AlbumView {
        album: album.into(),
        albums: list_albums(&state.db, &user).await.unwrap_or_default(),
        user: Some(user),
    })
}

#[axum_macros::debug_handler]
pub(crate) async fn album_list(
    State(state): State<AppState>,
    Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    log::info!("Listing albums for user: {:?}", user.id);
    let albums = list_albums(&state.db, &user)
        .await
        .inspect_err(|err| log::error!("Failed to list albums: {err}"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AlbumView {
        album: empty_album(),
        albums,
        user: Some(user),
    })
}

#[axum_macros::debug_handler]
pub(crate) async fn album_create(
    State(state): State<AppState>,
    Extension(user): Extension<user::Model>,
    album: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(form) = album
        .inspect_err(|err| log::error!("Failed to parse album form: {err}"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    create_album(&state.db, &user, form)
        .await
        .inspect_err(|err| log::error!("Failed to create album: {err}"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Redirect::to(ALBUM_PATH))
}

#[axum_macros::debug_handler]
pub(crate) async fn album_update(
    Extension(user): Extension<user::Model>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    album: Result<Form<serde_json::Value>, FormRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let Form(form) = album.map_err(|_| StatusCode::BAD_REQUEST)?;
    update_album(&state.db, &id, &user, form)
        .await
        .inspect_err(|err| log::error!("Failed to update album: {err}"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(AlbumView {
        album: empty_album(),
        albums: list_albums(&state.db, &user).await.unwrap_or_default(),
        user: Some(user),
    })
}

#[axum_macros::debug_handler]
pub(crate) async fn album_delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    delete_album(&state.db, &id)
        .await
        .inspect_err(|err| log::error!("Failed to delete album: {err}"))
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Redirect::to(ALBUM_PATH))
}
