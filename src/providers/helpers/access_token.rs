// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::{Duration, Utc};
use entities::{enums::role_enum::RoleEnum, user::Model};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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
    iat: i64,
    exp: i64,
    user: AccessToken,
}

impl Claims {
    pub fn create_token(user: &Model, secret: &str, exp: i64, iss: &str) -> Result<String, String> {
        let now = Utc::now();
        let claims = Claims {
            sub: "access".to_string(),
            iss: iss.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(exp)).timestamp(),
            user: AccessToken::from(user),
        };

        if let Ok(token) = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        ) {
            Ok(token)
        } else {
            Err("Could not create the access token".to_string())
        }
    }

    pub fn decode_token(secret: &str, token: &str) -> Result<(i32, RoleEnum), String> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        match claims {
            Ok(s) => Ok((s.claims.user.id, s.claims.user.role)),
            Err(_) => Err("Invalid token".to_string()),
        }
    }
}
