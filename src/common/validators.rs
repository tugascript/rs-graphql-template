// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anyhow::Error;
use chrono::NaiveDate;
use unicode_segmentation::UnicodeSegmentation;

use super::{
    error_handling::ServiceError,
    regexes::{email_regex, jwt_regex, name_regex},
    INTERNAL_SERVER_ERROR,
};

#[derive(Default)]
struct PasswordValidity {
    has_lowercase: bool,
    has_uppercase: bool,
    has_number: bool,
    has_symbol: bool,
}

impl PasswordValidity {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub enum ValidatorEnum {
    Valid,
    Invalid(String),
}

pub fn password_characters_validation(password: &str) -> ValidatorEnum {
    let mut validity = PasswordValidity::new();

    for char in password.chars() {
        if char.is_lowercase() {
            validity.has_lowercase = true;
        } else if char.is_uppercase() {
            validity.has_uppercase = true;
        } else if char.is_numeric() {
            validity.has_number = true;
        } else {
            validity.has_symbol = true;
        }
    }

    let mut messages = Vec::<&str>::new();
    if !validity.has_number {
        messages.push("number");
    }
    if !validity.has_lowercase {
        messages.push("lowercase character");
    }
    if !validity.has_uppercase {
        messages.push("uppercase character");
    }
    if !validity.has_symbol {
        messages.push("symbol");
    }

    if messages.is_empty() {
        ValidatorEnum::Valid
    } else {
        ValidatorEnum::Invalid(format!(
            "Password must contain at least one {}.",
            messages.join(", ")
        ))
    }
}

pub fn validate_password(password: &str) -> ValidatorEnum {
    let len = password.graphemes(true).count();

    if len < 8 || len > 40 {
        return ValidatorEnum::Invalid(
            "Password needs to be between 8 and 40 characters.".to_string(),
        );
    }

    password_characters_validation(password)
}

pub fn validate_email(email: &str) -> Result<ValidatorEnum, ServiceError> {
    let len = email.graphemes(true).count();

    if len < 5 || len > 200 {
        return Ok(ValidatorEnum::Invalid(
            "Email needs to be between 5 and 200 characters".to_string(),
        ));
    }
    if !email_regex()?.is_match(email) {
        return Ok(ValidatorEnum::Invalid("Invalid email".to_string()));
    }

    Ok(ValidatorEnum::Valid)
}

pub fn validate_name(name: &str, value: &str) -> Result<ValidatorEnum, ServiceError> {
    let len = value.graphemes(true).count();

    if len < 3 || len > 50 {
        return Ok(ValidatorEnum::Invalid(format!(
            "{} needs to be between 3 and 50 characters.",
            name
        )));
    }
    if !name_regex()?.is_match(value) {
        return Ok(ValidatorEnum::Invalid(format!("Invalid {}", name)));
    }

    Ok(ValidatorEnum::Valid)
}

pub fn validate_date(date: &str) -> ValidatorEnum {
    let len = date.graphemes(true).count();

    if len < 10 {
        return ValidatorEnum::Invalid("Date needs to be in the format YYYY-MM-DD.".to_string());
    }

    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(_) => ValidatorEnum::Valid,
        Err(_) => ValidatorEnum::Invalid("Date needs to be in the format YYYY-MM-DD.".to_string()),
    }
}

pub fn validate_passwords(password1: &str, password2: &str) -> ValidatorEnum {
    if password1.is_empty() {
        return ValidatorEnum::Invalid("Password is required".to_string());
    }
    if password2.is_empty() {
        return ValidatorEnum::Invalid("Password confirmation is required".to_string());
    }
    if password1 != password2 {
        return ValidatorEnum::Invalid("Passwords do not match".to_string());
    }

    validate_password(password1)
}

pub fn validate_jwt(name: &str, jwt: &str) -> Result<ValidatorEnum, ServiceError> {
    let len = jwt.chars().count();

    if len < 20 || len > 500 {
        return Ok(ValidatorEnum::Invalid(format!(
            "{} needs to be between 20 and 500 characters.",
            name
        )));
    }

    if !jwt_regex()?.is_match(jwt) {
        return Ok(ValidatorEnum::Invalid(format!("Invalid {}", name)));
    }

    Ok(ValidatorEnum::Valid)
}

pub fn validate_not_empty(name: &str, value: &str) -> ValidatorEnum {
    if value.is_empty() {
        return ValidatorEnum::Invalid(format!("{} is required", name));
    }

    ValidatorEnum::Valid
}

pub fn validations_handler(validations: &[ValidatorEnum]) -> Result<(), ServiceError> {
    let errors = validations
        .iter()
        .filter_map(|validator| {
            if let ValidatorEnum::Invalid(message) = validator {
                Some(message.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<&str>>();

    if errors.is_empty() {
        return Ok(());
    }

    let errors_json = serde_json::to_string(&errors)
        .map_err(|e| ServiceError::internal_server_error(INTERNAL_SERVER_ERROR, Some(e)))?;
    Err(ServiceError::bad_request::<Error>(&errors_json, None))
}
