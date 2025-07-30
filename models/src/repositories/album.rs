use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use sea_orm::{DatabaseConnection, Order};
use std::io::{Error as IOError, ErrorKind};

use crate::generated::album;

pub async fn list_albums(db: &DatabaseConnection) -> Result<Vec<album::Model>, IOError> {
    album::Entity::find()
        .order_by(album::Column::CreatedAt, Order::Desc)
        .paginate(db, 10)
        .fetch()
        .await
        .map_err(IOError::other)
}

pub async fn get_album_by_id(
    db: &DatabaseConnection,
    id: &Uuid,
) -> Result<Option<album::Model>, IOError> {
    album::Entity::find()
        .filter(album::Column::Id.eq(*id))
        .one(db)
        .await
        .map_err(IOError::other)
}

#[must_use]
pub fn empty_album() -> album::ActiveModel {
    album::ActiveModel::new()
}

pub async fn create_album(
    db: &DatabaseConnection,
    mut new_album: serde_json::Value,
) -> Result<album::Model, IOError> {
    let in_year = new_album["year"]
        .as_str()
        .and_then(|x| x.parse::<i128>().ok())
        .unwrap_or_default();
    log::debug!("Creating new album with year: {in_year:?}");
    new_album["year"] =
        serde_json::Value::Number(serde_json::Number::from_i128(in_year).ok_or_else(|| {
            IOError::new(ErrorKind::InvalidData, "Failed to convert year to i128")
        })?); // Ensure year is optional
    let album = album::ActiveModel::from_json(new_album)
        .inspect_err(|err| log::error!("Failed convert album data: {err}"))
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid album data"))?;
    log::debug!("Creating new album with data: {album:?}");
    album.insert(db).await.map_err(IOError::other)
}

pub async fn update_album(
    db: &DatabaseConnection,
    album_id: &Uuid,
    updated_album: serde_json::Value,
) -> Result<album::Model, IOError> {
    let mut album = album::ActiveModel::from_json(updated_album)
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid album data"))?;
    album.id = ActiveValue::unchanged(*album_id);
    album.update(db).await.map_err(IOError::other)
}

pub async fn delete_album(db: &DatabaseConnection, id: &Uuid) -> Result<(), IOError> {
    album::Entity::delete_by_id(*id)
        .exec(db)
        .await
        .map_err(IOError::other)
        .map(|_| ())
}
