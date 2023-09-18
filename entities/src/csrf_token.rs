// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue, Condition};

use super::enums::OAuthProviderEnum;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "csrf_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", index)]
    pub token: String,
    #[sea_orm(column_type = "Text")]
    pub verifier: String,
    #[sea_orm(column_type = "String(Some(8))", index)]
    pub provider: OAuthProviderEnum,
    #[sea_orm(column_type = "Timestamp", index)]
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C: ConnectionTrait>(mut self, _: &C, insert: bool) -> Result<Self, DbErr> {
        let current_time = Utc::now().naive_utc();
        if insert {
            self.created_at = ActiveValue::Set(current_time);
        }
        Ok(self)
    }
}

impl Entity {
    pub fn find_token(provider: OAuthProviderEnum, token: &str) -> Select<Entity> {
        Entity::find().filter(
            Condition::all()
                .add(Column::Provider.eq(provider))
                .add(Column::Token.eq(token)),
        )
    }
}
