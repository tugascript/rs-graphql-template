// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::{Context, Object, Result, Upload};

use entities::enums::{CursorEnum, OrderEnum};
use entities::helpers::GQLAfter;

use crate::dtos::inputs::UpdateName;
use crate::dtos::objects::{Message, TotalCount, User};
use crate::guards::AuthGuard;
use crate::helpers::AccessUser;
use crate::providers::Database;
use crate::services::users_service;

#[derive(Default)]
pub struct UsersQuery;

#[derive(Default)]
pub struct UsersMutation;

#[Object]
impl UsersQuery {
    async fn query_users(
        &self,
        ctx: &Context<'_>,
        order: OrderEnum,
        cursor: CursorEnum,
        limit: u64,
        after: Option<String>,
        search: Option<String>,
    ) -> Result<Connection<String, User, TotalCount, EmptyFields>> {
        let db = ctx.data::<Database>()?;
        let (users, count, previous_count) =
            users_service::query(db, order, cursor, limit, after, search).await?;
        let mut connection = Connection::with_additional_fields(
            previous_count > 0,
            count > limit,
            TotalCount::new(count, previous_count),
        );
        connection.edges.extend(
            users
                .into_iter()
                .map(|user| Edge::new(user.after(cursor), user.into())),
        );
        Ok(connection)
    }

    async fn user_by_id(&self, ctx: &Context<'_>, id: i32) -> Result<User> {
        let db = ctx.data::<Database>()?;
        Ok(users_service::find_one_by_id(db, id).await?.into())
    }

    async fn user_by_username(&self, ctx: &Context<'_>, username: String) -> Result<User> {
        let db = ctx.data::<Database>()?;
        Ok(users_service::find_one_by_username(db, &username)
            .await?
            .into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let db = ctx.data::<Database>()?;
        let user = AccessUser::get_access_user(ctx)?;
        Ok(users_service::find_one_by_id(db, user.id).await?.into())
    }
}

#[Object]
impl UsersMutation {
    #[graphql(guard = "AuthGuard")]
    async fn update_picture(&self, ctx: &Context<'_>, upload: Upload) -> Result<User> {
        Ok(users_service::update_picture(ctx, upload).await?.into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn update_name(&self, ctx: &Context<'_>, input: UpdateName) -> Result<User> {
        let db = ctx.data::<Database>()?;
        let user = AccessUser::get_access_user(ctx)?;
        Ok(
            users_service::update_name(db, user.id, input.first_name, input.last_name)
                .await?
                .into(),
        )
    }

    #[graphql(guard = "AuthGuard")]
    async fn update_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(email, min_length = 5, max_length = 200))] email: String,
    ) -> Result<User> {
        let db = ctx.data::<Database>()?;
        let user = AccessUser::get_access_user(ctx)?;
        Ok(users_service::update_email(db, user.id, email)
            .await?
            .into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn delete_user(&self, ctx: &Context<'_>) -> Result<Message> {
        let db = ctx.data::<Database>()?;
        let user = AccessUser::get_access_user(ctx)?;
        users_service::delete_user(db, user.id).await?;
        Ok(Message::new("User deleted successfully"))
    }
}
