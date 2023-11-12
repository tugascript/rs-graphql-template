// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anyhow::Error;
use async_graphql::{Context, Error as GqlError, Upload};
use chrono::NaiveDate;
use entities::user::Column;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};

use entities::helpers::GQLFilter;
use entities::{
    enums::{CursorEnum, OAuthProviderEnum, OrderEnum},
    oauth_provider,
    user::{ActiveModel, Entity, Model},
};

use crate::common::{
    format_name, format_point_slug, ServiceError, INVALID_CREDENTIALS, SOMETHING_WENT_WRONG,
};
use crate::dtos::Ratio;
use crate::helpers::AccessUser;
use crate::providers::{Database, ObjectStorage};

use super::{helpers::hash_password, uploader_service};

const USER_NOT_FOUND: &str = "User not found";

fn get_full_name(first_name: &str, last_name: &str) -> String {
    format!("{} {}", first_name, last_name)
}

async fn create_username(db: &Database, full_name: String) -> Result<String, ServiceError> {
    let point_slug = format_point_slug(&full_name);
    let count = Entity::find()
        .filter(Column::Username.like(format!("{}%", point_slug)))
        .count(db.get_connection())
        .await?;

    if count > 0 {
        return Ok(format!("{}.{}", point_slug, count + 1));
    }

    Ok(point_slug)
}

// add user name
pub async fn create_user(
    db: &Database,
    first_name: String,
    last_name: String,
    date_of_birth: String,
    email: String,
    mut password: String,
    provider: OAuthProviderEnum,
) -> Result<Model, ServiceError> {
    tracing::info_span!("users_service::create_user");
    tracing::trace_span!("Creating user");
    let email = email.to_lowercase();
    let first_name = format_name(&first_name)?;
    let last_name = format_name(&last_name)?;

    if provider == OAuthProviderEnum::Local {
        let count = Entity::find_by_email(&email)
            .count(db.get_connection())
            .await?;

        if count > 0 {
            return Err(ServiceError::conflict::<Error>("User already exists", None));
        }

        password = hash_password(&password)
            .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    }

    let date_of_birth = NaiveDate::parse_from_str(&date_of_birth, "%Y-%m-%d")
        .map_err(|e| ServiceError::bad_request("Could not parse date", Some(e)))?;
    let username = create_username(db, get_full_name(&first_name, &last_name)).await?;
    let user = db
        .get_connection()
        .transaction::<_, Model, DbErr>(|txn| {
            Box::pin(async move {
                let user = ActiveModel {
                    email: Set(email.clone()),
                    first_name: Set(first_name),
                    last_name: Set(last_name),
                    username: Set(username),
                    password: Set(password),
                    date_of_birth: Set(date_of_birth),
                    confirmed: Set(provider != OAuthProviderEnum::Local),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                oauth_provider::ActiveModel {
                    user_email: Set(email),
                    provider: Set(provider),
                    two_factor: Set(provider == OAuthProviderEnum::Local),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(user)
            })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => e,
            TransactionError::Transaction(e) => e,
        })?;
    tracing::trace_span!("Successfully created user", id=%user.id);
    Ok(user)
}

pub async fn find_or_create_oauth_provider(
    db: &Database,
    email: &str,
    provider: OAuthProviderEnum,
) -> Result<(), ServiceError> {
    let count = oauth_provider::Entity::find_by_email_and_provider(email, provider)
        .count(db.get_connection())
        .await?;

    if count == 0 {
        oauth_provider::ActiveModel {
            user_email: Set(email.to_string()),
            provider: Set(provider),
            ..Default::default()
        }
        .insert(db.get_connection())
        .await?;
    }

    Ok(())
}

pub async fn find_or_create(
    db: &Database,
    provider: OAuthProviderEnum,
    first_name: String,
    last_name: String,
    date_of_birth: String,
    email: String,
) -> Result<Model, ServiceError> {
    tracing::info_span!("users_service::find_or_create");
    let formatted_email = email.to_lowercase();
    let user = Entity::find_by_email(&formatted_email)
        .one(db.get_connection())
        .await?;

    if let Some(model) = user {
        tracing::trace_span!("User found");
        find_or_create_oauth_provider(db, &formatted_email, provider).await?;
        return Ok(model);
    }

    let user = create_user(
        db,
        first_name,
        last_name,
        date_of_birth,
        formatted_email,
        "none".to_string(),
        provider,
    )
    .await?;
    tracing::trace_span!("New user created");
    Ok(user)
}

pub async fn find_one_by_id(db: &Database, id: i32) -> Result<Model, ServiceError> {
    tracing::info_span!("users_service::find_one_by_id", %id);
    let user = Entity::find_by_id(id).one(db.get_connection()).await?;

    match user {
        Some(value) => {
            tracing::trace_span!("User found", %id);
            Ok(value)
        }
        None => Err(ServiceError::not_found::<Error>(USER_NOT_FOUND, None)),
    }
}

pub async fn find_one_by_email(db: &Database, email: &str) -> Result<Model, ServiceError> {
    let user = Entity::find_by_email(email)
        .one(db.get_connection())
        .await?;

    match user {
        Some(value) => Ok(value),
        None => Err(ServiceError::unauthorized::<ServiceError>(
            INVALID_CREDENTIALS,
            None,
        )),
    }
}

pub async fn find_one_by_username(db: &Database, username: &str) -> Result<Model, ServiceError> {
    let user = Entity::find_by_username(username)
        .one(db.get_connection())
        .await?;

    match user {
        Some(value) => Ok(value),
        None => Err(ServiceError::not_found::<Error>(USER_NOT_FOUND, None)),
    }
}

pub async fn find_one_by_version(
    db: &Database,
    id: i32,
    version: i16,
) -> Result<Model, ServiceError> {
    let user = Entity::find_by_version(id, version)
        .one(db.get_connection())
        .await?;

    match user {
        Some(value) => Ok(value),
        None => Err(ServiceError::not_found::<Error>(USER_NOT_FOUND, None)),
    }
}

pub async fn delete_user(db: &Database, id: i32) -> Result<(), ServiceError> {
    let user = find_one_by_id(db, id).await?;
    let result = user.delete(db.get_connection()).await?;

    if result.rows_affected > 0 {
        return Ok(());
    }

    Err(ServiceError::internal_server_error::<Error>(
        SOMETHING_WENT_WRONG,
        None,
    ))
}

pub async fn query(
    db: &Database,
    order: OrderEnum,
    cursor: CursorEnum,
    limit: u64,
    after: Option<String>,
    search: Option<String>,
) -> Result<(Vec<Model>, u64, u64), ServiceError> {
    let (select, inverse_select) = Entity::filter(order, cursor, after, search);
    let users = select.clone().limit(limit).all(db.get_connection()).await?;
    let count = select.count(db.get_connection()).await?;
    let previous_count = match inverse_select {
        Some(select) => select.count(db.get_connection()).await?,
        None => 0,
    };
    Ok((users, count, previous_count))
}

pub async fn update_picture(ctx: &Context<'_>, upload: Upload) -> Result<Model, GqlError> {
    let access_user = AccessUser::get_access_user(ctx)?;
    let db = ctx.data::<Database>()?;
    let user = find_one_by_id(db, access_user.id).await?;
    let object_storage = ctx.data::<ObjectStorage>()?;
    let image = uploader_service::upload_image(
        ctx,
        Some(access_user.id),
        Some(db),
        Some(object_storage),
        upload,
        Ratio::Square,
    )
    .await?;
    let mut user = user.into_active_model();
    user.picture = Set(Some(image.id));
    let user = user.update(db.get_connection()).await?;
    Ok(user)
}

pub async fn update_name(
    db: &Database,
    user_id: i32,
    first_name: String,
    last_name: String,
) -> Result<Model, ServiceError> {
    let first_name = format_name(&first_name)?;
    let last_name = format_name(&last_name)?;
    let mut user = find_one_by_id(db, user_id).await?.into_active_model();
    let username = create_username(db, get_full_name(&first_name, &last_name)).await?;
    user.first_name = Set(first_name);
    user.last_name = Set(last_name);
    user.username = Set(username);
    let user = user.update(db.get_connection()).await?;
    Ok(user)
}

pub async fn update_email(
    db: &Database,
    user_id: i32,
    email: String,
) -> Result<Model, ServiceError> {
    let email = email.to_lowercase();
    let mut user = find_one_by_id(db, user_id).await?.into_active_model();
    user.email = Set(email);
    let user = user.update(db.get_connection()).await?;
    Ok(user)
}
