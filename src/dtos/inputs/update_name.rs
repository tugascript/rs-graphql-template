// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::{CustomValidator, InputObject, InputValueError};

use crate::common::{validate_name, validations_handler};

#[derive(InputObject, Debug)]
pub struct UpdateName {
    pub first_name: String,
    pub last_name: String,
}

pub struct UpdateNameValidator;

impl CustomValidator<UpdateName> for UpdateNameValidator {
    fn check(&self, value: &UpdateName) -> Result<(), InputValueError<UpdateName>> {
        let validations = [
            validate_name("First name", &value.first_name)?,
            validate_name("Last name", &value.last_name)?,
        ];
        validations_handler(&validations)?;
        Ok(())
    }
}
