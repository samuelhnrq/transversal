use sea_orm::Database;
pub use sea_orm::DatabaseConnection;
use std::io::{self, Error as IOError};

pub use sea_orm::ActiveValue;
pub use sea_orm::prelude::*;
pub mod generated;

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
        .map_err(|err| IOError::new(io::ErrorKind::Other, err))
}
