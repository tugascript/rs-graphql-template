// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use async_graphql::{Error, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use entities::uploaded_file::{Column, Entity};

use crate::common::{InternalCause, ServiceError};
use crate::dtos::objects::UploadedFile;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct FileId(pub String);

pub async fn load_files(
    connection: &DatabaseConnection,
    keys: &[FileId],
) -> Result<HashMap<FileId, UploadedFile>> {
    let ids = keys.iter().map(|key| key.0.as_str()).collect::<Vec<&str>>();
    let files = Entity::find()
        .filter(Column::Id.is_in(ids))
        .all(connection)
        .await
        .map_err(|_| Error::from("Error loading files"))?;

    if files.len() != keys.len() {
        return Err(ServiceError::not_found(
            "File not found",
            Some(InternalCause::new("Keys and fetched files do not match")),
        )
        .into());
    }

    let mut files_map = HashMap::<FileId, UploadedFile>::new();

    for file in files {
        files_map.insert(FileId(file.id.to_string()), file.into());
    }

    Ok(files_map)
}
