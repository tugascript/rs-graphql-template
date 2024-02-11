// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{web::Data, HttpRequest, HttpResponse, Result};
use async_graphql::{
    dataloader::DataLoader,
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, MergedObject, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::data_loaders::SeaOrmLoader;
use crate::{
    helpers::AccessUser,
    providers::{Database, ObjectStorage},
};
use crate::{
    providers::Jwt,
    resolvers::{health_resolver, uploader_resolver, users_resolver},
};

#[derive(MergedObject, Default)]
pub struct MutationRoot(users_resolver::UsersMutation);

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    users_resolver::UsersQuery,
    uploader_resolver::UploaderQuery,
    health_resolver::HealthQuery,
);

pub fn build_schema(
    database: &Database,
    object_storage: ObjectStorage,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(DataLoader::new(
        SeaOrmLoader::new(database),
        tokio::task::spawn,
    ))
    .data(database.to_owned())
    .data(object_storage)
    .finish()
}

pub async fn graphql_request(
    schema: Data<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    jwt: Data<Jwt>,
    req: HttpRequest,
    gql_req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(
            gql_req
                .into_inner()
                .data(AccessUser::from_request(jwt.as_ref(), &req)),
        )
        .await
        .into()
}

pub async fn graphql_playground() -> Result<HttpResponse> {
    let source = playground_source(GraphQLPlaygroundConfig::new("/api/graphql"));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}
