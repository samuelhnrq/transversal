use sea_orm_migration::{
    prelude::*,
    schema::{timestamp_with_time_zone as timestamp_tz, *},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub(crate) enum User {
    Table,
    Id,
    Sid,
    Name,
    Email,
    #[sea_orm(iden = "_created_at")]
    CreatedAt,
    #[sea_orm(iden = "_updated_at")]
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(
                uuid(User::Id)
                    .primary_key()
                    .default(Expr::cust("uuid_generate_v1()")),
            )
            .col(text(User::Sid).not_null().unique_key())
            .col(text(User::Email).not_null().unique_key())
            .col(text(User::Name).not_null())
            .col(timestamp_tz(User::CreatedAt).default(Expr::current_timestamp()))
            .col(timestamp_tz(User::UpdatedAt).default(Expr::current_timestamp()))
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
