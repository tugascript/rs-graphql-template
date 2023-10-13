// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::Enum;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Enum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(Some(8))")]
pub enum OAuthProviderEnum {
    #[graphql(name = "LOCAL")]
    #[sea_orm(string_value = "LOCAL")]
    Local,
    #[graphql(name = "GOOGLE")]
    #[sea_orm(string_value = "GOOGLE")]
    Google,
    #[graphql(name = "FACEBOOK")]
    #[sea_orm(string_value = "FACEBOOK")]
    Facebook,
}

impl OAuthProviderEnum {
    pub fn to_str<'a>(&self) -> &'a str {
        match self {
            OAuthProviderEnum::Local => "LOCAL",
            OAuthProviderEnum::Google => "GOOGLE",
            OAuthProviderEnum::Facebook => "FACEBOOK",
        }
    }
}
