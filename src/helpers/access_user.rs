// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::{Context, Error, Result};

use entities::enums::RoleEnum;

use crate::common::{AuthTokens, InternalCause, ServiceError, UNAUTHORIZED};
use crate::providers::Jwt;

#[derive(Debug, Clone)]
pub struct AccessUser {
    pub id: i32,
    pub role: RoleEnum,
}

impl AccessUser {
    pub fn new(id: i32, role: RoleEnum) -> Self {
        Self { id, role }
    }

    pub fn get_access_user(ctx: &Context<'_>) -> Result<Self> {
        let tokens = ctx.data::<AuthTokens>()?;
        let access_token = tokens
            .access_token
            .as_ref()
            .ok_or(ServiceError::unauthorized(
                UNAUTHORIZED,
                Some(InternalCause::new("No access token in header")),
            ))?;
        let jwt = ctx.data::<Jwt>()?;

        match jwt.verify_access_token(access_token) {
            Ok((id, role)) => Ok(Self::new(id, role)),
            Err(_) => Err(Error::new("Unauthorized")),
        }
    }
}
