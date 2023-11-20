// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use secrecy::Secret;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn is_production(&self) -> bool {
        match self {
            Environment::Development => false,
            Environment::Production => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    environment: Environment,
    host: String,
    port: u16,
    api_id: Secret<String>,
    backend_url: String,
    frontend_url: String,
    database_url: Secret<String>,
    redis_url: Secret<String>,
    jwt_access_secret: Secret<String>,
    jwt_refresh_secret: Secret<String>,
    refresh_name: Secret<String>,
    jwt_confirmation_secret: Secret<String>,
    jwt_reset_secret: Secret<String>,
    jwt_access_expiration: i64,
    jwt_refresh_expiration: i64,
    jwt_confirmation_expiration: i64,
    jwt_reset_expiration: i64,
    email_host: String,
    email_port: u16,
    email_user: String,
    email_password: Secret<String>,
    google_client_id: String,
    google_client_secret: Secret<String>,
    facebook_client_id: String,
    facebook_client_secret: Secret<String>,
    object_storage_host: String,
    object_storage_access_key: Secret<String>,
    object_storage_secret_key: Secret<String>,
    object_storage_bucket: String,
    object_storage_region: String,
    object_storage_namespace: Secret<String>,
}

type Host = String;
type Port = u16;

#[derive(Clone, Debug)]
pub struct SingleJwt {
    pub secret: Secret<String>,
    pub exp: i64,
}

type AccessJWT = SingleJwt;
type RefreshJWT = SingleJwt;
type CookieName = Secret<String>;
type ConfirmationJWT = SingleJwt;
type ResetJWT = SingleJwt;
type EmailHost = String;
type EmailPort = u16;
type EmailUser = String;
type EmailPassword<'a> = &'a Secret<String>;
type ClientId = String;
type ClientSecret<'a> = &'a Secret<String>;
type ObjectStorageRegion = String;
type ObjectStorageHost = String;
type ObjectStorageBucket = String;
type ObjectStorageAccessKey<'a> = &'a Secret<String>;
type ObjectStorageSecretKey<'a> = &'a Secret<String>;
type ObjectStorageNamespace<'a> = &'a Secret<String>;

impl Config {
    pub fn new() -> Self {
        let mut environment = Environment::Development;

        if dotenvy::dotenv().is_err() {
            environment = Environment::Production;
        }

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
        let api_id = env::var("API_ID").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => panic!("Missing the API_ID environment variable."),
        });
        let backend_url = env::var("BACKEND_URL").unwrap_or_else(|_| match environment {
            Environment::Development => format!("http://localhost:{}", port),
            Environment::Production => panic!("Missing the BACKEND_URL environment variable."),
        });
        let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| match environment {
            Environment::Development => "http://localhost:3000".to_string(),
            Environment::Production => panic!("Missing the FRONTEND_URL environment variable."),
        });
        let database_url =
            env::var("DATABASE_URL").expect("Missing the DATABASE_URL environment variable.");
        let redis_url = env::var("REDIS_URL").expect("Missing the REDIS_URL environment variable.");
        let jwt_access_secret = env::var("ACCESS_SECRET").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => {
                panic!("Missing the JWT_ACCESS_SECRET environment variable.")
            }
        });
        let jwt_refresh_secret = env::var("REFRESH_SECRET").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => {
                panic!("Missing the JWT_REFRESH_SECRET environment variable.")
            }
        });
        let jwt_confirmation_secret =
            env::var("CONFIRMATION_SECRET").unwrap_or_else(|_| match environment {
                Environment::Development => Uuid::new_v4().to_string(),
                Environment::Production => {
                    panic!("Missing the JWT_CONFIRMATION_SECRET environment variable.")
                }
            });
        let jwt_reset_secret = env::var("RESET_SECRET").unwrap_or_else(|_| match environment {
            Environment::Development => Uuid::new_v4().to_string(),
            Environment::Production => panic!("Missing the JWT_RESET_SECRET environment variable."),
        });
        let jwt_access_expiration = env::var("ACCESS_EXPIRATION")
            .unwrap_or_else(|_| "600".to_string())
            .parse::<i64>()
            .unwrap_or(600);
        let jwt_refresh_expiration = env::var("REFRESH_EXPIRATION")
            .unwrap_or_else(|_| "259200".to_string())
            .parse::<i64>()
            .unwrap_or(259200);
        let jwt_confirmation_expiration = env::var("CONFIRMATION_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<i64>()
            .unwrap_or(86400);
        let jwt_reset_expiration = env::var("RESET_EXPIRATION")
            .unwrap_or_else(|_| "1800".to_string())
            .parse::<i64>()
            .unwrap_or(1800);
        let refresh_name = env::var("REFRESH_NAME").unwrap_or_else(|_| match environment {
            Environment::Development => "refresh".to_string(),
            Environment::Production => panic!("Missing the REFRESH_NAME environment variable."),
        });
        let email_host = env::var("EMAIL_HOST").unwrap_or_else(|_| match environment {
            Environment::Development => "smtp.mailtrap.io".to_string(),
            Environment::Production => panic!("Missing the EMAIL_HOST environment variable."),
        });
        let email_port = env::var("EMAIL_PORT")
            .expect("Missing the EMAIL_PORT environment variable.")
            .parse::<u16>()
            .expect("EMAIL_PORT must be a number.");
        let email_user =
            env::var("EMAIL_USER").expect("Missing the EMAIL_USER environment variable.");
        let email_password =
            env::var("EMAIL_PASSWORD").expect("Missing the EMAIL_PASSWORD environment variable.");
        let google_client_id = env::var("GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable.");
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable.");
        let facebook_client_id = env::var("FACEBOOK_CLIENT_ID")
            .expect("Missing the FACEBOOK_CLIENT_ID environment variable.");
        let facebook_client_secret = env::var("FACEBOOK_CLIENT_SECRET")
            .expect("Missing the FACEBOOK_CLIENT_SECRET environment variable.");
        let object_storage_host =
            env::var("OBJECT_STORAGE_HOST").unwrap_or_else(|_| match environment {
                Environment::Development => "digitalocean".to_string(),
                Environment::Production => {
                    panic!("Missing the OBJECT_STORAGE_HOST environment variable.")
                }
            });
        let object_storage_access_key = env::var("OBJECT_STORAGE_ACCESS_KEY")
            .expect("Missing the OBJECT_STORAGE_ACCESS_KEY environment variable.");
        let object_storage_secret_key = env::var("OBJECT_STORAGE_SECRET_KEY")
            .expect("Missing the OBJECT_STORAGE_SECRET_KEY environment variable.");
        let object_storage_bucket = env::var("OBJECT_STORAGE_BUCKET")
            .expect("Missing the OBJECT_STORAGE_BUCKET environment variable.");
        let object_storage_region = env::var("OBJECT_STORAGE_REGION")
            .expect("Missing the OBJECT_STORAGE_REGION environment variable.");
        let object_storage_namespace =
            env::var("OBJECT_STORAGE_NAMESPACE").unwrap_or_else(|_| match environment {
                Environment::Development => Uuid::new_v4().to_string(),
                Environment::Production => {
                    panic!("Missing the OBJECT_STORAGE_HOST environment variable.")
                }
            });

        Self {
            environment,
            host,
            port,
            api_id: Secret::new(api_id),
            backend_url,
            frontend_url,
            database_url: Secret::new(database_url),
            redis_url: Secret::new(redis_url),
            jwt_access_secret: Secret::new(jwt_access_secret),
            jwt_refresh_secret: Secret::new(jwt_refresh_secret),
            jwt_confirmation_secret: Secret::new(jwt_confirmation_secret),
            jwt_reset_secret: Secret::new(jwt_reset_secret),
            jwt_access_expiration,
            jwt_refresh_expiration,
            jwt_confirmation_expiration,
            jwt_reset_expiration,
            refresh_name: Secret::new(refresh_name),
            email_host,
            email_port,
            email_user,
            email_password: Secret::new(email_password),
            google_client_id,
            google_client_secret: Secret::new(google_client_secret),
            facebook_client_id,
            facebook_client_secret: Secret::new(facebook_client_secret),
            object_storage_host,
            object_storage_access_key: Secret::new(object_storage_access_key),
            object_storage_secret_key: Secret::new(object_storage_secret_key),
            object_storage_bucket,
            object_storage_region,
            object_storage_namespace: Secret::new(object_storage_namespace),
        }
    }

    pub fn app_config(&self) -> (Host, Port) {
        (self.host.to_owned(), self.port)
    }

    pub fn cache_config(&self) -> &Secret<String> {
        &self.redis_url
    }

    pub fn database_config(&self) -> &Secret<String> {
        &self.database_url
    }

    pub fn jwt_config(&self) -> (AccessJWT, RefreshJWT, ConfirmationJWT, ResetJWT) {
        (
            SingleJwt {
                secret: self.jwt_access_secret.to_owned(),
                exp: self.jwt_access_expiration,
            },
            SingleJwt {
                secret: self.jwt_refresh_secret.to_owned(),
                exp: self.jwt_refresh_expiration,
            },
            SingleJwt {
                secret: self.jwt_confirmation_secret.to_owned(),
                exp: self.jwt_confirmation_expiration,
            },
            SingleJwt {
                secret: self.jwt_reset_secret.to_owned(),
                exp: self.jwt_reset_expiration,
            },
        )
    }

    pub fn refresh_name(&self) -> CookieName {
        self.refresh_name.to_owned()
    }

    pub fn api_id(&self) -> Secret<String> {
        self.api_id.to_owned()
    }

    pub fn email_config(&self) -> (EmailHost, EmailPort, EmailUser, EmailPassword) {
        (
            self.email_host.to_owned(),
            self.email_port,
            self.email_user.to_owned(),
            &self.email_password,
        )
    }

    pub fn frontend_url(&self) -> String {
        self.frontend_url.to_owned()
    }

    pub fn google_config(&self) -> (ClientId, ClientSecret) {
        (self.google_client_id.to_owned(), &self.google_client_secret)
    }

    pub fn facebook_config(&self) -> (ClientId, ClientSecret) {
        (
            self.facebook_client_id.to_owned(),
            &self.facebook_client_secret,
        )
    }

    pub fn backend_url(&self) -> String {
        self.backend_url.to_owned()
    }

    pub fn object_storage_config(
        &self,
    ) -> (
        ObjectStorageRegion,
        ObjectStorageHost,
        ObjectStorageBucket,
        ObjectStorageAccessKey,
        ObjectStorageSecretKey,
        ObjectStorageNamespace,
    ) {
        (
            self.object_storage_region.to_owned(),
            self.object_storage_host.to_owned(),
            self.object_storage_bucket.to_owned(),
            &self.object_storage_access_key,
            &self.object_storage_secret_key,
            &self.object_storage_namespace,
        )
    }

    pub fn get_environment(&self) -> Environment {
        self.environment.to_owned()
    }
}
