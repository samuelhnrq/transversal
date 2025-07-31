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

pub async fn create_album(
    db: &DatabaseConnection,
    user: &user::Model,
    mut new_album: serde_json::Value,
) -> Result<album::Model, IOError> {
    sanitize_payload(&mut new_album);

    let mut album = album::ActiveModel::from_json(new_album)
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid album data"))?;
    album.created_by = ActiveValue::set(user.id);
    album.id = ActiveValue::NotSet; // Let the database generate the UUID
    log::debug!("Creating new album with data: {:?}", album.created_by);
    album.insert(db).await.map_err(IOError::other)
}

pub async fn update_album(
    db: &DatabaseConnection,
    album_id: &Uuid,
    user: &user::Model,
    mut payload: serde_json::Value,
) -> Result<album::Model, IOError> {
    sanitize_payload(&mut payload);
    let mut album = album::ActiveModel::new();
    album
        .set_from_json(payload)
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid album data"))?;
    album.created_by = ActiveValue::NotSet; // Do not change created_by on update
    album.created_at = ActiveValue::NotSet; // Do not change created_at on update
    album.updated_at = ActiveValue::Set(chrono::Utc::now().into());
    album.id = ActiveValue::unchanged(*album_id);
    album::Entity::update(album)
        .filter(album::Column::Id.eq(*album_id))
        .filter(album::Column::CreatedBy.eq(user.id))
        .exec(db)
        .await
        .map_err(IOError::other)
}

pub async fn delete_album(db: &DatabaseConnection, id: &Uuid) -> Result<(), IOError> {
    album::Entity::delete_by_id(*id)
        .exec(db)
        .await
        .map_err(IOError::other)
        .map(|_| ())
}

#[must_use]
pub fn empty_album() -> album::ActiveModel {
    album::ActiveModel::new()
}

/// Sanitize the payload by removing sensitive fields and ensuring correct types
/// This function modifies the payload in place.
fn sanitize_payload(value: &mut serde_json::Value) {
    value.as_object_mut().map(|obj| {
        obj.remove("_created_by");
        obj.remove("_created_at");
        obj.remove("_updated_at");
        Some(())
    });
    let in_year = value
        .get("year")
        .and_then(|x| x.as_str())
        .and_then(|y| y.parse::<i128>().ok())
        .unwrap_or_default();
    value["id"] = json!(Uuid::nil().to_string());
    value["year"] = json!(in_year);
    value["_updated_at"] = json!(chrono::Utc::now().to_rfc2822());
}
