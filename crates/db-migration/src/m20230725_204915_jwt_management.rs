use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jwt::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Jwt::Jwt).string().not_null().primary_key())
                    .col(ColumnDef::new(Jwt::Name).string().not_null())
                    .col(ColumnDef::new(Jwt::Valid).boolean().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Jwt::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Jwt {
    Table,
    Jwt,
    ClientId,
    Name,
    WebhookEndpoint,
    Valid,
}
