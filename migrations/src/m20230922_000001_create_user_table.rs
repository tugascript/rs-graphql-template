// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use sea_orm_migration::{
    prelude::*,
    sea_orm::{DbBackend, Schema},
};

use entities::user::{Column, Entity};

const USER_USERNAME_IDX: &'static str = "user_username_idx";
const USER_ID_VERSION_IDX: &'static str = "user_id_version_idx";

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
                            .name(USER_USERNAME_IDX)
                            .unique()
                            .col(Column::Username),
                    )
                    .index(
                        Index::create()
                            .if_not_exists()
                            .name(USER_ID_VERSION_IDX)
                            .unique()
                            .col(Column::Id)
                            .col(Column::Version),
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
                    .name(USER_USERNAME_IDX)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(Entity)
                    .name(USER_ID_VERSION_IDX)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await?;
        Ok(())
    }
}
