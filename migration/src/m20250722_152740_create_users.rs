use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub(crate) enum User {
    Table,
    Id,
    Sid,
    Name,
    Email,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(pk_uuid(User::Id).default(Expr::cust("uuid_generate_v1()")))
            .col(text(User::Sid).not_null().unique_key())
            .col(text(User::Email).not_null().unique_key())
            .col(text(User::Name).not_null())
            .to_owned();
        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
