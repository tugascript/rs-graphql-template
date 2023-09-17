// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::dtos::responses::Auth;

#[derive(Serialize, Deserialize, Debug)]
pub enum SignIn {
    Auth(Auth),
    Message(String),
}
