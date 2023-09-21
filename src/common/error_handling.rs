// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{error, http::StatusCode, HttpResponse};
use async_graphql::{Error, ErrorExtensions};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum ServiceError {
    InternalServerError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Forbidden(String),
}

#[derive(Debug)]
pub enum GraphQLError {
    InternalServerError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Forbidden(String),
}

impl From<ServiceError> for GraphQLError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::InternalServerError(message) => {
                GraphQLError::InternalServerError(message)
            }
            ServiceError::BadRequest(message) => GraphQLError::BadRequest(message),
            ServiceError::Unauthorized(message) => GraphQLError::Unauthorized(message),
            ServiceError::NotFound(message) => GraphQLError::NotFound(message),
            ServiceError::Forbidden(message) => GraphQLError::Forbidden(message),
        }
    }
}

impl error::ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ServiceError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServiceError::NotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::Forbidden(_) => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match *self {
            ServiceError::InternalServerError(ref message) => {
                HttpResponse::InternalServerError().json(message)
            }
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::Unauthorized(ref message) => HttpResponse::Unauthorized().json(message),
            ServiceError::NotFound(ref message) => HttpResponse::NotFound().json(message),
            ServiceError::Forbidden(ref message) => HttpResponse::Forbidden().json(message),
        }
    }
}

impl Into<Error> for GraphQLError {
    fn into(self) -> Error {
        match self {
            GraphQLError::InternalServerError(message) => {
                Error::new(message).extend_with(|_, e| {
                    e.set("type", "Internal Server Error");
                    e.set("code", "500");
                })
            }
            GraphQLError::BadRequest(message) => Error::new(message).extend_with(|_, e| {
                e.set("type", "Bad Request");
                e.set("code", "400");
            }),
            GraphQLError::Unauthorized(message) => Error::new(message).extend_with(|_, e| {
                e.set("type", "Unauthorized");
                e.set("code", "401");
            }),
            GraphQLError::NotFound(message) => Error::new(message).extend_with(|_, e| {
                e.set("type", "Not Found");
                e.set("code", "404");
            }),
            GraphQLError::Forbidden(message) => Error::new(message).extend_with(|_, e| {
                e.set("type", "Forbidden");
                e.set("code", "403");
            }),
        }
    }
}
