use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let col = Table::create()
            .table(Session::Table)
            .if_not_exists()
            .col(text(Session::Id).primary_key())
            .col(json_binary(Session::Data))
            .col(timestamp_with_time_zone(Session::ExpiresAt))
            .col(timestamp_with_time_zone(Session::RefreshedAt).default(Expr::current_timestamp()))
            .col(timestamp_with_time_zone(Session::CreatedAt).default(Expr::current_timestamp()))
            .to_owned();
        manager.create_table(col).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Session {
    Table,
    Id,
    Data,
    ExpiresAt,
    #[sea_orm(iden = "_refreshed_at")]
    RefreshedAt,
    #[sea_orm(iden = "_created_at")]
    CreatedAt,
}
