// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::{Duration, Utc};
use entities::{enums::role_enum::RoleEnum, user::Model};
use jsonwebtoken::{decode, encode, errors::Result, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct AccessToken {
    id: i32,
    role: RoleEnum,
}

impl From<&Model> for AccessToken {
    fn from(model: &Model) -> Self {
        Self {
            id: model.id.to_owned(),
            role: model.role.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    jti: String,
    iat: i64,
    exp: i64,
    user: AccessToken,
}

impl Claims {
    pub fn create_token(user: &Model, secret: &str, exp: i64, iss: &str) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: "access".to_string(),
            iss: iss.to_string(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            exp: (now + Duration::seconds(exp)).timestamp(),
            user: AccessToken::from(user),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }

    pub fn decode_token(secret: &str, token: &str) -> Result<(i32, RoleEnum)> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok((token_data.claims.user.id, token_data.claims.user.role))
    }
}
