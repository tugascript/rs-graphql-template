// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{io, net::TcpListener};

use actix_web::{dev::Server, web, HttpServer};
use anyhow::Error;
use tracing_actix_web::TracingLogger;

use crate::config::Config;
use crate::controllers::auth_controller::auth_router;
use crate::controllers::health_controller::health_router;
use crate::providers::{Cache, Database, Jwt, Mailer, OAuth, ObjectStorage};
use crate::startup::schema_builder::{graphql_playgroud_route, graphql_route};

use super::schema_builder::build_schema;

pub struct App {
    port: u16,
    server: Server,
}

impl App {
    pub async fn new() -> Result<Self, Error> {
        let config = Config::new();
        let db = Database::new(config.database_config()).await?;
        let cache = Cache::new(config.cache_config())?;
        let (access_jwt, refresh_jwt, confirmation_jwt, reset_jwt) = config.jwt_config();
        let jwt = Jwt::new(
            access_jwt,
            refresh_jwt,
            confirmation_jwt,
            reset_jwt,
            config.refresh_name(),
            config.api_id(),
        );
        let (email_host, email_port, email_user, email_password) = config.email_config();
        let mailer = Mailer::new(
            config.get_environment(),
            email_host,
            email_port,
            email_user,
            email_password,
            config.frontend_url(),
        );
        let (google_id, google_secret) = config.google_config();
        let (facebook_id, facebook_secret) = config.facebook_config();
        let oauth = OAuth::new(
            google_id,
            google_secret,
            facebook_id,
            facebook_secret,
            config.backend_url(),
        );
        let (region, host, bucket, access_key, secret_key, namespace) =
            config.object_storage_config();
        let object_storage =
            ObjectStorage::new(region, host, bucket, access_key, secret_key, namespace);
        let (host, port) = config.app_config();
        let listener = TcpListener::bind(format!("{}:{}", host, port))?;
        let port = listener.local_addr().unwrap().port();
        let schema = build_schema(&db, &jwt, object_storage);
        let server = HttpServer::new(move || {
            actix_web::App::new()
                .wrap(TracingLogger::default())
                .app_data(web::Data::new(oauth.clone()))
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(cache.clone()))
                .app_data(web::Data::new(jwt.clone()))
                .app_data(web::Data::new(mailer.clone()))
                .service(auth_router())
                .service(health_router())
                .app_data(web::Data::new(schema.clone()))
                .service(graphql_route())
                .service(graphql_playgroud_route())
        })
        .listen(listener)?
        .run();
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn start_server(self) -> Result<(), io::Error> {
        self.server.await
    }
}
