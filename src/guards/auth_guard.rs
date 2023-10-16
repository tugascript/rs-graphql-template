// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_graphql::{Error, Guard, Result};

use crate::common::AuthTokens;

pub struct AuthGuard;

impl Guard for AuthGuard {
    fn check(&self, ctx: &async_graphql::Context<'_>) -> Result<()> {
        let tokens = ctx.data::<AuthTokens>()?;

        if tokens.access_token.is_none() {
            return Err(Error::new("Unauthorized"));
        }

        Ok(())
    }
}
