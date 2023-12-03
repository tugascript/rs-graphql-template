// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{error, http::StatusCode, HttpResponse};
use async_graphql::{Error, ErrorExtensions};
use derive_more::Display;
use sea_orm::DbErr;

#[derive(Debug, Display)]
pub struct InternalCause(String);

impl InternalCause {
    pub fn new(cause: &str) -> Self {
        Self(cause.to_string())
    }
}

#[derive(Debug, Display)]
pub enum ServiceError {
    InternalServerError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Forbidden(String),
    Conflict(String),
}

pub const INTERNAL_SERVER_ERROR: &'static str = "Internal Server Error";
pub const INTERNAL_SERVER_ERROR_STATUS_CODE: u16 = 500;
pub const BAD_REQUEST: &'static str = "Bad Request";
pub const BAD_REQUEST_STATUS_CODE: u16 = 400;
pub const UNAUTHORIZED: &'static str = "Unauthorized";
pub const UNAUTHORIZED_STATUS_CODE: u16 = 401;
pub const NOT_FOUND: &'static str = "Not Found";
pub const NOT_FOUND_STATUS_CODE: u16 = 404;
pub const FORBIDDEN: &'static str = "Forbidden";
pub const FORBIDDEN_STATUS_CODE: u16 = 403;
pub const CONFLICT: &'static str = "Conflict";
pub const CONFLICT_STATUS_CODE: u16 = 409;
pub const SOMETHING_WENT_WRONG: &'static str = "Something went wrong";
pub const INVALID_CREDENTIALS: &'static str = "Invalid credentials";

impl ServiceError {
    pub fn to_str_name(&self) -> &'static str {
        match self {
            ServiceError::InternalServerError(_) => INTERNAL_SERVER_ERROR,
            ServiceError::BadRequest(_) => BAD_REQUEST,
            ServiceError::Unauthorized(_) => UNAUTHORIZED,
            ServiceError::NotFound(_) => NOT_FOUND,
            ServiceError::Forbidden(_) => FORBIDDEN,
            ServiceError::Conflict(_) => CONFLICT,
        }
    }

    pub fn get_status_code(&self) -> u16 {
        match self {
            ServiceError::InternalServerError(_) => INTERNAL_SERVER_ERROR_STATUS_CODE,
            ServiceError::BadRequest(_) => BAD_REQUEST_STATUS_CODE,
            ServiceError::Unauthorized(_) => UNAUTHORIZED_STATUS_CODE,
            ServiceError::NotFound(_) => NOT_FOUND_STATUS_CODE,
            ServiceError::Forbidden(_) => FORBIDDEN_STATUS_CODE,
            ServiceError::Conflict(_) => CONFLICT_STATUS_CODE,
        }
    }

    pub fn internal_server_error<T: std::fmt::Display + std::fmt::Debug>(
        message: &str,
        cause: Option<T>,
    ) -> Self {
        let error = Self::InternalServerError(message.to_string());

        if let Some(cause) = cause {
            tracing::error!(INTERNAL_SERVER_ERROR, %message, %cause);
        } else {
            tracing::error!(INTERNAL_SERVER_ERROR, %message);
        }

        error
    }

    pub fn bad_request<T: std::fmt::Display + std::fmt::Debug>(
        message: &str,
        cause: Option<T>,
    ) -> Self {
        let error = Self::BadRequest(message.to_string());

        if let Some(cause) = cause {
            tracing::error!(BAD_REQUEST, %message, %cause);
        } else {
            tracing::error!(BAD_REQUEST, %message);
        }

        error
    }

    pub fn unauthorized<T: std::fmt::Display + std::fmt::Debug>(
        message: &str,
        cause: Option<T>,
    ) -> Self {
        let error = Self::Unauthorized(message.to_string());

        if let Some(cause) = cause {
            tracing::error!(UNAUTHORIZED, %message, %cause);
        } else {
            tracing::error!(UNAUTHORIZED, %message);
        }

        error
    }

    pub fn not_found<T: std::fmt::Display + std::fmt::Debug>(
        message: &str,
        cause: Option<T>,
    ) -> Self {
        let error = Self::NotFound(message.to_string());

        if let Some(cause) = cause {
            tracing::error!(NOT_FOUND, %message, %cause);
        } else {
            tracing::error!(NOT_FOUND, %message);
        }

        error
    }

    pub fn forbidden<T: std::fmt::Display + std::fmt::Debug>(
        message: &str,
        cause: Option<T>,
    ) -> Self {
        let error = Self::Forbidden(message.to_string());

        if let Some(cause) = cause {
            tracing::error!(FORBIDDEN, %message, %cause);
        } else {
            tracing::error!(FORBIDDEN, %message);
        }

        error
    }

    pub fn conflict<T: std::fmt::Display + std::fmt::Debug>(
        message: &str,
        cause: Option<T>,
    ) -> Self {
        let error = Self::Conflict(message.to_string());

        if let Some(cause) = cause {
            tracing::error!(CONFLICT, %message, %cause);
        } else {
            tracing::error!(CONFLICT, %message);
        }

        error
    }
}

impl From<DbErr> for ServiceError {
    fn from(value: DbErr) -> Self {
        match value {
            DbErr::AttrNotSet(err) => {
                tracing::error!("Database attribute not set error: {}", err);
                Self::BadRequest("Missing fields".to_string())
            }
            DbErr::Conn(err) => {
                tracing::error!("Database connection error: {:?}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::Type(err) => {
                tracing::error!("Database parsing error: {}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::RecordNotInserted => {
                tracing::error!("Database record not inserted error");
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::RecordNotUpdated => {
                tracing::error!("Database record not updated error");
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::RecordNotFound(err) => {
                tracing::error!("Database record not found error: {}", err);
                Self::NotFound("Entity not found".to_string())
            }
            DbErr::ConnectionAcquire(err) => {
                tracing::error!("Database connection acquire error: {:?}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::Exec(err) => {
                tracing::error!("Database execution error: {:?}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::Query(err) => {
                tracing::error!("Database query error: {:?}", err);
                println!("Database query error: {:?}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::Json(err) => {
                tracing::error!("Database json error: {}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::UnpackInsertId => {
                tracing::error!("Database unpack insert id error");
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::UpdateGetPrimaryKey => {
                tracing::error!("Database update get primary key error");
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            DbErr::ConvertFromU64(err) => {
                tracing::error!("Database convert from u64 error: {}", err);
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
            _ => {
                tracing::error!("Database unknown error");
                Self::InternalServerError(SOMETHING_WENT_WRONG.to_string())
            }
        }
    }
}

#[derive(Debug)]
pub enum GraphQLError {
    InternalServerError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Forbidden(String),
    Conflict(String),
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
            ServiceError::Conflict(message) => GraphQLError::Conflict(message),
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
            ServiceError::Conflict(_) => StatusCode::CONFLICT,
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
            ServiceError::Conflict(ref message) => HttpResponse::Conflict().json(message),
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
            GraphQLError::Conflict(message) => Error::new(message).extend_with(|_, e| {
                e.set("type", "Conflict");
                e.set("code", "409");
            }),
        }
    }
}
