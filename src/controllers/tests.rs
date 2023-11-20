// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{body::to_bytes, test, web::Bytes, App};
use fake::{faker::name::raw::*, locales::EN, Fake};
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

use crate::{config::Config, providers::Database, startup::ActixApp};

async fn create_base_config() -> (Config, Database) {
    let config = Config::new();
    let db = Database::new(config.database_config())
        .await
        .expect("Failed to connect to database");
    (config, db)
}

#[actix_web::test]
async fn test_health_check() {
    let (config, db) = create_base_config().await;
    let app =
        test::init_service(App::new().configure(ActixApp::build_app_config(&config, &db))).await;

    let req = test::TestRequest::get()
        .uri("/api/health-check")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_sign_up() {
    let (config, db) = create_base_config().await;
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
            "email": email,
            "first_name": first_name,
            "last_name": last_name,
            "date_of_birth": date_of_birth,
            "password1": password1,
            "password2": password2,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(&resp.status().is_success());
    assert!(to_bytes(resp.into_body())
        .await
        .unwrap()
        .as_str()
        .contains("User created successfully"));
}
