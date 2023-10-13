// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{
    guard,
    web::{resource, Data},
    HttpRequest, HttpResponse, Resource, Result,
};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, MergedObject, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::common::AuthTokens;
use crate::providers::{Database, Jwt, ObjectStorage};

#[derive(MergedObject, Default)]
pub struct MutationRoot;

#[derive(MergedObject, Default)]
pub struct QueryRoot;

pub fn build_schema(
    database: &Database,
    jwt: &Jwt,
    object_storage: ObjectStorage,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(database.to_owned())
    .data(jwt.to_owned())
    .data(object_storage)
    .finish()
}

async fn graphql_post(
    schema: Data<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    req: HttpRequest,
    gql_req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(gql_req.into_inner().data(AuthTokens::new(&req)))
        .await
        .into()
}

async fn graphql_get() -> Result<HttpResponse> {
    let source = playground_source(GraphQLPlaygroundConfig::new("/api/graphql"));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}

pub fn graphql_router() -> Resource {
    resource("/api/graphql")
        .guard(guard::Post())
        .to(graphql_post)
        .guard(guard::Get())
        .to(graphql_get)
}
