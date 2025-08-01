//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.14

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: Json,
    pub expires_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "_refreshed_at")]
    #[serde(skip)]
    pub refreshed_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "_created_at")]
    #[serde(skip)]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
