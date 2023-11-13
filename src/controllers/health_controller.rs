// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use actix_web::{web, HttpResponse, Scope};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn health_router() -> Scope {
    web::scope("/health-check").route("/", web::get().to(health_check))
}
