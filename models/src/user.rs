use axum_login::AuthUser;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use sea_orm::{DatabaseConnection, Order};
use std::io::{Error as IOError, ErrorKind};

use crate::generated::user;

impl AuthUser for user::Model {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.token.as_bytes()
    }
}

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<user::Model>, IOError> {
    user::Entity::find()
        .order_by(user::Column::CreatedAt, Order::Desc)
        .paginate(db, 10)
        .fetch()
        .await
        .map_err(IOError::other)
}

pub async fn get_user_by_id(db: &DatabaseConnection, id: &Uuid) -> Result<user::Model, IOError> {
    user::Entity::find()
        .filter(user::Column::Id.eq(*id))
        .one(db)
        .await
        .map_err(IOError::other)
        .and_then(|user| user.ok_or_else(|| IOError::new(ErrorKind::NotFound, "User not found")))
}

pub fn empty_user() -> user::ActiveModel {
    user::ActiveModel::new()
}

pub async fn create_user(
    db: &DatabaseConnection,
    new_user: serde_json::Value,
) -> Result<user::Model, IOError> {
    user::ActiveModel::from_json(new_user)
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid user data"))?
        .insert(db)
        .await
        .map_err(IOError::other)
}

pub async fn update_user(
    db: &DatabaseConnection,
    user_id: &Uuid,
    updated_user: serde_json::Value,
) -> Result<user::Model, IOError> {
    let mut user = user::ActiveModel::from_json(updated_user)
        .map_err(|_| IOError::new(ErrorKind::InvalidData, "Invalid user data"))?;
    user.id = ActiveValue::unchanged(*user_id);
    user.update(db).await.map_err(IOError::other)
}

pub async fn delete_user(db: &DatabaseConnection, id: &Uuid) -> Result<(), IOError> {
    user::Entity::delete_by_id(*id)
        .exec(db)
        .await
        .map_err(IOError::other)
        .map(|_| ())
}
