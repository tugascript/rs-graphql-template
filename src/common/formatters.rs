// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::error_handling::ServiceError;
use super::regexes::{multi_spaces_regex, new_line_regex};
use slug::slugify;
use uuid::Uuid;

pub fn format_name(name: &str) -> Result<String, ServiceError> {
    let mut title = name.trim().to_lowercase();
    title = new_line_regex()?.replace_all(&title, " ").to_string();
    title = multi_spaces_regex()?.replace_all(&title, " ").to_string();
    let mut c = title.chars();

    match c.next() {
        None => Ok(title),
        Some(f) => Ok(f.to_uppercase().collect::<String>() + c.as_str()),
    }
}

pub fn format_slug(value: &str) -> String {
    let slug = slugify(value);

    if slug.is_empty() {
        return Uuid::new_v4().to_string();
    }

    slug
}

pub fn format_point_slug(value: &str) -> String {
    let slug = slugify(value);

    if slug.is_empty() {
        return Uuid::new_v4().to_string().replace("-", ".");
    }

    slug.replace("-", ".")
}
