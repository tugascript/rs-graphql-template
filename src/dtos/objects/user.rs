// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Result, SimpleObject};

use entities::enums::RoleEnum;
use entities::user::Model;

use crate::data_loaders::{FileId, SeaOrmLoader};
use crate::guards::AuthGuard;
use crate::helpers::AccessUser;

use super::UploadedFile;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct User {
    pub id: i32,
    pub name: String,
    #[graphql(skip)]
    pub email: String,
    #[graphql(skip)]
    pub picture: Option<String>,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub role: RoleEnum,
    pub create_at: i64,
    pub updated_at: i64,
}

impl From<Model> for User {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.full_name(),
            email: value.email,
            picture: value.picture,
            username: value.username,
            first_name: value.first_name,
            last_name: value.last_name,
            date_of_birth: value.date_of_birth.to_string(),
            role: value.role,
            create_at: value.created_at.timestamp(),
            updated_at: value.updated_at.timestamp(),
        }
    }
}

#[ComplexObject]
impl User {
    #[graphql(guard = "AuthGuard")]
    pub async fn email(&self, ctx: &Context<'_>) -> Result<Option<&str>> {
        let user = AccessUser::get_access_user(ctx)?;

        if user.id == self.id {
            Ok(Some(&self.email))
        } else {
            Ok(None)
        }
    }

    pub async fn picture(&self, ctx: &Context<'_>) -> Result<Option<UploadedFile>> {
        if let Some(picture) = &self.picture {
            ctx.data::<DataLoader<SeaOrmLoader>>()?
                .load_one(FileId(picture.to_owned()))
                .await
        } else {
            Ok(None)
        }
    }
}
