// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

impl Auth {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            expires_in,
            refresh_token,
            token_type: "Bearer".to_string(),
        }
    }
}
