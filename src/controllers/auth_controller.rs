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

use crate::common::{AuthTokens, InternalCause, ServiceError};
use crate::dtos::{bodies, queries, responses};
use crate::providers::{Database, ExternalProvider, Jwt, Mailer, OAuth, TokenType};
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
        body.into_inner(),
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
        auth_service::confirm_email(db.get_ref(), jwt_ref, &body.confirmation_token).await?,
    ))
}

async fn sign_in(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    mailer: web::Data<Mailer>,
    body: web::Json<bodies::SignIn>,
) -> Result<HttpResponse, ServiceError> {
    let jwt_ref = jwt.get_ref();
    match auth_service::sign_in(db.get_ref(), jwt_ref, mailer.get_ref(), body.into_inner()).await? {
        responses::SignIn::Auth(auth_response) => Ok(save_refresh_token(
            jwt_ref.get_refresh_name(),
            jwt_ref.get_email_token_time(TokenType::Refresh),
            auth_response,
        )),
        responses::SignIn::Mfa => {
            Ok(HttpResponse::Ok().json(responses::Message::new("Confirmation code sent")))
        }
    }
}

async fn confirm_sign_in(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ConfirmSignIn>,
) -> Result<HttpResponse, ServiceError> {
    let jwt_ref = jwt.get_ref();
    Ok(save_refresh_token(
        jwt_ref.get_refresh_name(),
        jwt_ref.get_email_token_time(TokenType::Refresh),
        auth_service::confirm_sign_in(db.get_ref(), jwt_ref, body.into_inner()).await?,
    ))
}

async fn forgot_password(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    mailer: web::Data<Mailer>,
    body: web::Json<bodies::Email>,
) -> Result<HttpResponse, ServiceError> {
    auth_service::forgot_password(db.get_ref(), jwt.get_ref(), mailer.get_ref(), &body.email)
        .await?;
    Ok(HttpResponse::Ok().json(responses::Message::new("Password reset link sent")))
}

async fn reset_password(
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    body: web::Json<bodies::ResetPassword>,
) -> Result<HttpResponse, ServiceError> {
    auth_service::reset_password(db.get_ref(), jwt.get_ref(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(responses::Message::new("Password reset successfully")))
}

async fn sign_out(
    auth_token: AuthTokens,
    db: web::Data<Database>,
    jwt: web::Data<Jwt>,
    body: web::Json<Option<bodies::RefreshToken>>,
) -> Result<HttpResponse, ServiceError> {
    let refresh_token = match body.into_inner() {
        Some(body) => body.refresh_token,
        None => {
            if let Some(refresh_token) = auth_token.refresh_token {
                refresh_token
            } else {
                return Err(ServiceError::unauthorized(
                    "Refresh token not found",
                    Some(InternalCause::new("Refresh token not found")),
                ));
            }
        }
    };
    auth_service::sign_out(db.get_ref(), jwt.get_ref(), &refresh_token).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn facebook_sign_in(
    db: web::Data<Database>,
    oauth: web::Data<OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let url =
        auth_service::oauth_sign_in(db.get_ref(), oauth.get_ref(), ExternalProvider::Facebook)
            .await?;
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((LOCATION, url))
        .finish())
}

async fn facebook_callback(
    db: web::Data<Database>,
    oauth: web::Data<OAuth>,
    jwt: web::Data<Jwt>,
    query: web::Query<queries::OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let data = auth_service::oauth_callback(
        db.get_ref(),
        oauth.get_ref(),
        jwt.get_ref(),
        ExternalProvider::Facebook,
        query.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(data))
}

async fn google_sign_in(
    db: web::Data<Database>,
    oauth: web::Data<OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let url = auth_service::oauth_sign_in(db.get_ref(), oauth.get_ref(), ExternalProvider::Google)
        .await?;
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((LOCATION, url))
        .finish())
}

async fn google_callback(
    db: web::Data<Database>,
    oauth: web::Data<OAuth>,
    jwt: web::Data<Jwt>,
    query: web::Query<queries::OAuth>,
) -> Result<HttpResponse, ServiceError> {
    let data = auth_service::oauth_callback(
        db.get_ref(),
        oauth.get_ref(),
        jwt.get_ref(),
        ExternalProvider::Google,
        query.into_inner(),
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
        .route("/forgot-password", web::post().to(forgot_password))
        .route("/reset-password", web::post().to(reset_password))
        .route("/ext/facebook", web::get().to(facebook_sign_in))
        .route("/ext/facebook/callback", web::get().to(facebook_callback))
        .route("/ext/google", web::get().to(google_sign_in))
        .route("/ext/google/callback", web::get().to(google_callback))
}
