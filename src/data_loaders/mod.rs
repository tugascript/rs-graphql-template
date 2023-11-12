// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use async_graphql::dataloader::*;
use async_graphql::*;

use file_loader::load_files;
pub use file_loader::FileId;
use user_loader::load_users;
pub use user_loader::UserId;

use crate::dtos::objects::{UploadedFile, User};
use crate::providers::Database;

pub mod file_loader;
pub mod user_loader;

pub struct SeaOrmLoader {
    db: Database,
}

impl SeaOrmLoader {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait::async_trait]
impl Loader<FileId> for SeaOrmLoader {
    type Value = UploadedFile;
    type Error = Error;

    async fn load(&self, keys: &[FileId]) -> Result<HashMap<FileId, Self::Value>, Self::Error> {
        load_files(self.db.get_connection(), keys).await
    }
}

#[async_trait::async_trait]
impl Loader<UserId> for SeaOrmLoader {
    type Value = User;
    type Error = Error;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, Self::Value>, Self::Error> {
        load_users(self.db.get_connection(), keys).await
    }
}
