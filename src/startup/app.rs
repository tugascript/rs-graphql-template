// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{io, net::TcpListener};

use actix_web::guard;
use actix_web::{dev::Server, web, App, HttpServer};
use anyhow::Error;
use tracing_actix_web::TracingLogger;

use crate::controllers::auth_controller::auth_router;
use crate::controllers::health_controller::health_router;
use crate::providers::{
    ApiURLs, Cache, Database, Environment, Jwt, Mailer, OAuth, ObjectStorage, ServerLocation,
};

use super::schema_builder::{build_schema, graphql_playground, graphql_request};

pub struct ActixApp {
    port: u16,
    server: Server,
}

impl ActixApp {
    pub async fn new() -> Result<Self, Error> {
        if dotenvy::dotenv().is_err() {
            println!("No .env file found");
            println!("Using environment variables instead");
        }

        let ServerLocation(host, port) = ServerLocation::new();
        let db = Database::new().await?;
        let listener = TcpListener::bind(format!("{}:{}", &host, &port))?;
        let port = listener.local_addr().unwrap().port();
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .configure(Self::build_app_config(Environment::new(), port, &db))
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

    pub fn build_app_config(
        environment: Environment,
        port: u16,
        db: &Database,
    ) -> impl Fn(&mut web::ServiceConfig) {
        let db = db.clone();
        move |cfg: &mut web::ServiceConfig| {
            let urls = ApiURLs::new(&environment, port);
            let jwt = Jwt::new(&environment, &urls.api_id);
            cfg.app_data(web::Data::new(build_schema(
                &db,
                &jwt,
                ObjectStorage::new(&environment),
            )))
            .service(
                web::resource("/api/graphql")
                    .guard(guard::Post())
                    .to(graphql_request),
            )
            .service(
                web::resource("/api/graphql")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
            .app_data(web::Data::new(OAuth::new(urls.backend_url)))
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(Cache::new()))
            .app_data(web::Data::new(jwt))
            .app_data(web::Data::new(Mailer::new(&environment, urls.frontend_url)))
            .service(auth_router())
            .service(health_router());
        }
    }
}
