// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

use entities::{enums::role_enum::RoleEnum, user::Model};

use crate::common::{ServiceError, SOMETHING_WENT_WRONG};

use super::{
    helpers::{access_token, email_token},
    Environment,
};

#[derive(Clone, Debug)]
struct SingleJwt {
    secret: Secret<String>,
    exp: i64,
}

impl SingleJwt {
    fn new(secret: String, exp: i64) -> Self {
        Self {
            secret: Secret::new(secret),
            exp,
        }
    }
}

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
    pub fn new(environment: &Environment, api_id: &str) -> Self {
        let jwt_access_secret = env::var("ACCESS_SECRET").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => {
                panic!("Missing the JWT_ACCESS_SECRET environment variable.")
            }
        });
        let jwt_refresh_secret = env::var("REFRESH_SECRET").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => {
                panic!("Missing the JWT_REFRESH_SECRET environment variable.")
            }
        });
        let jwt_confirmation_secret =
            env::var("CONFIRMATION_SECRET").unwrap_or_else(|_| match environment {
                Environment::Development => Uuid::new_v4().to_string(),
                Environment::Production => {
                    panic!("Missing the JWT_CONFIRMATION_SECRET environment variable.")
                }
            });
        let jwt_reset_secret = env::var("RESET_SECRET").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => panic!("Missing the JWT_RESET_SECRET environment variable."),
        });
        let jwt_access_expiration = env::var("ACCESS_EXPIRATION")
            .unwrap_or_else(|_| "600".to_string())
            .parse::<i64>()
            .unwrap_or(600);
        let jwt_refresh_expiration = env::var("REFRESH_EXPIRATION")
            .unwrap_or_else(|_| "259200".to_string())
            .parse::<i64>()
            .unwrap_or(259200);
        let jwt_confirmation_expiration = env::var("CONFIRMATION_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<i64>()
            .unwrap_or(86400);
        let jwt_reset_expiration = env::var("RESET_EXPIRATION")
            .unwrap_or_else(|_| "1800".to_string())
            .parse::<i64>()
            .unwrap_or(1800);
        let refresh_name = env::var("REFRESH_NAME").unwrap_or_else(|_| match environment {
            Environment::Development => "refresh".to_string(),
            Environment::Production => panic!("Missing the REFRESH_NAME environment variable."),
        });

        Self {
            access: SingleJwt::new(jwt_access_secret, jwt_access_expiration),
            reset: SingleJwt::new(jwt_reset_secret, jwt_reset_expiration),
            confirmation: SingleJwt::new(jwt_confirmation_secret, jwt_confirmation_expiration),
            refresh: SingleJwt::new(jwt_refresh_secret, jwt_refresh_expiration),
            refresh_name: Secret::new(refresh_name),
            iss: Uuid::parse_str(api_id).unwrap(),
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
