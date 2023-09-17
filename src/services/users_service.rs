// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use chrono::NaiveDate;
use sea_orm::{ActiveModelTrait, DbErr, ModelTrait, PaginatorTrait, Set, TransactionTrait};

use entities::{
    enums::OAuthProviderEnum,
    oauth_provider,
    user::{ActiveModel, Entity, Model},
};

use crate::providers::Database;

use super::helpers::hash_password;

pub async fn create_user(
    db: &Database,
    first_name: String,
    last_name: String,
    date_of_birth: String,
    email: String,
    password: String,
    provider: OAuthProviderEnum,
) -> Result<Model, String> {
    let formatted_email = email.to_lowercase();
    let count = Entity::find_by_email(&formatted_email)
        .count(db.get_connection())
        .await
        .map_err(|_| "Could not check if user exists".to_string())?;

    if count > 0 {
        return Err("User already exists".to_string());
    }

    let hashed_password = hash_password(&password)?;
    let date_of_birth = NaiveDate::parse_from_str(&date_of_birth, "%Y-%m-%d")
        .map_err(|_| "Could not parse date")?;
    let user = db
        .get_connection()
        .transaction::<_, Model, DbErr>(|txn| {
            Box::pin(async move {
                let user = ActiveModel {
                    email: Set(formatted_email.clone()),
                    first_name: Set(first_name.to_string()),
                    last_name: Set(last_name.to_string()),
                    password: Set(hashed_password),
                    date_of_birth: Set(date_of_birth),
                    confirmed: Set(provider != OAuthProviderEnum::Local),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                oauth_provider::ActiveModel {
                    user_email: Set(formatted_email),
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
        .map_err(|_| "Something went wrong")?;
    Ok(user)
}

pub async fn find_one_by_id<'a>(db: &'_ Database, id: i32) -> Result<Model, &'a str> {
    let user = Entity::find_by_id(id)
        .one(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;

    match user {
        Some(value) => Ok(value),
        None => Err("User not found"),
    }
}

pub async fn find_one_by_email<'a>(db: &'_ Database, email: &'a str) -> Result<Model, &'a str> {
    let user = Entity::find_by_email(email)
        .one(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;

    match user {
        Some(value) => Ok(value),
        None => Err("Invalid credentials"),
    }
}

pub async fn find_one_by_username<'a>(
    db: &'_ Database,
    username: &'a str,
) -> Result<Model, &'a str> {
    let user = Entity::find_by_username(username)
        .one(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;

    match user {
        Some(value) => Ok(value),
        None => Err("User not found"),
    }
}

pub async fn find_one_by_version<'a>(
    db: &'_ Database,
    id: i32,
    version: i16,
) -> Result<Model, &'a str> {
    let user = Entity::find_by_version(id, version)
        .one(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;

    match user {
        Some(value) => Ok(value),
        None => Err("Token has expired"),
    }
}

pub async fn delete_user<'a>(db: &'_ Database, id: i32) -> Result<(), &'a str> {
    let user = find_one_by_id(db, id).await?;
    let result = user
        .delete(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;

    if result.rows_affected > 0 {
        return Ok(());
    }

    Err("Something went wrong")
}
