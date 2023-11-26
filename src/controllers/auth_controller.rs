// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{
    cookie::{time::Duration, Cookie},
    http::header::LOCATION,
    web, HttpResponse, Scope,
};

use crate::common::{AuthTokens, InternalCause, ServiceError, UNAUTHORIZED};
use crate::dtos::{bodies, queries, responses};
use crate::providers::{Cache, Database, ExternalProvider, Jwt, Mailer, OAuth, TokenType};
use crate::services::auth_service;

fn save_refresh_token(
    cookie_name: &str,
    cookie_expiration: i64,
    auth_response: responses::Auth,
) -> HttpResponse {
    HttpResponse::Ok()
        .cookie(
            Cookie::build(cookie_name, &auth_response.refresh_token)
                .path("/api/auth")
                .http_only(true)
                .max_age(Duration::seconds(cookie_expiration))
                .finish(),
        )
        .json(auth_response)
}

fn remove_refresh_token(cookie_name: &str) -> HttpResponse {
    let mut cookie = Cookie::build(cookie_name, "")
        .path("/api/auth")
        .http_only(true)
        .max_age(Duration::seconds(0))
        .finish();
    cookie.make_removal();
    HttpResponse::Ok().cookie(cookie).finish()
}

async fn sign_up(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    mailer: web::Data<Mailer>,
    body: web::Json<bodies::SignUp>,
) -> Result<HttpResponse, ServiceError> {
    auth_service::sign_up(
        db.get_ref(),
        jwt.get_ref(),
        mailer.get_ref(),
        body.into_inner().validate()?,
    )
    .await?;
    Ok(HttpResponse::Ok().json(responses::Message::new("User created successfully")))
}

async fn confirm_email(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ConfirmEmail>,
) -> Result<HttpResponse, ServiceError> {
    let jwt_ref = jwt.get_ref();
    Ok(save_refresh_token(
        jwt_ref.get_refresh_name(),
        jwt_ref.get_email_token_time(TokenType::Refresh),
        auth_service::confirm_email(
            db.get_ref(),
            jwt_ref,
            &body.into_inner().validate()?.confirmation_token,
        )
        .await?,
    ))
}

async fn sign_in(
    db: web::Data<Database>,
    cache: web::Data<Cache>,
    jwt: web::Data<Jwt>,
    mailer: web::Data<Mailer>,
    body: web::Json<bodies::SignIn>,
) -> Result<HttpResponse, ServiceError> {
    let jwt_ref = jwt.get_ref();
    match auth_service::sign_in(
        db.get_ref(),
        cache.get_ref(),
        jwt_ref,
        mailer.get_ref(),
        body.into_inner().validate()?,
    )
    .await?
    {
        responses::SignIn::Auth(auth_response) => Ok(save_refresh_token(
            jwt_ref.get_refresh_name(),
            jwt_ref.get_email_token_time(TokenType::Refresh),
            auth_response,
        )),
        responses::SignIn::Mfa => Ok(HttpResponse::Ok().json(responses::Message::new(
            "Confirmation code sent, check your email",
        ))),
    }
}

async fn confirm_sign_in(
    db: web::Data<Database>,
    cache: web::Data<Cache>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ConfirmSignIn>,
) -> Result<HttpResponse, ServiceError> {
    let jwt_ref = jwt.get_ref();
    Ok(save_refresh_token(
        jwt_ref.get_refresh_name(),
        jwt_ref.get_email_token_time(TokenType::Refresh),
        auth_service::confirm_sign_in(
            db.get_ref(),
            cache.get_ref(),
            jwt_ref,
            body.into_inner().validate()?,
        )
        .await?,
    ))
}

async fn forgot_password(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    mailer: web::Data<Mailer>,
    body: web::Json<bodies::Email>,
) -> Result<HttpResponse, ServiceError> {
    auth_service::forgot_password(
        db.get_ref(),
        jwt.get_ref(),
        mailer.get_ref(),
        &body.into_inner().validate()?.email,
    )
    .await?;
    Ok(HttpResponse::Ok().json(responses::Message::new("Password reset link sent")))
}

async fn reset_password(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ResetPassword>,
) -> Result<HttpResponse, ServiceError> {
    auth_service::reset_password(db.get_ref(), jwt.get_ref(), body.into_inner().validate()?)
        .await?;
    Ok(HttpResponse::Ok().json(responses::Message::new("Password reset successfully")))
}

async fn sign_out(
    auth_tokens: AuthTokens,
    cache: web::Data<Cache>,
    jwt: web::Data<Jwt>,
    body: Option<web::Json<bodies::RefreshToken>>,
) -> Result<HttpResponse, ServiceError> {
    let refresh_token = match body {
        Some(body) => body.into_inner().validate()?.refresh_token,
        None => {
            if let Some(refresh_token) = auth_tokens.refresh_token {
                refresh_token
            } else {
                return Err(ServiceError::unauthorized(
                    "Refresh token not found",
                    Some(InternalCause::new("Refresh token not found")),
                ));
            }
        }
    };
    let jwt_ref = jwt.get_ref();
    auth_service::sign_out(cache.get_ref(), jwt_ref, &refresh_token).await?;
    Ok(remove_refresh_token(jwt_ref.get_refresh_name()))
}

async fn refresh_token(
    auth_tokens: AuthTokens,
    db: web::Data<Database>,
    cache: web::Data<Cache>,
    jwt: web::Data<Jwt>,
    body: Option<web::Json<bodies::RefreshToken>>,
) -> Result<HttpResponse, ServiceError> {
    let jwt_ref = jwt.get_ref();
    let token = match body {
        Some(body) => body.into_inner().validate()?.refresh_token,
        None => match auth_tokens.refresh_token {
            Some(refresh_token) => refresh_token,
            None => {
                return Err(ServiceError::unauthorized(
                    UNAUTHORIZED,
                    Some(InternalCause::new("Refresh token not found")),
                ));
            }
        },
    };
    Ok(save_refresh_token(
        jwt_ref.get_refresh_name(),
        jwt_ref.get_email_token_time(TokenType::Refresh),
        auth_service::refresh_token(db.get_ref(), cache.get_ref(), jwt_ref, &token).await?,
    ))
}

async fn update_password(
    auth_tokens: AuthTokens,
    db: web::Data<Database>,
    cache: web::Data<Cache>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ChangePassword>,
) -> Result<HttpResponse, ServiceError> {
    let access_token = match auth_tokens.access_token {
        Some(access_token) => access_token,
        None => {
            return Err(ServiceError::unauthorized(
                UNAUTHORIZED,
                Some(InternalCause::new("Access token not found")),
            ));
        }
    };
    let refresh_token = match auth_tokens.refresh_token {
        Some(refresh_token) => refresh_token,
        None => {
            return Err(ServiceError::unauthorized(
                UNAUTHORIZED,
                Some(InternalCause::new("Refresh token not found")),
            ));
        }
    };
    let jwt_ref = jwt.get_ref();
    Ok(save_refresh_token(
        jwt_ref.get_refresh_name(),
        jwt_ref.get_email_token_time(TokenType::Refresh),
        auth_service::update_password(
            db.get_ref(),
            cache.get_ref(),
            jwt_ref,
            body.into_inner().validate()?,
            &access_token,
            &refresh_token,
        )
        .await?,
    ))
}

async fn update_two_factor(
    auth_tokens: AuthTokens,
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ChangeTwoFactor>,
) -> Result<HttpResponse, ServiceError> {
    let access_token = match auth_tokens.access_token {
        Some(access_token) => access_token,
        None => {
            return Err(ServiceError::unauthorized(
                UNAUTHORIZED,
                Some(InternalCause::new("Access token not found")),
            ));
        }
    };
    auth_service::update_two_factor(
        db.get_ref(),
        jwt.get_ref(),
        body.into_inner(),
        &access_token,
    )
    .await?;
    Ok(HttpResponse::Ok().json(responses::Message::new("Two factor updated successfully")))
}

async fn facebook_sign_in(
    cache: web::Data<Cache>,
    oauth: web::Data<OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let url =
        auth_service::oauth_sign_in(cache.get_ref(), oauth.get_ref(), ExternalProvider::Facebook)
            .await?;
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((LOCATION, url))
        .finish())
}

async fn facebook_callback(
    db: web::Data<Database>,
    cache: web::Data<Cache>,
    oauth: web::Data<OAuth>,
    jwt: web::Data<Jwt>,
    query: web::Query<queries::OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let data = auth_service::oauth_callback(
        db.get_ref(),
        cache.get_ref(),
        oauth.get_ref(),
        jwt.get_ref(),
        ExternalProvider::Facebook,
        query.into_inner().validate()?,
    )
    .await?;
    Ok(HttpResponse::Ok().json(data))
}

async fn google_sign_in(
    cache: web::Data<Cache>,
    oauth: web::Data<OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let url =
        auth_service::oauth_sign_in(cache.get_ref(), oauth.get_ref(), ExternalProvider::Google)
            .await?;
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((LOCATION, url))
        .finish())
}

async fn google_callback(
    db: web::Data<Database>,
    cache: web::Data<Cache>,
    oauth: web::Data<OAuth>,
    jwt: web::Data<Jwt>,
    query: web::Query<queries::OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let data = auth_service::oauth_callback(
        db.get_ref(),
        cache.get_ref(),
        oauth.get_ref(),
        jwt.get_ref(),
        ExternalProvider::Google,
        query.into_inner().validate()?,
    )
    .await?;
    Ok(HttpResponse::Ok().json(data))
}

pub fn auth_router() -> Scope {
    web::scope("/api/auth")
        .route("/sign-up", web::post().to(sign_up))
        .route("/confirm-email", web::post().to(confirm_email))
        .route("/sign-in", web::post().to(sign_in))
        .route("/confirm-sign-in", web::post().to(confirm_sign_in))
        .route("/sign-out", web::post().to(sign_out))
        .route("/refresh-token", web::post().to(refresh_token))
        .route("/forgot-password", web::post().to(forgot_password))
        .route("/reset-password", web::post().to(reset_password))
        .route("/update-password", web::post().to(update_password))
        .route("/update-two-factor", web::post().to(update_two_factor))
        .route("/ext/facebook", web::get().to(facebook_sign_in))
        .route("/ext/facebook/callback", web::get().to(facebook_callback))
        .route("/ext/google", web::get().to(google_sign_in))
        .route("/ext/google/callback", web::get().to(google_callback))
}
