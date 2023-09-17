// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::*;
use chrono::Utc;
use sea_orm::{ActiveValue, Condition, entity::prelude::*};

use crate::enums::{cursor_enum::CursorEnum, order_enum::OrderEnum, role_enum::RoleEnum};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(concrete(name = "User", params()), complex)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(Some(200))", unique)]
    pub email: String,
    #[sea_orm(column_type = "String(Some(109))", unique)]
    pub username: String,
    #[sea_orm(column_type = "String(Some(50))")]
    pub first_name: String,
    #[sea_orm(column_type = "String(Some(50))")]
    pub last_name: String,
    #[sea_orm(column_type = "Date")]
    #[graphql(skip)]
    pub date_of_birth: chrono::NaiveDate,
    #[sea_orm(column_type = "String(Some(5))", default = "USER")]
    pub role: RoleEnum,
    #[sea_orm(column_type = "String(Some(200))", nullable)]
    pub picture: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "SmallInteger", default = 0)]
    pub version: i16,
    #[sea_orm(column_type = "Boolean", default = false)]
    pub confirmed: bool,
    #[sea_orm(column_type = "Boolean", default = false)]
    pub suspended: bool,
    #[sea_orm(column_type = "Text")]
    pub password: String,
    #[graphql(skip)]
    pub created_at: DateTime,
    #[graphql(skip)]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

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

#[ComplexObject]
impl Model {
    pub async fn date_of_birth(&self) -> String {
        self.date_of_birth.format("%Y-%m-%d").to_string()
    }

    pub async fn created_at(&self) -> i64 {
        self.created_at.timestamp()
    }

    pub async fn updated_at(&self) -> i64 {
        self.updated_at.timestamp()
    }
}

impl Model {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

impl Entity {
    pub fn find_by_id(id: i32) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(id))
    }

    pub fn find_by_username(username: &str) -> Select<Entity> {
        Self::find().filter(Column::Username.eq(username))
    }

    pub fn find_by_email(email: &str) -> Select<Entity> {
        Self::find().filter(Column::Email.eq(email))
    }

    pub fn find_by_version(id: i32, version: i16) -> Select<Entity> {
        Self::find().filter(
            Condition::all()
                .add(Column::Id.eq(id))
                .add(Column::Version.eq(version)),
        )
    }

    pub fn filter(
        order: OrderEnum,
        cursor: CursorEnum,
        after: Option<&str>,
        username: Option<&str>,
        first_name: Option<&str>,
        last_name: Option<&str>,
        description: Option<&str>,
    ) -> Select<Entity> {
        let mut condition = Condition::any();

        if let Some(username) = username {
            condition = condition.add(Column::Username.contains(username));
        }
        if let Some(first_name) = first_name {
            condition = condition.add(Column::FirstName.contains(first_name));
        }
        if let Some(last_name) = last_name {
            condition = condition.add(Column::LastName.contains(last_name));
        }
        if let Some(description) = description {
            condition = condition.add(Column::Description.contains(description));
        }
        if let Some(after) = after {
            let after = crate::helpers::decode_after(after);

            if let Some(after) = after {
                match cursor {
                    CursorEnum::Alpha => {
                        condition = condition.add(match order {
                            OrderEnum::Asc => Column::Username.gt(after),
                            OrderEnum::Desc => Column::Username.lt(after),
                        });
                    }
                    CursorEnum::Date => {
                        let after = after.parse::<i32>();

                        if let Ok(after) = after {
                            condition = condition.add(match order {
                                OrderEnum::Asc => Column::Id.gt(after),
                                OrderEnum::Desc => Column::Id.lt(after),
                            });
                        }
                    }
                }
            }
        }

        Self::find().filter(condition)
    }
}
