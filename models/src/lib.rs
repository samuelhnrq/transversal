use sea_orm::Database;
pub use sea_orm::entity::prelude::*;
pub use sea_orm::{ActiveValue, DatabaseConnection, Value};
use std::io::{self, Error as IOError};

pub mod generated;
pub mod oauth;
pub mod repositories;
pub mod session;
pub mod state;
pub mod user_auth;

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
