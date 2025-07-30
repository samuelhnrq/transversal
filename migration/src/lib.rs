pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250722_152740_create_users;
mod m20250724_161551_sessions;
mod m20250727_234302_albums;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250722_152740_create_users::Migration),
            Box::new(m20250724_161551_sessions::Migration),
            Box::new(m20250727_234302_albums::Migration),
        ]
    }
}
