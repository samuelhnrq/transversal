use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, QueryOrder};
use sea_orm::{DatabaseConnection, Order};
use std::io::{Error as IOError, ErrorKind};

use crate::generated::user;
use crate::oauth::UserInfo;

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<user::Model>, IOError> {
    user::Entity::find()
        .order_by(user::Column::CreatedAt, Order::Desc)
        .paginate(db, 10)
        .fetch()
        .await
        .map_err(IOError::other)
}

pub async fn get_user_by_id(
    db: &DatabaseConnection,
    id: &Uuid,
) -> Result<Option<user::Model>, IOError> {
    user::Entity::find()
        .filter(user::Column::Id.eq(*id))
        .one(db)
        .await
        .map_err(IOError::other)
}

#[must_use]
pub fn empty_user() -> user::ActiveModel {
    user::ActiveModel::new()
}

pub async fn upsert_user(
    db: &DatabaseConnection,
    new_user: UserInfo,
) -> Result<user::Model, IOError> {
    let mut model = user::ActiveModel {
        ..Default::default()
    };
    model.sid = ActiveValue::Set(new_user.sub);
    model.email = ActiveValue::Set(new_user.email);
    model.name = ActiveValue::Set(new_user.name);
    user::Entity::insert(model)
        .on_conflict(
            OnConflict::column(user::Column::Sid)
                .update_column(user::Column::Email)
                .update_column(user::Column::Name)
                .to_owned(),
        )
        .exec_with_returning(db)
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
