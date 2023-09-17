// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// use async_graphql::{Context, Error, Result};

// use crate::{
//     auth::helpers::{decode_access_token, AccessToken},
//     config::Jwt,
//     gql_set_up::AuthTokens,
// };

// pub fn get_access_user(ctx: &Context<'_>) -> Result<AccessToken> {
//     let tokens = ctx.data::<AuthTokens>()?;
//     let access_token = tokens
//         .access_token
//         .as_ref()
//         .ok_or(Error::new("Unauthorized"))?;
//     let jwt = ctx.data::<Jwt>()?;

//     match decode_access_token(access_token, &jwt.access.public_key) {
//         Ok(user) => Ok(user),
//         Err(_) => Err(Error::new("Unauthorized")),
//     }
// }
