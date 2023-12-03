// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::common::format_name;
use crate::services::users_service;
use actix_web::{body::to_bytes, test, web::Bytes, App};
use entities::{enums, user};
use fake::{faker::name::raw::*, locales::EN, Fake};
use sea_orm::{ActiveModelTrait, ModelTrait, Set};
use serde_json::json;
use tracing_actix_web::TracingLogger;
use uuid::Uuid;

const PORT: u16 = 5000;
const GRAPHQL_PATH: &'static str = "/api/graphql";

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

use crate::providers::{Cache, Environment, TokenType};
use crate::{
    providers::{Database, Jwt},
    startup::ActixApp,
};

const VALID_PASSWORD: &'static str = "Valid_Password12";

async fn create_base_config() -> (Environment, Database, Jwt, Cache) {
    dotenvy::dotenv().expect("Failed to load .env file");
    let environment = Environment::Development;
    let db = Database::new()
        .await
        .expect("Failed to connect to database");
    let jwt = Jwt::new(&environment, &Uuid::new_v4().to_string());
    let cache = Cache::new();
    (environment, db, jwt, cache)
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

async fn delete_user(db: &Database, user: user::Model) {
    user.delete(db.get_connection()).await.unwrap();
}

#[actix_web::test]
async fn test_resolver_health_check() {
    let (environment, db, _, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .set_json(&json!({
            "query": r#"
                query { 
                    healthCheck { 
                        id
                        message
                    } 
                }
            "#
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains("OK"));
}

#[actix_web::test]
async fn test_resolver_users() {
    let (environment, db, _, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let mut user_vec = Vec::<user::Model>::new();

    for _ in 0..20 {
        user_vec.push(create_user(&db, true).await);
    }

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .set_json(&json!({
            "query": r#"
                query { 
                    users(order: ASC, cursor: DATE, limit: 10) {
                        edges {
                            node {
                                id
                                firstName
                                lastName
                                age
                                email
                                createdAt
                                updatedAt
                            }
                            cursor
                        }
                        pageInfo {
                            hasNextPage
                            hasPreviousPage
                            startCursor
                            endCursor
                        }
                        totalCount
                        previousCount
                    }
                }
            "#
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("users"));
    assert!(body.contains("edges"));
    assert!(body.contains("node"));
    assert!(body.contains("cursor"));
    assert!(body.contains("pageInfo"));
    assert!(body.contains("\"hasNextPage\":true"));
    assert!(body.contains("\"hasPreviousPage\":false"));

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .set_json(&json!({
            "query": format!(r#"
                query {{ 
                    users(order: ASC, cursor: DATE, limit: 10, after: "{}") {{
                        edges {{
                            node {{
                                id
                                firstName
                                lastName
                                age
                                email
                                createdAt
                                updatedAt
                            }}
                            cursor
                        }}
                        pageInfo {{
                            hasNextPage
                            hasPreviousPage
                            startCursor
                            endCursor
                        }}
                        totalCount
                        previousCount
                    }}
                }}
            "#, 
            body
                .split("endCursor")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .split("\"")
                .collect::<Vec<&str>>()
                .get(2)
                .unwrap(),
            ),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains("\"hasPreviousPage\":true"));

    for user in user_vec {
        delete_user(&db, user).await;
    }
}

#[actix_web::test]
async fn test_resolver_user_by_id() {
    let (environment, db, jwt, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let user = create_user(&db, true).await;

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .set_json(&json!({
            "query": format!(r#"
                query {{ 
                    userById(id: {}) {{
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }}
                }}
            "#, user.id),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("userById"));
    assert!(body.contains("id"));
    assert!(body.contains("firstName"));
    assert!(body.contains("lastName"));
    assert!(body.contains("age"));
    assert!(body.contains("\"email\":null"));
    assert!(body.contains("createdAt"));
    assert!(body.contains("updatedAt"));

    let access_token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &access_token);
    let authorization_header = ("Authorization", bearer_token.as_str());

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": format!(r#"
                query {{ 
                    userById(id: {}) {{
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }}
                }}
            "#, user.id),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains(&format!("\"email\":\"{}\"", user.email)));
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_resolver_user_by_username() {
    let (environment, db, _, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let user = create_user(&db, true).await;

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .set_json(&json!({
            "query": format!(r#"
                query {{ 
                    userByUsername(username: "{}") {{
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }}
                }}
            "#, user.username),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("userByUsername"));
    assert!(body.contains("id"));
    assert!(body.contains("firstName"));
    assert!(body.contains("lastName"));
    assert!(body.contains("age"));
    assert!(body.contains("\"email\":null"));
    assert!(body.contains("createdAt"));
    assert!(body.contains("updatedAt"));
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_resolver_me() {
    let (environment, db, jwt, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let user = create_user(&db, true).await;

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .set_json(&json!({
            "query": r#"
                query { 
                    me {
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }
                }
            "#,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("Unauthorized"));

    let access_token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &access_token);
    let authorization_header = ("Authorization", bearer_token.as_str());

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": r#"
                query { 
                    me {
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }
                }
            "#,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains(&format!("\"email\":\"{}\"", user.email)));
    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_resolver_update_user_name() {
    let (environment, db, jwt, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let user = create_user(&db, true).await;
    let access_token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &access_token);
    let authorization_header = ("Authorization", bearer_token.as_str());

    let first_name: String = Name(EN).fake();
    let last_name: String = Name(EN).fake();

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": format!(r#"
                mutation {{
                    updateUserName(input: {{ firstName: "{}", lastName: "{}" }}) {{
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }}
                }}
            "#, &first_name, &last_name),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("updateUserName"));
    assert!(body.contains("id"));
    assert!(body.contains("firstName"));
    assert!(body.contains(&format_name(&first_name).unwrap()));
    assert!(body.contains("lastName"));
    assert!(body.contains(&format_name(&last_name).unwrap()));

    // test bad formated names
    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": format!(r#"
            mutation {{
                updateUserName(input: {{ firstName: "{}", lastName: "{}" }}) {{
                    id
                    firstName
                    lastName
                    age
                    email
                    createdAt
                    updatedAt
                }}
            }}
        "#, "adsf&*&92--", &last_name),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("\"data\":null"));

    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_resolver_update_user_email() {
    let (environment, db, jwt, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let user = create_user(&db, true).await;
    let access_token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &access_token);
    let authorization_header = ("Authorization", bearer_token.as_str());

    let email = format!("{}@gmail.com", Uuid::new_v4().to_string());

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": format!(r#"
                mutation {{
                    updateUserEmail(email: "{}") {{
                        id
                        firstName
                        lastName
                        age
                        email
                        createdAt
                        updatedAt
                    }}
                }}
            "#, &email),
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();

    assert!(body.contains("updateUserEmail"));
    assert!(body.contains("id"));
    assert!(body.contains("firstName"));
    assert!(body.contains("lastName"));
    assert!(body.contains(&format!("\"email\":\"{}\"", email)));

    // test bad formated email
    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": format!(r#"
            mutation {{
                updateUserEmail(email: "{}") {{
                    id
                    firstName
                    lastName
                    age
                    email
                    createdAt
                    updatedAt
                }}
            }}
        "#, "not-an-email"),
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();
    assert!(body.contains("\"data\":null"));

    delete_user(&db, user).await;
}

#[actix_web::test]
async fn test_delete_user() {
    let (environment, db, jwt, _) = create_base_config().await;
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(ActixApp::build_app_config(environment, PORT, &db)),
    )
    .await;
    let user = create_user(&db, true).await;
    let access_token = create_token(&jwt, &user, None).await;
    let bearer_token = format!("Bearer {}", &access_token);
    let authorization_header = ("Authorization", bearer_token.as_str());

    let req = test::TestRequest::post()
        .uri(GRAPHQL_PATH)
        .insert_header(authorization_header)
        .set_json(&json!({
            "query": r#"
                mutation {
                    deleteUser {
                        message
                    }
                }
            "#,
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    let body = to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .to_owned();

    assert!(body.contains("deleteUser"));
    assert!(body.contains("message"));
    assert!(body.contains("User deleted successfully"));
}
