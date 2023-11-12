// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use regex::{Regex, RegexBuilder};

use super::error_handling::{ServiceError, INTERNAL_SERVER_ERROR};

pub fn email_regex() -> Result<Regex, ServiceError> {
    match Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]{2,}$") {
        Ok(value) => Ok(value),
        Err(e) => Err(ServiceError::internal_server_error(
            INTERNAL_SERVER_ERROR,
            Some(e),
        )),
    }
}

pub fn name_regex() -> Result<Regex, ServiceError> {
    match RegexBuilder::new(r"(^[\p{L}0-9'\.\s]*$)")
        .unicode(true)
        .build()
    {
        Ok(value) => Ok(value),
        Err(e) => Err(ServiceError::internal_server_error(
            INTERNAL_SERVER_ERROR,
            Some(e),
        )),
    }
}

pub fn jwt_regex() -> Result<Regex, ServiceError> {
    match Regex::new(r"^[A-Za-z0-9-_=]+\.[A-Za-z0-9-_=]+\.?[A-Za-z0-9-_.+/=]*$") {
        Ok(value) => Ok(value),
        Err(e) => Err(ServiceError::internal_server_error(
            INTERNAL_SERVER_ERROR,
            Some(e),
        )),
    }
}

pub fn new_line_regex() -> Result<Regex, ServiceError> {
    match Regex::new(r"\r\n|\r|\n") {
        Ok(value) => Ok(value),
        Err(e) => Err(ServiceError::internal_server_error(
            INTERNAL_SERVER_ERROR,
            Some(e),
        )),
    }
}

pub fn multi_spaces_regex() -> Result<Regex, ServiceError> {
    match Regex::new(r"\s\s+") {
        Ok(value) => Ok(value),
        Err(e) => Err(ServiceError::internal_server_error(
            INTERNAL_SERVER_ERROR,
            Some(e),
        )),
    }
}
