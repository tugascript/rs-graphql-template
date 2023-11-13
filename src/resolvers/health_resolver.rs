// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::Object;

use crate::dtos::objects::Message;

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    async fn health_check(&self) -> Message {
        Message::new("OK")
    }
}
