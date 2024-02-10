// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use async_graphql::{Error, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use entities::user::{Column, Entity};

use crate::common::{InternalCause, ServiceError};
use crate::dtos::objects::User;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct UserId(pub i32);

pub async fn load_users(
    connection: &DatabaseConnection,
    keys: &[UserId],
) -> Result<HashMap<UserId, User>> {
    let ids = keys.iter().map(|key| key.0).collect::<Vec<i32>>();
    let users = Entity::find()
        .filter(Column::Id.is_in(ids))
        .all(connection)
        .await
        .map_err(|_| Error::from("Error loading users"))?;

    if users.len() != keys.len() {
        return Err(ServiceError::not_found(
            "User not found",
            Some(InternalCause::new("Keys and fetched users do not match")),
        )
        .into());
    }

    Ok(users
        .into_iter()
        .map(|user| (UserId(user.id), user.into()))
        .collect())
}
