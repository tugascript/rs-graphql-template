// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, errors::Result, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::user::Model;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailToken {
    id: i32,
    version: i16,
    token_id: String,
}

impl From<&Model> for EmailToken {
    fn from(model: &Model) -> Self {
        Self {
            id: model.id.to_owned(),
            version: model.version.to_owned(),
            token_id: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
    user: EmailToken,
}

impl Claims {
    pub fn create_token(
        user: &Model,
        secret: &str,
        exp: i64,
        iss: &str,
        sub: String,
    ) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub,
            iss: iss.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(exp)).timestamp(),
            user: EmailToken::from(user),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }

    pub fn decode_token(secret: &str, token: &str) -> Result<(i32, i16, String, i64)> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok((
            token_data.claims.user.id,
            token_data.claims.user.version,
            token_data.claims.user.token_id,
            token_data.claims.exp,
        ))
    }
}
