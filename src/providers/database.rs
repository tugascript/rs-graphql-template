// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use anyhow::Error;
use sea_orm::DatabaseConnection;

#[derive(Clone, Debug)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        let database_url =
            env::var("DATABASE_URL").expect("Missing the DATABASE_URL environment variable.");
        let connection = sea_orm::Database::connect(&database_url).await?;

        Ok(Self { connection })
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
