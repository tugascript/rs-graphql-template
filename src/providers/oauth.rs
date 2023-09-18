// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use entities::enums::OAuthProviderEnum;

#[derive(Debug)]
pub enum ExternalProvider {
    Google,
    Facebook,
}

impl ExternalProvider {
    pub fn from_str(provider: &str) -> Option<Self> {
        match provider {
            "google" => Some(Self::Google),
            "facebook" => Some(Self::Facebook),
            _ => None,
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
    secret: String,
}

impl OAuth {
    pub fn new() -> Self {
        let google = Self::build_google();
        let facebook = Self::build_facebook();
        let backend_url =
            env::var("BACKEND_URL").expect("Missing the BACKEND_URL environment variable.");
        let secret =
            env::var("OAUTH_SECRET").expect("Missing the OAUTH_SECRET environment variable.");

        Self {
            google,
            facebook,
            url: format!("{}/api/auth/ext", backend_url),
            secret,
        }
    }

    pub fn get_external_client(&self, provider: &ExternalProvider) -> Result<BasicClient, String> {
        match provider {
            &ExternalProvider::Google => {
                let auth_url =
                    AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                        .map_err(|_| "Something went wrong")?;
                let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .map_err(|_| "Something went wrong")?;
                let redirect_url = RedirectUrl::new(format!("{}/google/callback", &self.url))
                    .map_err(|_| "Something went wrong")?;

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
                        .map_err(|_| "Something went wrong")?;
                let token_url = TokenUrl::new(
                    "https://graph.facebook.com/v18.0/oauth/access_token".to_string(),
                )
                .map_err(|_| "Something went wrong")?;
                let redirect_url = RedirectUrl::new(format!("{}/facebook/callback", &self.url))
                    .map_err(|_| "Something went wrong")?;

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

    fn build_google() -> ClientCredentials {
        let client_id = ClientId::new(
            env::var("GOOGLE_CLIENT_ID")
                .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        );
        let client_secret = ClientSecret::new(
            env::var("GOOGLE_CLIENT_SECRET")
                .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
        );

        ClientCredentials {
            client_id,
            client_secret,
        }
    }

    fn build_facebook() -> ClientCredentials {
        let client_id = ClientId::new(
            env::var("FACEBOOK_CLIENT_ID")
                .expect("Missing the FACEBOOK_CLIENT_ID environment variable."),
        );
        let client_secret = ClientSecret::new(
            env::var("FACEBOOK_CLIENT_SECRET")
                .expect("Missing the FACEBOOK_CLIENT_SECRET environment variable."),
        );

        ClientCredentials {
            client_id,
            client_secret,
        }
    }
}
