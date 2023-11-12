use sea_orm_migration::prelude::*;

use entities::{uploaded_file, user};

const FK_NAME: &'static str = "uploaded_file_user_id_fkey";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_NAME)
                    .from_tbl(user::Entity)
                    .from_col(user::Column::Picture)
                    .to_tbl(uploaded_file::Entity)
                    .to_col(uploaded_file::Column::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(ForeignKey::drop().name(FK_NAME).to_owned())
            .await
    }
}
