// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::future::{ready, Ready};

use actix_web::{cookie::Cookie, dev::Payload, http::header::HeaderMap, FromRequest, HttpRequest};

use crate::common::ServiceError;

fn get_access_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let auth_header = match headers.get("Authorization") {
        Some(ah) => ah,
        None => return None,
    };
    let auth_header = match auth_header.to_str() {
        Ok(ah) => ah,
        Err(_) => return None,
    };

    if auth_header.is_empty() || !auth_header.starts_with("Bearer ") {
        return None;
    }

    let token = match auth_header.split_whitespace().last() {
        Some(t) => t,
        None => return None,
    };

    if token.is_empty() {
        return None;
    }

    Some(token.to_string())
}

fn get_refresh_token_from_cookie(cookie: Option<Cookie>) -> Option<String> {
    if let Some(cookie) = cookie {
        if cookie.value().is_empty() {
            return None;
        }

        Some(cookie.value().to_string())
    } else {
        None
    }
}

pub struct AuthTokens {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl AuthTokens {
    pub fn new(request: &HttpRequest) -> Self {
        Self {
            access_token: get_access_token_from_headers(request.headers()),
            refresh_token: get_refresh_token_from_cookie(request.cookie("refresh_token")),
        }
    }
}

impl FromRequest for AuthTokens {
    type Error = ServiceError;
    type Future = Ready<Result<AuthTokens, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(Self::new(request)))
    }
}
