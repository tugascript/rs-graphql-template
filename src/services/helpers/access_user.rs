// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use entities::enums::role_enum::RoleEnum;

use crate::{providers::Jwt, startup::AuthTokens};

pub struct AccessUser {
    pub id: i32,
    pub role: RoleEnum,
}

impl AccessUser {
    pub fn get_access_user<'a>(
        auth_tokens: &'a AuthTokens,
        jwt: &'a Jwt,
    ) -> Result<AccessUser, &'a str> {
        let token = auth_tokens.access_token.as_ref().ok_or("Unauthorized")?;
        let (id, role) = jwt.verify_access_token(token)?;
        Ok(Self { id, role })
    }
}
