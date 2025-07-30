use sea_orm_migration::{
    prelude::*,
    schema::{timestamp_with_time_zone as timestamp_tz, *},
};

use crate::m20250722_152740_create_users::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Album {
    Table,
    Id,
    Title,
    Artist,
    Year,
    #[sea_orm(iden = "_created_at")]
    CreatedAt,
    #[sea_orm(iden = "_updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "_created_by")]
    CreatedBy,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let pk = uuid(Album::Id)
            .primary_key()
            .default(Expr::cust("uuid_generate_v1()"))
            .to_owned();
        let table = Table::create()
            .table(Album::Table)
            .if_not_exists()
            .col(pk)
            .col(text(Album::Title))
            .col(text(Album::Artist))
            .col(integer(Album::Year))
            .col(timestamp_tz(Album::CreatedAt).default(Expr::current_timestamp()))
            .col(timestamp_tz(Album::UpdatedAt).default(Expr::current_timestamp()))
            .col(uuid(Album::CreatedBy))
            .to_owned();
        manager.create_table(table).await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_album_user")
                    .from(Album::Table, Album::CreatedBy)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Album::Table).to_owned())
            .await
    }
}
