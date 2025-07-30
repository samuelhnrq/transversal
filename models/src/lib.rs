pub use sea_orm::DatabaseConnection;
use sea_orm::{Database, Order, QueryOrder};
use std::io::{self, Error as IOError};

pub use sea_orm::ActiveValue;
pub use sea_orm::entity::prelude::*;

use crate::generated::user;
pub mod generated;

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<user::Model>, IOError> {
    user::Entity::find()
        .order_by(user::Column::CreatedAt, Order::Desc)
        .paginate(db, 10)
        .fetch()
        .await
        .map_err(IOError::other)
}

pub async fn get_database(db_url: &str) -> Result<DatabaseConnection, IOError> {
    log::info!("Connecting to database...");
    if db_url.is_empty() {
        return Err(IOError::new(
            io::ErrorKind::InvalidInput,
            "Database URL is empty",
        ));
    }
    Database::connect(db_url)
        .await
        .inspect(|_| log::info!("Connected to database"))
        .map_err(IOError::other)
}
