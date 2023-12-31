// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use entities::enums::OAuthProviderEnum;

use crate::common::{ServiceError, SOMETHING_WENT_WRONG};

#[derive(Debug)]
pub enum ExternalProvider {
    Google,
    Facebook,
}

const GOOGLE: &'static str = "google";
const FACEBOOK: &'static str = "facebook";

impl ExternalProvider {
    pub fn to_str(&self) -> &str {
        match self {
            ExternalProvider::Google => GOOGLE,
            ExternalProvider::Facebook => FACEBOOK,
        }
    }

    pub fn to_oauth_provider(&self) -> OAuthProviderEnum {
        match self {
            ExternalProvider::Google => OAuthProviderEnum::Google,
            ExternalProvider::Facebook => OAuthProviderEnum::Facebook,
        }
    }
}

#[derive(Clone, Debug)]
struct ClientCredentials {
    client_id: ClientId,
    client_secret: ClientSecret,
}

#[derive(Clone, Debug)]
pub struct OAuth {
    google: ClientCredentials,
    facebook: ClientCredentials,
    url: String,
}

impl OAuth {
    pub fn new(backend_url: String) -> Self {
        let google_client_id = env::var("GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable.");
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable.");
        let facebook_client_id = env::var("FACEBOOK_CLIENT_ID")
            .expect("Missing the FACEBOOK_CLIENT_ID environment variable.");
        let facebook_client_secret = env::var("FACEBOOK_CLIENT_SECRET")
            .expect("Missing the FACEBOOK_CLIENT_SECRET environment variable.");
        Self {
            google: Self::build_client_credentials(google_client_id, google_client_secret),
            facebook: Self::build_client_credentials(facebook_client_id, facebook_client_secret),
            url: format!("{}/api/auth/ext", backend_url),
        }
    }

    pub fn get_external_client(
        &self,
        provider: &ExternalProvider,
    ) -> Result<BasicClient, ServiceError> {
        match provider {
            &ExternalProvider::Google => {
                let auth_url =
                    AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                        .map_err(|e| {
                            ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e))
                        })?;
                let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .map_err(|e| {
                        ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e))
                    })?;
                let redirect_url = RedirectUrl::new(format!("{}/google/callback", &self.url))
                    .map_err(|e| {
                        ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e))
                    })?;

                Ok(BasicClient::new(
                    self.google.client_id.clone(),
                    Some(self.google.client_secret.clone()),
                    auth_url,
                    Some(token_url),
                )
                .set_redirect_uri(redirect_url))
            }
            &ExternalProvider::Facebook => {
                let auth_url =
                    AuthUrl::new("https://www.facebook.com/v18.0/dialog/oauth".to_string())
                        .map_err(|e| {
                            ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e))
                        })?;
                let token_url = TokenUrl::new(
                    "https://graph.facebook.com/v18.0/oauth/access_token".to_string(),
                )
                .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
                let redirect_url = RedirectUrl::new(format!("{}/facebook/callback", &self.url))
                    .map_err(|e| {
                        ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e))
                    })?;

                Ok(BasicClient::new(
                    self.facebook.client_id.clone(),
                    Some(self.facebook.client_secret.clone()),
                    auth_url,
                    Some(token_url),
                )
                .set_redirect_uri(redirect_url))
            }
        }
    }

    pub fn get_external_client_scopes(&self, provider: &ExternalProvider) -> [&str; 3] {
        match provider {
            ExternalProvider::Google => [
                "https://www.googleapis.com/auth/userinfo.email",
                "https://www.googleapis.com/auth/userinfo.profile",
                "https://www.googleapis.com/auth/user.birthday.read",
            ],
            ExternalProvider::Facebook => ["email", "public_profile", "user_birthday"],
        }
    }

    pub fn get_external_client_info_url(&self, provider: &ExternalProvider) -> &str {
        match provider {
            ExternalProvider::Google => "https://www.googleapis.com/oauth2/v3/userinfo",
            ExternalProvider::Facebook => "https://graph.facebook.com/v18.0/me",
        }
    }

    fn build_client_credentials(id: String, secret: String) -> ClientCredentials {
        ClientCredentials {
            client_id: ClientId::new(id),
            client_secret: ClientSecret::new(secret),
        }
    }
}
