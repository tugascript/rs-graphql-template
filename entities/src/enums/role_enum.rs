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
#[sea_orm(rs_type = "String", db_type = "String(Some(5))")]
pub enum RoleEnum {
    #[graphql(name = "USER")]
    #[sea_orm(string_value = "USER")]
    User,
    #[graphql(name = "STAFF")]
    #[sea_orm(string_value = "STAFF")]
    Staff,
    #[graphql(name = "ADMIN")]
    #[sea_orm(string_value = "ADMIN")]
    Admin,
}
