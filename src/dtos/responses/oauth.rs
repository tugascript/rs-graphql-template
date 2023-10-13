// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::common::ServiceError;

pub struct UserInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub date_of_birth: String,
    pub picture: Option<String>,
}

impl TryFrom<GoogleUserInfoResponse> for UserInfo {
    type Error = ServiceError;

    fn try_from(value: GoogleUserInfoResponse) -> Result<Self, Self::Error> {
        let first_name = value.given_name.ok_or_else(|| {
            ServiceError::internal_server_error::<Error>("Missing given name", None)
        })?;
        let last_name = value.family_name.ok_or_else(|| {
            ServiceError::internal_server_error::<Error>("Missing family name", None)
        })?;
        let email = value
            .email
            .ok_or_else(|| ServiceError::internal_server_error::<Error>("Missing email", None))?;
        let date_of_birth = value.birthdate.ok_or_else(|| {
            ServiceError::internal_server_error::<Error>("Missing birthdate", None)
        })?;

        Ok(Self {
            first_name,
            last_name,
            email,
            date_of_birth,
            picture: value.picture,
        })
    }
}

impl TryFrom<FacebookUserInfoResponse> for UserInfo {
    type Error = ServiceError;

    fn try_from(value: FacebookUserInfoResponse) -> Result<Self, Self::Error> {
        let first_name = value.first_name.ok_or_else(|| {
            ServiceError::internal_server_error::<Error>("Missing first name", None)
        })?;
        let last_name = value.last_name.ok_or_else(|| {
            ServiceError::internal_server_error::<Error>("Missing last name", None)
        })?;
        let email = value
            .email
            .ok_or_else(|| ServiceError::internal_server_error::<Error>("Missing email", None))?;
        let birth_date = value.birthday.ok_or_else(|| {
            ServiceError::internal_server_error::<Error>("Missing birth date", None)
        })?;

        Ok(Self {
            first_name,
            last_name,
            email,
            date_of_birth: birth_date,
            picture: value.picture.and_then(|p| p.data).and_then(|d| d.url),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfoResponse {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub locale: Option<String>,
    pub birthdate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookPictureInfo {
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub url: Option<String>,
    pub is_silhouette: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookPictureData {
    pub data: Option<FacebookPictureInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookUserInfoResponse {
    pub id: String,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub birthday: Option<String>,
    pub picture: Option<FacebookPictureData>,
    pub gender: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OAuthUserInfo {
    Google(GoogleUserInfoResponse),
    Facebook(FacebookUserInfoResponse),
}

impl TryInto<UserInfo> for OAuthUserInfo {
    type Error = ServiceError;

    fn try_into(self) -> Result<UserInfo, Self::Error> {
        match self {
            OAuthUserInfo::Google(google) => google.try_into(),
            OAuthUserInfo::Facebook(facebook) => facebook.try_into(),
        }
    }
}
