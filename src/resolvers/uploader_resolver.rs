// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::services::uploader_service;
use async_graphql::{Object, Result};

#[derive(Default)]
pub struct UploaderQuery;

#[Object]
impl UploaderQuery {
    async fn file_by_id(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> Result<crate::dtos::objects::UploadedFile> {
        let db = ctx.data::<crate::providers::Database>()?;
        Ok(uploader_service::find_one_by_id(db, &id).await?.into())
    }
}
