// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::common::{validate_email, validate_not_empty, validations_handler, ServiceError};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignIn {
    pub email: String,
    pub password: String,
}

impl SignIn {
    pub fn validate(self) -> Result<Self, ServiceError> {
        let validations = [
            validate_email(&self.email)?,
            validate_not_empty("Password", &self.password),
        ];
        validations_handler(&validations)?;
        Ok(self)
    }
}
