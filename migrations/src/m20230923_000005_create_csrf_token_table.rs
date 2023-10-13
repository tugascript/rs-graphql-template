use sea_orm_migration::{
    prelude::*,
    sea_orm::{DbBackend, Schema},
};

use entities::csrf_token::{Column, Entity};

const CSRF_TOKEN_PROVIDER_TOKEN_IDX: &'static str = "csrf_token_provider_token_idx";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_table(
                schema
                    .create_table_from_entity(Entity)
                    .if_not_exists()
                    .index(
                        Index::create()
                            .if_not_exists()
                            .name(CSRF_TOKEN_PROVIDER_TOKEN_IDX)
                            .col(Column::Provider)
                            .col(Column::Token),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(Entity)
                    .name(CSRF_TOKEN_PROVIDER_TOKEN_IDX)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await?;
        Ok(())
    }
}