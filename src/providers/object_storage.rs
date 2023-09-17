// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use async_graphql::Error;
use rusoto_core::{credential::StaticProvider, HttpClient, Region};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use uuid::Uuid;

#[derive(Clone)]
pub struct ObjectStorage {
    client: S3Client,
    bucket: String,
    endpoint: String,
    namespace: Uuid,
}

impl ObjectStorage {
    pub fn new() -> Self {
        let region = env::var("BUCKET_REGION").unwrap();
        let host = env::var("BUCKET_HOST").unwrap();
        let bucket = env::var("BUCKET_NAME").unwrap();
        let access_key = env::var("BUCKET_ACCESS_KEY").unwrap();
        let secret_key = env::var("BUCKET_SECRET_KEY").unwrap();
        let namespace_string = env::var("USER_NAMESPACE").unwrap();

        let endpoint = format!("https://{}.{}.com", region, host);
        let namespace = Uuid::parse_str(&namespace_string).unwrap();

        let region = Region::Custom {
            name: "custom".to_string(),
            endpoint: endpoint.to_string(),
        };
        let client = S3Client::new_with(
            HttpClient::new().unwrap(),
            StaticProvider::new(access_key, secret_key, None, None),
            region,
        );
        Self {
            client,
            bucket,
            endpoint,
            namespace,
        }
    }

    pub async fn upload_file(
        &self,
        user_id: &str,
        file_key: &str,
        file_contents: Vec<u8>,
    ) -> Result<String, Error> {
        let combined_key = format!(
            "{}/{}",
            Uuid::new_v5(&self.namespace, user_id.as_bytes())
                .to_string()
                .replace("-", ""),
            file_key
        );
        let request = PutObjectRequest {
            bucket: self.bucket.to_string(),
            key: combined_key.clone(),
            body: Some(file_contents.into()),
            ..Default::default()
        };
        self.client
            .put_object(request)
            .await
            .map_err(|_| Error::from("Something went wrong uploading the file"))?;
        Ok(format!("{}/{}", self.endpoint, combined_key))
    }

    pub async fn delete_file(&self, file_key: &str) -> Result<(), Error> {
        let request = rusoto_s3::DeleteObjectRequest {
            bucket: self.bucket.to_string(),
            key: file_key.to_string(),
            ..Default::default()
        };
        self.client
            .delete_object(request)
            .await
            .map_err(|_| Error::from("Something went wrong deleting the file"))?;
        Ok(())
    }
}
