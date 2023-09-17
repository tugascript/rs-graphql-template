// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, &str> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt);

    match hash {
        Ok(value) => Ok(value.to_string()),
        Err(_) => Err("Could not hash password, please try again"),
    }
}

pub fn verify_password<'a>(password: &'a str, str_hash: &'a str) -> bool {
    if let Ok(value) = PasswordHash::new(&str_hash) {
        return Argon2::default()
            .verify_password(password.as_bytes(), &value)
            .is_ok();
    }

    false
}
