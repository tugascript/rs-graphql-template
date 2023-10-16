// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::connection::Edge;
use async_graphql::{
    connection::{Connection, EmptyFields},
    Context, Object, Result,
};

use entities::enums::{CursorEnum, OrderEnum};
use entities::helpers::GQLAfter;
use entities::user;

use crate::guards::AuthGuard;
use crate::helpers::AccessUser;
use crate::providers::Database;
use crate::services::users_service;

#[derive(Default)]
pub struct UsersQuery;

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
    ) -> Result<Connection<String, user::Model, EmptyFields, EmptyFields>> {
        let db = ctx.data::<Database>()?;
        let (users, count, previous_count) =
            users_service::query(db, order, cursor, limit, after, search).await?;
        let mut connection = Connection::new(previous_count > 0, count > limit);
        connection.edges.extend(
            users
                .into_iter()
                .map(|user| Edge::new(user.after(cursor), user)),
        );
        Ok(connection)
    }

    async fn user_by_id(&self, ctx: &Context<'_>, id: i32) -> Result<user::Model> {
        let db = ctx.data::<Database>()?;
        Ok(users_service::find_one_by_id(db, id).await?)
    }

    async fn user_by_username(&self, ctx: &Context<'_>, username: String) -> Result<user::Model> {
        let db = ctx.data::<Database>()?;
        Ok(users_service::find_one_by_username(db, &username).await?)
    }

    #[graphql(guard = "AuthGuard")]
    async fn me(&self, ctx: &Context<'_>) -> Result<user::Model> {
        let db = ctx.data::<Database>()?;
        let user = AccessUser::get_access_user(ctx)?;
        Ok(users_service::find_one_by_id(db, user.id).await?)
    }
}
