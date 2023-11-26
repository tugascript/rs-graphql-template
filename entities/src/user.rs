// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::Utc;
use sea_orm::QueryOrder;
use sea_orm::{entity::prelude::*, ActiveValue, Condition};

use crate::enums::{cursor_enum::CursorEnum, order_enum::OrderEnum, role_enum::RoleEnum};
use crate::helpers::{decode_cursor, encode_cursor, GQLAfter, GQLQuery};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
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
    pub date_of_birth: chrono::NaiveDate,
    #[sea_orm(column_type = "String(Some(5))", default_value = "USER")]
    pub role: RoleEnum,
    #[sea_orm(column_type = "Uuid", nullable)]
    pub picture: Option<String>,
    #[sea_orm(column_type = "SmallInteger", default_value = 0)]
    pub version: i16,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub confirmed: bool,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub suspended: bool,
    #[sea_orm(column_type = "Text")]
    pub password: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::oauth_provider::Entity")]
    OAuthProvider,
}

impl Related<super::oauth_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OAuthProvider.def()
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

impl Model {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

impl GQLAfter for Model {
    fn after(&self, cursor: CursorEnum) -> String {
        match cursor {
            CursorEnum::Alpha => encode_cursor(&self.username),
            CursorEnum::Date => encode_cursor(&self.id.to_string()),
        }
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
}

impl GQLQuery for Entity {
    fn query(
        order: OrderEnum,
        cursor: CursorEnum,
        after: Option<String>,
        search: Option<String>,
    ) -> (Select<Entity>, Option<Select<Entity>>) {
        let mut condition = Condition::any();
        let mut inverse_condition = None;

        if let Some(search) = search {
            condition = condition
                .add(Column::Username.contains(&search))
                .add(Column::FirstName.contains(&search))
                .add(Column::LastName.contains(&search));
        }
        if condition.is_empty() {
            condition = Condition::all()
                .add(Column::Confirmed.eq(true))
                .add(Column::Suspended.eq(false));
        } else {
            condition = Condition::all()
                .add(Column::Confirmed.eq(true))
                .add(Column::Suspended.eq(false))
                .add(condition);
        }
        if let Some(after) = after {
            let after = decode_cursor(&after);

            if let Some(after) = after {
                match cursor {
                    CursorEnum::Alpha => {
                        inverse_condition = Some(condition.clone().add(match order {
                            OrderEnum::Asc => Column::Username.lt(&after),
                            OrderEnum::Desc => Column::Username.gt(&after),
                        }));
                        condition = condition.add(match order {
                            OrderEnum::Asc => Column::Username.gt(&after),
                            OrderEnum::Desc => Column::Username.lt(&after),
                        });
                    }
                    CursorEnum::Date => {
                        let after = after.parse::<i32>();

                        if let Ok(after) = after {
                            inverse_condition = Some(condition.clone().add(match order {
                                OrderEnum::Asc => Column::Id.lt(after),
                                OrderEnum::Desc => Column::Id.gt(after),
                            }));
                            condition = condition.add(match order {
                                OrderEnum::Asc => Column::Id.gt(after),
                                OrderEnum::Desc => Column::Id.lt(after),
                            });
                        }
                    }
                }
            }
        }

        (
            Self::find().filter(condition).order_by_asc(match cursor {
                CursorEnum::Alpha => Column::Username,
                CursorEnum::Date => Column::Id,
            }),
            match inverse_condition {
                Some(inverse_condition) => Some(Self::find().filter(inverse_condition)),
                None => None,
            },
        )
    }
}
