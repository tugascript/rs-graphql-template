// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anyhow::Error;
use bcrypt::{hash, verify};
use oauth2::{
    reqwest::async_http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    Scope, TokenResponse,
};
use rand::Rng;
use redis::AsyncCommands;
use reqwest::Client;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;

use entities::{enums::oauth_provider_enum::OAuthProviderEnum, oauth_provider, user};

use crate::common::{
    InternalCause, ServiceError, CONFLICT_STATUS_CODE, INVALID_CREDENTIALS, NOT_FOUND_STATUS_CODE,
    SOMETHING_WENT_WRONG,
};
use crate::dtos::{bodies, queries, responses};
use crate::providers::{Cache, Database, ExternalProvider, Jwt, Mailer, OAuth, TokenType};
use crate::services::helpers::hash_password;

use super::{helpers::verify_password, users_service};

const BLACKLIST_TOKEN: &'static str = "blacklist_token";

fn generate_random_code() -> String {
    let mut code = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..6 {
        code.push_str(&rng.gen_range(0..10).to_string());
    }
    code
}

fn generate_email_code() -> Result<(String, String), ServiceError> {
    tracing::trace_span!("Generating random access code");
    let code = generate_random_code();
    let code_hash = hash(&code, 5)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    Ok((code, code_hash))
}

fn verify_code(code: &str, hashed_code: &str) -> bool {
    tracing::trace_span!("Verifying access code");
    if let Ok(result) = verify(code, hashed_code) {
        return result;
    }

    false
}

async fn find_oauth_provider(
    db: &Database,
    email: &str,
    provider: OAuthProviderEnum,
) -> Result<oauth_provider::Model, ServiceError> {
    tracing::trace_span!("Finding OAuth provider", provider = %provider.to_str());
    let provider = oauth_provider::Entity::find_by_email_and_provider(email, provider)
        .one(db.get_connection())
        .await?;
    if let Some(provider) = provider {
        Ok(provider)
    } else {
        Err(ServiceError::unauthorized(
            "Invalid credentials",
            Some(InternalCause::new("OAuth provider not found")),
        ))
    }
}

async fn create_code(
    cache: &Cache,
    user_id: i32,
    email: &str,
    code_hash: String,
    exp: i64,
) -> Result<(), ServiceError> {
    tracing::trace_span!("Creating two factor code", id = %user_id);
    let exp_usize = usize::try_from(exp)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    let mut connection = cache.get_connection().await?;
    connection
        .set_ex(format!("access_code:{}", email), code_hash, exp_usize)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    Ok(())
}

async fn validate_code(cache: &Cache, email: &str, code: &str) -> Result<(), ServiceError> {
    let key = format!("access_code:{}", email);
    let mut connection = cache.get_connection().await?;
    let hashed_code: Option<String> = connection
        .get(&key)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    if let Some(hashed_code) = hashed_code {
        if verify_code(code, &hashed_code) {
            connection
                .del(&key)
                .await
                .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
            return Ok(());
        }

        return Err(ServiceError::unauthorized::<Error>("Invalid code", None));
    }
    Err(ServiceError::unauthorized::<Error>("Code expired", None))
}

pub async fn sign_up(
    db: &Database,
    jwt: &Jwt,
    mailer: &Mailer,
    body: bodies::SignUp,
) -> Result<(), ServiceError> {
    tracing::info_span!("auth_service::sign_up");
    if body.password1 != body.password2 {
        return Err(ServiceError::bad_request::<Error>(
            "Passwords do not match",
            None,
        ));
    }

    let user = users_service::create_user(
        db,
        body.first_name,
        body.last_name,
        body.date_of_birth,
        body.email,
        body.password1,
        OAuthProviderEnum::Local,
    )
    .await?;
    tracing::trace_span!("User created");
    let confirmation_token = jwt.generate_email_token(TokenType::Confirmation, &user)?;
    mailer.send_confirmation_email(&user.email, &user.full_name(), &confirmation_token)?;
    tracing::trace_span!("Successfully signed up user", id = %user.id);
    Ok(())
}

pub async fn confirm_email(
    db: &Database,
    jwt: &Jwt,
    token: &str,
) -> Result<responses::Auth, ServiceError> {
    tracing::info_span!("auth_service::confirm_email");
    tracing::trace_span!("Verifying confirmation token");
    let (id, version, _, _) = jwt.verify_email_token(TokenType::Confirmation, token)?;
    let user = users_service::find_one_by_version(db, id, version).await?;

    tracing::trace_span!("Confirming user");
    let mut user: user::ActiveModel = user.into();
    user.confirmed = Set(true);
    user.version = Set(version + 1);
    let user = user.update(db.get_connection()).await?;

    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    tracing::trace_span!("Successfully confirmed user", id = %user.id);
    Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ))
}

pub async fn sign_in(
    db: &Database,
    cache: &Cache,
    jwt: &Jwt,
    mailer: &Mailer,
    body: bodies::SignIn,
) -> Result<responses::SignIn, ServiceError> {
    tracing::info_span!("Local signing in");
    let user = users_service::find_one_by_email(db, &body.email).await?;

    if !user.confirmed {
        tracing::trace_span!("User not confirmed", id = %user.id);
        let confirmation_token = jwt.generate_email_token(TokenType::Confirmation, &user)?;
        mailer.send_confirmation_email(&user.email, &user.full_name(), &confirmation_token)?;
        return Err(ServiceError::unauthorized::<ServiceError>(
            "Please confirm your email",
            None,
        ));
    }
    if user.suspended {
        tracing::trace_span!("User suspended", id = %user.id);
        return Err(ServiceError::forbidden::<ServiceError>(
            "Your account has been suspended",
            None,
        ));
    }
    if !verify_password(&body.password, &user.password) {
        tracing::trace_span!("Invalid credentials", id = %user.id);
        return Err(ServiceError::unauthorized::<ServiceError>(
            INVALID_CREDENTIALS,
            None,
        ));
    }

    let provider = find_oauth_provider(db, &body.email, OAuthProviderEnum::Local).await?;
    if provider.two_factor {
        tracing::trace_span!("Two factor authentication enabled", id = %user.id);
        let (code, code_hash) = generate_email_code()?;
        create_code(
            cache,
            user.id,
            &body.email,
            code_hash,
            jwt.get_email_token_time(TokenType::Confirmation),
        )
        .await?;
        mailer.send_access_email(&user.email, &user.full_name(), &code)?;
        tracing::info_span!("Sign in successful", id = %user.id);
        return Ok(responses::SignIn::Mfa);
    }

    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    tracing::info_span!("Sign in successful", id = %user.id);
    Ok(responses::SignIn::Auth(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    )))
}

pub async fn confirm_sign_in(
    db: &Database,
    cache: &Cache,
    jwt: &Jwt,
    body: bodies::ConfirmSignIn,
) -> Result<responses::Auth, ServiceError> {
    let email = body.email.to_lowercase();
    let user = users_service::find_one_by_email(db, &email).await?;
    validate_code(cache, &email, &body.code).await?;
    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ))
}

async fn check_blacklist(cache: &Cache, token_id: &str) -> Result<bool, ServiceError> {
    let mut connection = cache.get_connection().await?;
    let key = format!("{}:{}", BLACKLIST_TOKEN, token_id);
    let value: Option<i32> = connection
        .get(&key)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    Ok(value.is_some())
}

pub async fn refresh_token(
    db: &Database,
    cache: &Cache,
    jwt: &Jwt,
    refresh_token: &str,
) -> Result<responses::Auth, ServiceError> {
    let (id, version, token_id, exp) =
        jwt.verify_email_token(TokenType::Refresh, &refresh_token)?;

    if check_blacklist(cache, &token_id).await? {
        return Err(ServiceError::unauthorized(
            "Invalid token",
            Some(InternalCause::new("Token is blacklisted")),
        ));
    }

    let user = users_service::find_one_by_version(db, id, version).await?;
    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    create_blacklisted_token(cache, id, &token_id, exp).await?;
    return Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ));
}

pub async fn forgot_password(
    db: &Database,
    jwt: &Jwt,
    mailer: &Mailer,
    email: &str,
) -> Result<(), ServiceError> {
    tracing::info_span!("auth_service::reset_password_email");
    let formatted_email = email.to_lowercase();

    if let Err(err) = find_oauth_provider(db, &formatted_email, OAuthProviderEnum::Local).await {
        if err.get_status_code() == CONFLICT_STATUS_CODE {
            tracing::trace_span!("Failed to find user local OAuth provider");
            return Ok(());
        }

        return Err(err);
    }

    let user = match users_service::find_one_by_email(db, &formatted_email).await {
        Ok(user) => user,
        Err(err) => {
            if err.get_status_code() == NOT_FOUND_STATUS_CODE {
                tracing::trace_span!("Failed to find user");
                return Ok(());
            }

            return Err(err);
        }
    };

    let reset_token = jwt.generate_email_token(TokenType::Reset, &user)?;
    mailer.send_password_reset_email(&formatted_email, &user.full_name(), &reset_token)?;

    Ok(())
}

pub async fn reset_password(
    db: &Database,
    jwt: &Jwt,
    body: bodies::ResetPassword,
) -> Result<(), ServiceError> {
    let (id, version, _, _) = jwt.verify_email_token(TokenType::Reset, &body.reset_token)?;

    if body.password1 != body.password2 {
        return Err(ServiceError::bad_request::<Error>(
            "Passwords do not match",
            None,
        ));
    }

    let user = users_service::find_one_by_version(db, id, version).await?;
    let mut user: user::ActiveModel = user.into();
    user.password = Set(hash_password(&body.password1)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?);
    user.version = Set(version + 1);
    user.update(db.get_connection()).await?;
    Ok(())
}

pub async fn update_password(
    db: &Database,
    cache: &Cache,
    jwt: &Jwt,
    body: bodies::ChangePassword,
    access_token: &str,
    refresh_token: &str,
) -> Result<responses::Auth, ServiceError> {
    let (id, _) = jwt.verify_access_token(&access_token)?;
    let user = users_service::find_one_by_id(db, id).await?;
    let user_version = user.version;
    let (_, version, token_id, exp) = jwt.verify_email_token(TokenType::Refresh, &refresh_token)?;

    if user_version != version {
        return Err(ServiceError::unauthorized(
            "Invalid token",
            Some(InternalCause::new(
                "Token version does not match user version",
            )),
        ));
    }

    let mut user: user::ActiveModel = user.into();
    user.password = Set(hash_password(&body.password1)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?);
    user.version = Set(user_version + 1);
    let user = user.update(db.get_connection()).await?;
    create_blacklisted_token(cache, id, &token_id, exp).await?;
    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ))
}

async fn create_blacklisted_token(
    cache: &Cache,
    user_id: i32,
    token_id: &str,
    exp: i64,
) -> Result<(), ServiceError> {
    tracing::trace_span!("Creating blacklisted token", id = %user_id);
    let exp_usize = usize::try_from(exp)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    let mut connection = cache.get_connection().await?;
    let key = format!("{}:{}", BLACKLIST_TOKEN, token_id);
    connection
        .set_ex(&key, user_id, exp_usize)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    Ok(())
}

pub async fn sign_out(cache: &Cache, jwt: &Jwt, refresh_token: &str) -> Result<(), ServiceError> {
    let (id, _, token_id, exp) = jwt.verify_email_token(TokenType::Refresh, refresh_token)?;

    if check_blacklist(cache, &token_id).await? {
        return Ok(());
    }
    create_blacklisted_token(cache, id, &token_id, exp).await?;
    return Ok(());
}

async fn save_csrf_token(
    cache: &Cache,
    provider: &ExternalProvider,
    token: &str,
    verifier: &str,
) -> Result<(), ServiceError> {
    let mut connection = cache.get_connection().await?;
    let key = format!("{}:{}", provider.to_str(), token);
    connection
        .set_ex(&key, verifier, 300)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    Ok(())
}

async fn get_csrf_token(
    cache: &Cache,
    provider: &ExternalProvider,
    token: &str,
) -> Result<String, ServiceError> {
    let mut connection = cache.get_connection().await?;
    let key = format!("{}:{}", provider.to_str(), token);
    let verifier: Option<String> = connection
        .get(&key)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;

    if let Some(verifier) = verifier {
        return Ok(verifier);
    }

    Err(ServiceError::unauthorized(
        "Invalid credentials",
        Some(InternalCause::new("Invalid CSRF token")),
    ))
}

pub async fn oauth_sign_in(
    cache: &Cache,
    oauth: &OAuth,
    provider: ExternalProvider,
) -> Result<String, ServiceError> {
    let scopes = oauth.get_external_client_scopes(&provider);
    let client = oauth.get_external_client(&provider)?;
    let mut request = client.authorize_url(CsrfToken::new_random);
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    for scope in scopes {
        request = request.add_scope(Scope::new(scope.to_string()));
    }

    let (url, token) = request.set_pkce_challenge(pkce_code_challenge).url();
    save_csrf_token(
        cache,
        &provider,
        token.secret(),
        pkce_code_verifier.secret(),
    )
    .await?;
    Ok(url.to_string())
}

pub async fn oauth_callback(
    db: &Database,
    cache: &Cache,
    oauth: &OAuth,
    jwt: &Jwt,
    provider: ExternalProvider,
    query: queries::OAuth,
) -> Result<responses::Auth, ServiceError> {
    let client = oauth.get_external_client(&provider)?;
    let verifier = get_csrf_token(cache, &provider, &query.state).await?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(PkceCodeVerifier::new(verifier))
        .request_async(async_http_client)
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    let url = oauth.get_external_client_info_url(&provider);
    let auth_header = format!("Bearer {}", token_response.access_token().secret());
    let result = Client::new()
        .get(url)
        .header("Authorization", &auth_header)
        .send()
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    let user_info: responses::UserInfo = result
        .json::<responses::OAuthUserInfo>()
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?
        .try_into()?;
    let user = users_service::find_or_create(
        db,
        provider.to_oauth_provider(),
        user_info.first_name,
        user_info.last_name,
        user_info.date_of_birth,
        user_info.email,
    )
    .await?;
    let (access_token, refresh_token) = jwt.generate_auth_tokens(&user)?;
    Ok(responses::Auth::new(
        access_token,
        refresh_token,
        jwt.get_access_token_time(),
    ))
}
