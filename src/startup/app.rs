// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{env, io, net::TcpListener};

use actix_web::{dev::Server, web, HttpServer};
use anyhow::Error;
use tracing_actix_web::TracingLogger;

use crate::controllers::auth_controller::auth_router;
use crate::providers::{Cache, Database, Jwt, Mailer, OAuth, ObjectStorage};
use crate::startup::schema_builder::{graphql_playgroud_route, graphql_route};

use super::schema_builder::build_schema;

pub struct App {
    port: u16,
    server: Server,
}

impl App {
    pub async fn new() -> Result<Self, Error> {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let db = Database::new().await?;
        let cache = Cache::new();
        let jwt = Jwt::new();
        let mailer = Mailer::new();
        let oauth = OAuth::new();
        let object_storage = ObjectStorage::new();
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
