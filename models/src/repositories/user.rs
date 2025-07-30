use sea_orm::{ActiveValue, DatabaseConnection, entity::prelude::*, sea_query::OnConflict};
use std::io::Error as IOError;

use crate::generated::user;
use crate::oauth::UserInfo;

pub(crate) async fn get_user_by_id(
    db: &DatabaseConnection,
    id: &Uuid,
) -> Result<Option<user::Model>, IOError> {
    user::Entity::find()
        .filter(user::Column::Id.eq(*id))
        .one(db)
        .await
        .map_err(IOError::other)
}

pub(crate) async fn upsert_user(
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
