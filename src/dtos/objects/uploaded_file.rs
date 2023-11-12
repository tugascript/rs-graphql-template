// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};

use entities::uploaded_file::Model;

use crate::common::{InternalCause, ServiceError, NOT_FOUND};
use crate::data_loaders::{SeaOrmLoader, UserId};
use crate::dtos::objects::User;

#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct UploadedFile {
    pub id: String,
    pub url: String,
    #[graphql(skip)]
    pub user_id: i32,
    pub extension: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Model> for UploadedFile {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            url: value.url,
            user_id: value.user_id,
            extension: value.extension,
            created_at: value.created_at.timestamp(),
            updated_at: value.updated_at.timestamp(),
        }
    }
}

#[ComplexObject]
impl UploadedFile {
    pub async fn user(&self, ctx: &Context<'_>) -> Result<User> {
        if let Some(user) = ctx
            .data::<DataLoader<SeaOrmLoader>>()?
            .load_one(UserId(self.user_id))
            .await?
        {
            return Ok(user);
        }

        Err(ServiceError::not_found(
            NOT_FOUND,
            Some(InternalCause::new("User not found on dataloader")),
        )
        .into())
    }
}
