// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::common::{validate_jwt, validate_passwords, validations_handler, ServiceError};

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPassword {
    pub reset_token: String,
    pub password1: String,
    pub password2: String,
}

impl ResetPassword {
    pub fn validate(self) -> Result<Self, ServiceError> {
        let validations = [
            validate_jwt("Reset token", &self.reset_token)?,
            validate_passwords(&self.password1, &self.password2),
        ];
        validations_handler(&validations)?;
        Ok(self)
    }
}
