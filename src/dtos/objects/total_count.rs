// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct TotalCount {
    pub total_count: u64,
    pub previous_count: u64,
}

impl TotalCount {
    pub fn new(total_count: u64, previous_count: u64) -> Self {
        Self {
            total_count,
            previous_count,
        }
    }
}
