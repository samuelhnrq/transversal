use crate::m20250722_152740_create_users::User;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let alter_add_column = Table::alter()
            .table(User::Table)
            .add_column(
                ColumnDef::new("_created_at")
                    .timestamp()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .add_column(
                ColumnDef::new("_updated_at")
                    .timestamp()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .to_owned();
        manager.alter_table(alter_add_column).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_column = Table::alter()
            .table(User::Table)
            .drop_column("_created_at")
            .drop_column("_updated_at")
            .to_owned();
        manager.alter_table(drop_column).await
    }
}
