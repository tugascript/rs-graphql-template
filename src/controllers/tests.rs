// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::services::users_service;
use actix_web::{body::to_bytes, test, web::Bytes, App};
use bcrypt::hash;
use entities::{enums, oauth_provider, user};
use fake::{faker::name::raw::*, locales::EN, Fake};
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ModelTrait, Set};
use serde_json::json;
use tracing_actix_web::TracingLogger;
use uuid::Uuid;

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

use crate::providers::{Cache, TokenType};
use crate::{
    config::Config,
    providers::{Database, Jwt},
    startup::ActixApp,
};

const VALID_PASSWORD: &'static str = "Valid_Password12";

async fn create_base_config() -> (Config, Database, Jwt, Cache) {
    let config = Config::new();
    let db = Database::new(config.database_config())
        .await
        .expect("Failed to connect to database");
    let (access_jwt, refresh_jwt, confirmation_jwt, reset_jwt) = config.jwt_config();
    let api_id = config.api_id();
    let refresh_name = config.refresh_name();
    let jwt = Jwt::new(
        access_jwt,
        refresh_jwt,
        confirmation_jwt,
        reset_jwt,
        refresh_name,
        api_id,
    );
    let cache = Cache::new(config.cache_config()).unwrap();
    (config, db, jwt, cache)
}

async fn create_user(db: &Database, confirm: bool) -> user::Model {
    let email = format!("{}@gmail.com", Uuid::new_v4().to_string());
    let first_name: String = Name(EN).fake();
    let last_name: String = Name(EN).fake();
    let date_of_birth = "1990-01-01".to_string();
    let user = users_service::create_user(
        &db,
        first_name,
        last_name,
        date_of_birth,
        email,
        VALID_PASSWORD.to_string(),
        enums::OAuthProviderEnum::Local,
    )
    .await
    .unwrap();

    if !confirm {
        return user;
    }

    let mut user: user::ActiveModel = user.into();
    user.confirmed = Set(true);
    user.version = Set(1);
    let user = user.update(db.get_connection()).await.unwrap();
    user
}

async fn create_token(jwt: &Jwt, user: &user::Model, token_type: Option<TokenType>) -> String {
    if let Some(token_type) = token_type {
        jwt.generate_email_token(token_type, &user).unwrap()
    } else {
        jwt.generate_access_token(user).unwrap()
    }
}

fn check_is_auth_response(json_body: String) {
    assert!(json_body.contains("access_token"));
    assert!(json_body.contains("refresh_token"));
    assert!(json_body.contains("token_type"));
    assert!(json_body.contains("expires_in"));
}

async fn delete_user(db: &Database, user: user::Model) {
    user.delete(db.get_connection()).await.unwrap();
}

#[actix_web::test]
async fn test_health_check() {
    let (config, db, _, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/health-check")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_sign_up() {
    let (config, db, _, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success sign in
    let email = format!("{}@gmail.com", Uuid::new_v4().to_string().to_uppercase());
    let first_name: String = Name(EN).fake();
    let last_name: String = Name(EN).fake();
    let date_of_birth = "1990-01-01".to_string();
    let password1 = "Valid_Password12".to_string();
    let password2 = password1.clone();
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-up")
        .set_json(json!({
            "email": &email,
            "first_name": &first_name,
            "last_name": &last_name,
            "date_of_birth": &date_of_birth,
            "password1": &password1,
            "password2": &password2,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains("User created successfully"));

    let invalid_payloads = [
        json!({
            "email": "not_an_email",
            "first_name": &first_name,
            "last_name": &last_name,
            "date_of_birth": &date_of_birth,
            "password1": &password1,
            "password2": &password2,
        }),
        json!({
            "email": &email,
            "first_name": "Invalid%%66",
            "last_name": &last_name,
            "date_of_birth": &date_of_birth,
            "password1": &password1,
            "password2": &password2,
        }),
        json!({
            "email": &email,
            "first_name": &first_name,
            "last_name": "to_long".repeat(50),
            "date_of_birth": &date_of_birth,
            "password1": &password1,
            "password2": &password2,
        }),
        json!({
            "email": &email,
            "first_name": &first_name,
            "last_name": &last_name,
            "date_of_birth": "01-01-1990",
            "password1": &password1,
            "password2": &password2,
        }),
        json!({
            "email": &email,
            "first_name": &first_name,
            "last_name": &last_name,
            "date_of_birth": &date_of_birth,
            "password1": "not_valid_password",
            "password2": "not_valid_password",
        }),
        json!({
            "email": &email,
            "first_name": &first_name,
            "last_name": &last_name,
            "date_of_birth": &date_of_birth,
            "password1": &password1,
            "password2": format!("{}_e", &password2),
        }),
    ];

    for body in invalid_payloads {
        let req = test::TestRequest::post()
            .uri("/api/auth/sign-up")
            .set_json(body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(&resp.status().is_client_error());
        assert_eq!(&resp.status().as_u16(), &400);
    }

    // User already exists
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-up")
        .set_json(json!({
            "email": &email,
            "first_name": &first_name,
            "last_name": &last_name,
            "date_of_birth": &date_of_birth,
            "password1": &password1,
            "password2": &password2,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &409);

    // clean user
    let user = users_service::find_one_by_email(&db, &email.to_lowercase())
        .await
        .unwrap();
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_confirm_email() {
    let (config, db, jwt, _) = create_base_config().await;
    let user = create_user(&db, false).await;
    let token = create_token(&jwt, &user, Some(TokenType::Confirmation)).await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success confirm email
    let req = test::TestRequest::post()
        .uri("/api/auth/confirm-email")
        .set_json(json!({
            "confirmation_token": &token,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);
    check_is_auth_response(
        to_bytes(resp.into_body())
            .await
            .unwrap()
            .as_str()
            .to_owned(),
    );

    // User already confirmed
    let req = test::TestRequest::post()
        .uri("/api/auth/confirm-email")
        .set_json(json!({
            "confirmation_token": &token,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &401);

    // Invalid token
    let req = test::TestRequest::post()
        .uri("/api/auth/confirm-email")
        .set_json(json!({
            "confirmation_token": "invalid_token",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_sign_in() {
    let (config, db, _, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success sign in MFA
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-in")
        .set_json(json!({
            "email": &user.email.to_uppercase(),
            "password": VALID_PASSWORD,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains("Confirmation code sent, check your email"));

    // Success sign in no MFA
    // set two_factor to false
    let oauth_provider = oauth_provider::Entity::find_by_email_and_provider(
        &user.email,
        enums::OAuthProviderEnum::Local,
    )
    .one(db.get_connection())
    .await
    .unwrap()
    .unwrap();
    let mut oauth_provider: oauth_provider::ActiveModel = oauth_provider.into();
    oauth_provider.two_factor = Set(false);
    oauth_provider.update(db.get_connection()).await.unwrap();
    // run test
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-in")
        .set_json(json!({
            "email": &user.email.to_uppercase(),
            "password": VALID_PASSWORD,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);
    check_is_auth_response(
        to_bytes(resp.into_body())
            .await
            .unwrap()
            .as_str()
            .to_owned(),
    );

    // Invalid password
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-in")
        .set_json(json!({
            "email": &user.email,
            "password": "invalid_password",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &401);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_confirm_sign_in() {
    let (config, db, _, cache) = create_base_config().await;
    let user = create_user(&db, true).await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Generate code
    let code = "123456";
    let code_hash = hash(code, 5).unwrap();
    let key = format!("access_code:{}", &user.email);
    let mut connection = cache.get_connection().await.unwrap();
    connection
        .set_ex::<&str, &str, ()>(&key, &code_hash, 600)
        .await
        .unwrap();

    // Success confirm sign in
    let req = test::TestRequest::post()
        .uri("/api/auth/confirm-sign-in")
        .set_json(json!({
            "email": &user.email,
            "code": &code,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);
    check_is_auth_response(
        to_bytes(resp.into_body())
            .await
            .unwrap()
            .as_str()
            .to_owned(),
    );

    // Invalid code
    let req = test::TestRequest::post()
        .uri("/api/auth/confirm-sign-in")
        .set_json(json!({
            "email": &user.email,
            "code": "654321",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &401);

    // Invalid email
    let req = test::TestRequest::post()
        .uri("/api/auth/confirm-sign-in")
        .set_json(json!({
            "email": "not_an_email",
            "code": &code,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_sign_out() {
    let (config, db, jwt, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let token = create_token(&jwt, &user, Some(TokenType::Refresh)).await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success sign out
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-out")
        .set_json(json!({
            "refresh_token": &token,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);

    // Invalid refresh token
    let req = test::TestRequest::post()
        .uri("/api/auth/sign-out")
        .set_json(json!({
            "refresh_token": "invalid_token",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_refresh_token() {
    let (config, db, jwt, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let token = create_token(&jwt, &user, Some(TokenType::Refresh)).await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success refresh token
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh-token")
        .set_json(json!({
            "refresh_token": &token,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_forgot_password() {
    let (config, db, _, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success forgot password
    let req = test::TestRequest::post()
        .uri("/api/auth/forgot-password")
        .set_json(json!({
            "email": &user.email,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);

    // Should succed for a random email
    let req = test::TestRequest::post()
        .uri("/api/auth/forgot-password")
        .set_json(json!({
            "email": format!("{}@gmail.com", Uuid::new_v4().to_string()),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);

    // Invalid email
    let req = test::TestRequest::post()
        .uri("/api/auth/forgot-password")
        .set_json(json!({
            "email": "not_an_email",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_reset_password() {
    let (config, db, jwt, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let token = create_token(&jwt, &user, Some(TokenType::Reset)).await;
    let new_password = "New_Password12".to_string();
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Invalid password
    let req = test::TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(json!({
            "reset_token": &token,
            "password1": "invalid_password",
            "password2": "invalid_password",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // Success reset password
    let req = test::TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(json!({
            "reset_token": &token,
            "password1": &new_password,
            "password2": &new_password,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);

    // Invalid token
    let req = test::TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(json!({
            "reset_token": "invalid_token",
            "password1": &new_password,
            "password2": &new_password,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_update_password() {
    let (config, db, jwt, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &token);
    let authorization_header = ("Authorization", bearer_token.as_str());
    let new_password = "New_Password12".to_string();
    let new_password2 = new_password.clone();
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Invalid password
    let req = test::TestRequest::post()
        .uri("/api/auth/update-password")
        .insert_header(authorization_header)
        .set_json(json!({
            "old_password": VALID_PASSWORD,
            "password1": "invalid_password",
            "password2": "invalid_password",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // Passwords do not match
    let req = test::TestRequest::post()
        .uri("/api/auth/update-password")
        .insert_header(authorization_header)
        .set_json(json!({
            "old_password": VALID_PASSWORD,
            "password1": &new_password,
            "password2": &format!("{}_e", &new_password2),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &400);

    // Success update password
    let req = test::TestRequest::post()
        .uri("/api/auth/update-password")
        .insert_header(authorization_header)
        .set_json(json!({
            "old_password": VALID_PASSWORD,
            "password1": &new_password,
            "password2": &new_password2,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert_eq!(&resp.status().as_u16(), &200);

    // Invalid token
    let req = test::TestRequest::post()
        .uri("/api/auth/update-password")
        .insert_header(("Authorization", "invalid_token"))
        .set_json(json!({
            "old_password":&new_password,
            "password1": VALID_PASSWORD,
            "password2": VALID_PASSWORD,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());
    assert_eq!(&resp.status().as_u16(), &401);

    // clean user
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_update_two_factor() {
    let (config, db, jwt, _) = create_base_config().await;
    let user = create_user(&db, true).await;
    let token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &token);
    let authorization_header = ("Authorization", bearer_token.as_str());
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(&config, &db)),
    )
    .await;

    // Success update two factor
    let req = test::TestRequest::post()
        .uri("/api/auth/update-two-factor")
        .insert_header(authorization_header)
        .set_json(json!({
            "two_factor": false,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());

    // Invalid token
    let req = test::TestRequest::post()
        .uri("/api/auth/update-two-factor")
        .insert_header(("Authorization", "invalid_token"))
        .set_json(json!({
            "two_factor": false,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_client_error());

    // clean user
    delete_user(&db, user).await;
}
