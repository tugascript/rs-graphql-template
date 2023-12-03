// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use redis::{aio::Connection, Client};
use std::env;

use crate::common::{ServiceError, INTERNAL_SERVER_ERROR};

#[derive(Clone)]
pub struct Cache {
    client: Client,
}

impl Cache {
    pub fn new() -> Self {
        let redis_url = env::var("REDIS_URL").expect("Missing the REDIS_URL environment variable.");
        let client = Client::open(redis_url).expect("Failed to create Redis client.");
        Self { client }
    }

    pub async fn get_connection(&self) -> Result<Connection, ServiceError> {
        let con = self.client.get_tokio_connection().await;

        match con {
            Ok(con) => Ok(con),
            Err(err) => Err(ServiceError::internal_server_error(
                INTERNAL_SERVER_ERROR,
                Some(err),
            )),
        }
    }
}
