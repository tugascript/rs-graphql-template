// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use rusoto_core::{credential::StaticProvider, HttpClient, Region};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use uuid::Uuid;

use crate::common::{ServiceError, INTERNAL_SERVER_ERROR};

use super::Environment;

#[derive(Clone)]
pub struct ObjectStorage {
    client: S3Client,
    bucket: String,
    endpoint: String,
    namespace: Uuid,
}

impl ObjectStorage {
    pub fn new(environment: &Environment) -> Self {
        let object_storage_host = env::var("OBJECT_STORAGE_HOST")
            .expect("Missing the OBJECT_STORAGE_HOST environment variable.");
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
                &Environment::Development => Uuid::new_v4().to_string(),
                &Environment::Production => {
                    panic!("Missing the OBJECT_STORAGE_HOST environment variable.")
                }
            });
        let domain = match environment {
            &Environment::Development => object_storage_host,
            &Environment::Production => {
                format!("{}.{}", object_storage_region, object_storage_host)
            }
        };

        let namespace = Uuid::parse_str(&object_storage_namespace).unwrap();
        let region = Region::Custom {
            name: object_storage_region,
            endpoint: match environment {
                &Environment::Development => format!("http://{}", &domain),
                &Environment::Production => format!("https://{}", &domain),
            },
        };
        let client = S3Client::new_with(
            HttpClient::new().expect("Failed to create HTTP client"),
            StaticProvider::new(
                object_storage_access_key,
                object_storage_secret_key,
                None,
                None,
            ),
            region,
        );
        Self {
            client,
            endpoint: match environment {
                &Environment::Development => {
                    format!("http://{}/{}", domain, &object_storage_bucket)
                }
                &Environment::Production => {
                    format!("https://{}.{}", &object_storage_bucket, domain)
                }
            },
            bucket: object_storage_bucket,
            namespace,
        }
    }

    pub async fn upload_file(
        &self,
        user_id: i32,
        file_key: &Uuid,
        file_extension: &str,
        file_contents: Vec<u8>,
    ) -> Result<String, ServiceError> {
        let user_prefix = Uuid::new_v5(&self.namespace, user_id.to_string().as_bytes()).to_string();
        let combined_key = format!(
            "{}/{}.{}",
            &user_prefix,
            file_key.to_string(),
            file_extension
        );
        let request = PutObjectRequest {
            bucket: self.bucket.to_string(),
            key: combined_key.clone(),
            body: Some(file_contents.into()),
            acl: Some("public-read".to_string()),
            ..Default::default()
        };
        self.client
            .put_object(request)
            .await
            .map_err(|e| ServiceError::internal_server_error(INTERNAL_SERVER_ERROR, Some(e)))?;
        Ok(format!("{}/{}", self.endpoint, combined_key))
    }

    pub async fn delete_file(&self, file_key: &str) -> Result<(), ServiceError> {
        let request = rusoto_s3::DeleteObjectRequest {
            bucket: self.bucket.to_string(),
            key: file_key.to_string(),
            ..Default::default()
        };
        self.client
            .delete_object(request)
            .await
            .map_err(|e| ServiceError::internal_server_error(INTERNAL_SERVER_ERROR, Some(e)))?;
        Ok(())
    }

    pub fn get_user_prefix(&self, user_id: i32) -> String {
        Uuid::new_v5(&self.namespace, user_id.to_string().as_bytes()).to_string()
    }
}
