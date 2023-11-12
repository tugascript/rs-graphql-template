// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::common::{
    validate_date, validate_email, validate_name, validate_passwords, validations_handler,
    ServiceError,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUp {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub password1: String,
    pub password2: String,
}

impl SignUp {
    pub fn validate(self) -> Result<Self, ServiceError> {
        let validations = [
            validate_email(&self.email)?,
            validate_name("First name", &self.first_name)?,
            validate_name("Last name", &self.last_name)?,
            validate_date(&self.date_of_birth),
            validate_passwords(&self.password1, &self.password2),
        ];
        validations_handler(&validations)?;
        Ok(self)
    }
}
