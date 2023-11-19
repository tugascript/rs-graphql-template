// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

use entities::{enums::role_enum::RoleEnum, user::Model};

use crate::{
    common::{ServiceError, SOMETHING_WENT_WRONG},
    config::SingleJwt,
};

use super::helpers::{access_token, email_token};

pub enum TokenType {
    Reset,
    Confirmation,
    Refresh,
}

impl TokenType {
    pub fn to_string(&self) -> String {
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
    refresh_name: Secret<String>,
    iss: Uuid,
}

impl Jwt {
    pub fn new(
        access_jwt: SingleJwt,
        refresh_jwt: SingleJwt,
        confirmation_jwt: SingleJwt,
        reset_jwt: SingleJwt,
        refresh_name: Secret<String>,
        api_id: Secret<String>,
    ) -> Self {
        Self {
            access: access_jwt,
            reset: reset_jwt,
            confirmation: confirmation_jwt,
            refresh: refresh_jwt,
            refresh_name,
            iss: Uuid::parse_str(&api_id.expose_secret()).unwrap(),
        }
    }

    pub fn generate_access_token(&self, user: &Model) -> Result<String, ServiceError> {
        access_token::Claims::create_token(
            user,
            &self.access.secret.expose_secret(),
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
            match token_type {
                TokenType::Confirmation => &self.confirmation.secret.expose_secret(),
                TokenType::Reset => &self.reset.secret.expose_secret(),
                TokenType::Refresh => &self.refresh.secret.expose_secret(),
            },
            self.confirmation.exp,
            &self.iss.to_string(),
            token_type.to_string(),
        )
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<(i32, RoleEnum), ServiceError> {
        match access_token::Claims::decode_token(&self.access.secret.expose_secret(), token) {
            Ok((id, role)) => Ok((id, role)),
            Err(e) => Err(ServiceError::unauthorized("Invalid token", Some(e))),
        }
    }

    pub fn verify_email_token(
        &self,
        token_type: TokenType,
        token: &str,
    ) -> Result<(i32, i16, String, i64), ServiceError> {
        match email_token::Claims::decode_token(
            match token_type {
                TokenType::Reset => &self.reset.secret.expose_secret(),
                TokenType::Confirmation => &self.confirmation.secret.expose_secret(),
                TokenType::Refresh => &self.refresh.secret.expose_secret(),
            },
            token,
        ) {
            Ok((id, version, token_id, exp)) => Ok((id, version, token_id, exp)),
            Err(e) => Err(ServiceError::unauthorized("Invalid token", Some(e))),
        }
    }

    pub fn get_refresh_name(&self) -> &str {
        &self.refresh_name.expose_secret()
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
