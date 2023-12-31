// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub use sea_orm_migration::prelude::*;

mod m20230922_000001_create_user_table;
mod m20230922_000002_create_oauth_provider_table;
mod m20231014_000003_create_uploaded_file_table;
mod m20231112_000004_user_picture_foreign_key;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230922_000001_create_user_table::Migration),
            Box::new(m20230922_000002_create_oauth_provider_table::Migration),
            Box::new(m20231014_000003_create_uploaded_file_table::Migration),
            Box::new(m20231112_000004_user_picture_foreign_key::Migration),
        ]
    }
}
