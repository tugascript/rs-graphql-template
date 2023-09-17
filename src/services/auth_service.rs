// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use rand::Rng;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ModelTrait};

use entities::{access_code, enums::oauth_provider_enum::OAuthProviderEnum, oauth_provider, user};

use crate::dtos::{bodies, responses};
use crate::providers::{Database, Jwt, Mailer, TokenType};
use crate::startup::AuthTokens;

use super::{helpers::verify_password, users_service};

fn generate_random_code() -> String {
    let mut code = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..6 {
        code.push_str(&rng.gen_range(0..10).to_string());
    }
    code
}

fn generate_email_code() -> Result<(String, String), String> {
    let code = generate_random_code();
    let code_hash = hash(&code, 5).map_err(|_| "Could not hash the code".to_string())?;
    Ok((code, code_hash))
}

fn verify_code(code: &str, hashed_code: &str) -> bool {
    if let Ok(result) = verify(code, hashed_code) {
        return result;
    }

    false
}

async fn find_oauth_provider<'a>(
    db: &'_ Database,
    email: &'a str,
    provider: OAuthProviderEnum,
) -> Result<oauth_provider::Model, &'a str> {
    let provider = oauth_provider::Entity::find_by_email_and_provider(email, provider)
        .one(db.get_connection())
        .await
        .map_err(|_| "Could not check if oauth exists")?;
    if let Some(provider) = provider {
        Ok(provider)
    } else {
        Err("Something went wrong")
    }
}

async fn create_code<'a>(
    db: &'_ Database,
    email: &'a str,
    code_hash: String,
    expires_in: i64,
) -> Result<(), &'a str> {
    access_code::ActiveModel {
        user_email: Set(email.to_string()),
        code: Set(code_hash),
        expires_at: Set((Utc::now() + Duration::seconds(expires_in)).naive_utc()),
        ..Default::default()
    }
    .insert(db.get_connection())
    .await
    .map_err(|_| "Could not create code")?;
    Ok(())
}

async fn find_code<'a>(db: &'_ Database, email: &'a str) -> Result<access_code::Model, &'a str> {
    let code = access_code::Entity::find_by_user(email)
        .one(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;
    if let Some(code) = code {
        Ok(code)
    } else {
        Err("Something went wrong")
    }
}

async fn validate_code<'a>(db: &'_ Database, email: &'a str, code: &'a str) -> Result<(), &'a str> {
    let access_code = find_code(db, email).await?;
    if verify_code(code, &access_code.code) {
        if access_code.expires_at > Utc::now().naive_utc() {
            Ok(())
        } else {
            Err("Code expired")
        }
    } else {
        Err("Invalid code")
    }
}

pub async fn sign_up(
    db: &Database,
    jwt: &Jwt,
    mailer: &Mailer,
    body: bodies::SignUp,
    provider: OAuthProviderEnum,
) -> Result<responses::SignUp, String> {
    if body.password1 != body.password2 {
        return Err("Passwords do not match".to_string());
    }

    let user = users_service::create_user(
        db,
        body.first_name,
        body.last_name,
        body.date_of_birth,
        body.email,
        body.password1,
        provider.clone(),
    )
    .await?;

    match provider {
        OAuthProviderEnum::Local => {
            let confirmation_token = jwt.generate_email_token(TokenType::Confirmation, &user)?;
            mailer.send_confirmation_email(&user.email, &user.full_name(), &confirmation_token)?;
            Ok(responses::SignUp::Message(
                "Confirmation email sent".to_string(),
            ))
        }
        _ => {
            let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
            Ok(responses::SignUp::Auth(responses::Auth::new(
                access_token,
                refresh_token,
                jwt.get_access_token_time(),
            )))
        }
    }
}

pub async fn confirm_email(
    db: &Database,
    jwt: &Jwt,
    token: &str,
) -> Result<responses::Auth, String> {
    let (id, version, _) = jwt.verify_email_token(TokenType::Confirmation, token)?;
    let user = users_service::find_one_by_version(db, id, version).await?;

    let mut user: user::ActiveModel = user.into();
    user.confirmed = Set(true);
    user.version = Set(version + 1);
    let user = user
        .update(db.get_connection())
        .await
        .map_err(|_| "Something went wrong")?;

    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ))
}

pub async fn sign_in(
    db: &Database,
    jwt: &Jwt,
    mailer: &Mailer,
    body: bodies::SignIn,
) -> Result<responses::SignIn, String> {
    let user = users_service::find_one_by_email(db, &body.email).await?;

    if !user.confirmed {
        let confirmation_token = jwt.generate_email_token(TokenType::Confirmation, &user)?;
        mailer.send_confirmation_email(&user.email, &user.full_name(), &confirmation_token)?;
        return Err("Please confirm your email".to_string());
    }
    if user.suspended {
        return Err("Your account has been suspended".to_string());
    }
    if !verify_password(&body.password, &user.password) {
        return Err("Invalid credentials".to_string());
    }

    let provider = find_oauth_provider(db, &body.email, OAuthProviderEnum::Local).await?;
    if provider.two_factor {
        let (code, code_hash) = generate_email_code()?;
        create_code(
            db,
            &body.email,
            code_hash,
            jwt.get_email_token_time(TokenType::Confirmation),
        )
        .await?;
        mailer.send_access_email(&user.email, &user.full_name(), &code)?;
        return Ok(responses::SignIn::Message(
            "Please check your email for the access code".to_string(),
        ));
    }

    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    Ok(responses::SignIn::Auth(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    )))
}

pub async fn confirm_sign_in(
    db: &Database,
    jwt: &Jwt,
    body: bodies::ConfirmSignIn,
) -> Result<responses::Auth, String> {
    let user = users_service::find_one_by_email(db, &body.email).await?;
    validate_code(db, &body.email, &body.code).await?;
    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ))
}

pub async fn refresh_token(
    db: &Database,
    jwt: &Jwt,
    auth_tokens: AuthTokens,
) -> Result<responses::Auth, String> {
    if let Some(refresh_token) = auth_tokens.refresh_token {
        let (id, version, _) = jwt.verify_email_token(TokenType::Refresh, &refresh_token)?;
        let user = users_service::find_one_by_version(db, id, version).await?;
        let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
        Ok(responses::Auth::new(
            access_token,
            refresh_token,
            jwt.get_access_token_time(),
        ))
    } else {
        Err("Invalid token".to_string())
    }
}

pub async fn reset_password(db: &Database, jwt: &Jwt, mailer: &Mailer, email: &str) -> bool {
    let formatted_email = email.to_lowercase();

    if find_oauth_provider(db, &formatted_email, OAuthProviderEnum::Local)
        .await
        .is_err()
    {
        return false;
    }

    let user = match users_service::find_one_by_email(db, &formatted_email).await {
        Ok(user) => user,
        Err(_) => return false,
    };
    let reset_token = match jwt.generate_email_token(TokenType::Reset, &user) {
        Ok(token) => token,
        Err(_) => return false,
    };

    if mailer
        .send_password_reset_email(&formatted_email, &user.full_name(), &reset_token)
        .is_err()
    {
        return false;
    }

    true
}
