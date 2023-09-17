// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue, Condition};

use crate::enums::oauth_provider_enum::OAuthProviderEnum;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "login_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(Some(200))")]
    pub user_email: String,
    #[sea_orm(column_type = "String(Some(8))")]
    pub provider: OAuthProviderEnum,
    #[sea_orm(column_type = "Boolean", default = true)]
    pub two_factor: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserEmail",
        to = "super::user::Column::Email"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C: ConnectionTrait>(mut self, _: &C, insert: bool) -> Result<Self, DbErr> {
        let current_time = Utc::now().naive_utc();
        self.updated_at = ActiveValue::Set(current_time);
        if insert {
            self.created_at = ActiveValue::Set(current_time);
        }
        Ok(self)
    }
}

impl Entity {
    pub fn find_by_email_and_provider(
        user_email: &str,
        provider: OAuthProviderEnum,
    ) -> Select<Entity> {
        Entity::find().filter(
            Condition::all()
                .add(Column::UserEmail.eq(user_email))
                .add(Column::Provider.eq(provider)),
        )
    }
}
