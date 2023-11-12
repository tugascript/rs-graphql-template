// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::Deserialize;

use crate::common::{validate_not_empty, validations_handler, ServiceError};

#[derive(Debug, Deserialize)]
pub struct OAuth {
    pub code: String,
    pub state: String,
}

impl OAuth {
    pub fn validate(self) -> Result<Self, ServiceError> {
        let validations = [
            validate_not_empty("Code", &self.code),
            validate_not_empty("State", &self.state),
        ];
        validations_handler(&validations)?;
        Ok(self)
    }
}
