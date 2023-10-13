// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use uuid::Uuid;

use entities::{enums::role_enum::RoleEnum, user::Model};

use crate::common::{ServiceError, SOMETHING_WENT_WRONG};

use super::helpers::{access_token, email_token};

#[derive(Clone, Debug)]
pub struct SingleJwt {
    pub secret: String,
    pub exp: i64,
}

pub enum TokenType {
    Reset,
    Confirmation,
    Refresh,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::Reset => "reset".to_string(),
            TokenType::Confirmation => "confirmation".to_string(),
            TokenType::Refresh => "refresh".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Jwt {
    access: SingleJwt,
    reset: SingleJwt,
    confirmation: SingleJwt,
    refresh: SingleJwt,
    refresh_name: String,
    iss: Uuid,
}

impl Jwt {
    pub fn new() -> Self {
        let access_secret = env::var("ACCESS_SECRET").unwrap();
        let access_time = env::var("ACCESS_TIME").unwrap().parse::<i64>().unwrap();
        let reset_secret = env::var("RESET_SECRET").unwrap();
        let reset_time = env::var("RESET_TIME").unwrap().parse::<i64>().unwrap();
        let confirmation_secret = env::var("CONFIRMATION_SECRET").unwrap();
        let confirmation_time = env::var("CONFIRMATION_TIME")
            .unwrap()
            .parse::<i64>()
            .unwrap();
        let refresh_secret = env::var("REFRESH_SECRET").unwrap();
        let refresh_time = env::var("REFRESH_TIME").unwrap().parse::<i64>().unwrap();
        let refresh_name = env::var("REFRESH_NAME").unwrap();
        let api_id = env::var("API_ID").unwrap();
        let iss = Uuid::parse_str(&api_id).unwrap();

        Self {
            access: SingleJwt {
                secret: access_secret,
                exp: access_time,
            },
            reset: SingleJwt {
                secret: reset_secret,
                exp: reset_time,
            },
            confirmation: SingleJwt {
                secret: confirmation_secret,
                exp: confirmation_time,
            },
            refresh: SingleJwt {
                secret: refresh_secret,
                exp: refresh_time,
            },
            refresh_name,
            iss,
        }
    }

    pub fn generate_access_token(&self, user: &Model) -> Result<String, ServiceError> {
        access_token::Claims::create_token(
            user,
            &self.access.secret,
            self.access.exp,
            &self.iss.to_string(),
        )
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))
    }

    pub fn generate_email_token(
        &self,
        token_type: TokenType,
        user: &Model,
    ) -> Result<String, ServiceError> {
        email_token::Claims::create_token(
            user,
            &self.confirmation.secret,
            self.confirmation.exp,
            &self.iss.to_string(),
            token_type,
        )
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<(i32, RoleEnum), ServiceError> {
        match access_token::Claims::decode_token(&self.access.secret, token) {
            Ok((id, role)) => Ok((id, role)),
            Err(e) => Err(ServiceError::unauthorized("Invalid token", Some(e))),
        }
    }

    pub fn verify_email_token(
        &self,
        token_type: TokenType,
        token: &str,
    ) -> Result<(i32, i16, &str, i64), ServiceError> {
        match email_token::Claims::decode_token(
            match token_type {
                TokenType::Reset => &self.reset.secret,
                TokenType::Confirmation => &self.confirmation.secret,
                TokenType::Refresh => &self.refresh.secret,
            },
            token,
        ) {
            Ok((id, version, token_id, exp)) => Ok((id, version, token_id, exp)),
            Err(e) => Err(ServiceError::unauthorized("Invalid token", Some(e))),
        }
    }

    pub fn get_refresh_name(&self) -> &str {
        &self.refresh_name
    }

    pub fn get_access_token_time(&self) -> i64 {
        self.access.exp
    }

    pub fn get_email_token_time(&self, token_type: TokenType) -> i64 {
        match token_type {
            TokenType::Reset => self.reset.exp,
            TokenType::Confirmation => self.confirmation.exp,
            TokenType::Refresh => self.refresh.exp,
        }
    }

    pub fn generate_auth_tokens(&self, user: &Model) -> Result<(String, String), ServiceError> {
        tracing::trace_span!("Generating authentication tokens", id = %user.id);
        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_email_token(TokenType::Refresh, user)?;
        Ok((access_token, refresh_token))
    }
}
