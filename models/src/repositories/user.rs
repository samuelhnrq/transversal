use sea_orm::{ActiveValue, DatabaseConnection, entity::prelude::*, sea_query::OnConflict};
use std::io::Error as IOError;

use crate::generated::user;
use crate::oauth::UserInfo;

pub(crate) async fn get_user_by_sub(
    db: &DatabaseConnection,
    sub: &str,
) -> Result<Option<user::Model>, IOError> {
    user::Entity::find()
        .filter(user::Column::Sid.eq(sub))
        .one(db)
        .await
        .map_err(IOError::other)
}

pub(crate) async fn upsert_user(
    db: &DatabaseConnection,
    new_user: UserInfo,
) -> Result<user::Model, IOError> {
    let model = user::ActiveModel {
        sid: ActiveValue::Set(new_user.sub),
        email: ActiveValue::Set(new_user.email),
        name: ActiveValue::Set(new_user.name),
        ..Default::default()
    };
    let on_conflict = OnConflict::column(user::Column::Sid)
        .update_column(user::Column::Email)
        .update_column(user::Column::Name)
        .to_owned();
    user::Entity::insert(model)
        .on_conflict(on_conflict)
        .exec_with_returning(db)
        .await
        .map_err(IOError::other)
}
