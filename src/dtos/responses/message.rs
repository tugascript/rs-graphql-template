// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: String,
    pub message: String,
}

impl Message {
    pub fn new(message: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.to_string(),
        }
    }
}
