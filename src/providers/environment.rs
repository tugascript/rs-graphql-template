// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

#[derive(Clone, Debug)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn new() -> Self {
        match env::var("ENVIRONMENT") {
            Ok(environment) => match environment.to_lowercase().as_str() {
                "development" => Environment::Development,
                "production" => Environment::Production,
                "dev" => Environment::Development,
                "prod" => Environment::Production,
                _ => panic!("Invalid environment."),
            },
            Err(_) => Environment::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        match self {
            Environment::Development => false,
            Environment::Production => true,
        }
    }
}
