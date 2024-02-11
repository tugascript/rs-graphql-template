// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::HttpRequest;
use entities::enums::RoleEnum;

use crate::common::AuthTokens;
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

    pub fn from_request(jwt: &Jwt, req: &HttpRequest) -> Option<Self> {
        let tokens = AuthTokens::new(req);

        if let Some(access_token) = tokens.access_token {
            match jwt.verify_access_token(&access_token) {
                Ok((id, role)) => Some(Self::new(id, role)),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
