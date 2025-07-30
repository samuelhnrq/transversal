use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use sea_orm::{DatabaseConnection, Order};
use serde_json::json;
use std::io::{Error as IOError, ErrorKind};

use crate::generated::{album, user};

pub async fn list_albums(
    db: &DatabaseConnection,
    user: &user::Model,
) -> Result<Vec<album::Model>, IOError> {
    album::Entity::find()
        .order_by(album::Column::CreatedAt, Order::Desc)
        .filter(album::Column::CreatedBy.eq(user.id))
        .paginate(db, 10)
        .fetch()
        .await
        .map_err(IOError::other)
}

pub async fn get_album_by_id(
    db: &DatabaseConnection,
    user: &user::Model,
    id: &Uuid,
) -> Result<Option<album::Model>, IOError> {
    album::Entity::find()
        .filter(album::Column::Id.eq(*id))
        .filter(album::Column::CreatedBy.eq(user.id))
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
    prepare_value(&mut new_album);
    // Its simpler to appease serde then sea-orm
    new_album["id"] = json!(Uuid::nil().to_string());
    let created_by = new_album
        .get("_created_by")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);
    let mut album = album::ActiveModel::from_json(new_album)
        .inspect_err(|err| log::error!("Failed to create album: {err}"))
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid album data"))?;
    album.created_by = ActiveValue::Set(created_by);
    log::debug!("Creating new album with data: {:?}", album.created_by);
    album.insert(db).await.map_err(IOError::other)
}

pub async fn update_album(
    db: &DatabaseConnection,
    album_id: &Uuid,
    mut updated_album: serde_json::Value,
) -> Result<album::Model, IOError> {
    prepare_value(&mut updated_album);
    let mut album = album::ActiveModel::new();
    album
        .set_from_json(updated_album)
        .inspect_err(|err| log::error!("Failed to update album: {err}"))
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

fn prepare_value(value: &mut serde_json::Value) {
    let in_year = value["year"]
        .as_str()
        .and_then(|x| x.parse::<i128>().ok())
        .unwrap_or_default();
    log::debug!("Creating new album with year: {in_year:?}");
    value["year"] = json!(in_year);
}
