// Copyright (c) 2023 Afonso Barracha
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    cmp::min,
    fs::File,
    io::{Cursor, Read},
};

use anyhow::Error as AnyHowError;
use async_graphql::{Context, Error, Upload};
use image::{GenericImageView, ImageOutputFormat::Jpeg};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use entities::uploaded_file::{ActiveModel, Entity, Model};

use crate::common::{InternalCause, ServiceError, SOMETHING_WENT_WRONG};
use crate::helpers::AccessUser;
use crate::providers::Database;
use crate::{dtos::ratio::Ratio, providers::ObjectStorage};

type ImageData = Vec<u8>;
type ImageId = String;

fn load_file_data(mut file: File) -> Result<Vec<u8>, ServiceError> {
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    Ok(buffer)
}

fn image_processor(
    ctx: &Context<'_>,
    file: Upload,
    ratio: Ratio,
) -> Result<(ImageId, ImageData), ServiceError> {
    let file_info = file
        .value(ctx)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    let file_type = file_info
        .content_type
        .ok_or(ServiceError::internal_server_error(
            SOMETHING_WENT_WRONG,
            Some(InternalCause::new("File does not have content_type")),
        ))?;

    if !file_type.contains("image") {
        return Err(ServiceError::bad_request::<AnyHowError>("File is not an image", None).into());
    }

    let image_data = load_file_data(file_info.content)?;
    let image_control = image::load_from_memory(&image_data)
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;
    let (width, height) = image_control.dimensions();

    let cropped_image = match ratio {
        // Ratio::None => image_control,
        Ratio::Square => {
            let size = min(width, height);
            image_control.crop_imm((width - size) / 2, (height - size) / 2, size, size)
        } // Ratio::Landscape => {
          //     let height_size = height;
          //     let width_size = (height * 16) / 9;
          //     let x_offset = if width_size > width {
          //         0
          //     } else {
          //         (width - width_size) / 2
          //     };
          //     let y_offset = 0;
          //     image_control.crop_imm(x_offset, y_offset, min(width_size, width), height_size)
          // }
          // Ratio::Portrait => {
          //     let width_size = width;
          //     let height_size = (width * 9) / 16;
          //     let x_offset = 0;
          //     let y_offset = if height_size > height {
          //         0
          //     } else {
          //         (height - height_size) / 2
          //     };
          //     image_control.crop_imm(x_offset, y_offset, width_size, min(height_size, height))
          // }
    };
    let mut compressed_buffer = Cursor::new(Vec::<u8>::new());

    cropped_image
        .write_to(&mut compressed_buffer, Jpeg(80))
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;

    Ok((Uuid::new_v4().to_string(), compressed_buffer.into_inner()))
}

pub async fn upload_image(
    ctx: &Context<'_>,
    user_id: Option<i32>,
    db: Option<&Database>,
    os: Option<&ObjectStorage>,
    file: Upload,
    ratio: Ratio,
) -> Result<Model, Error> {
    tracing::info_span!("uploader_service::upload_image");
    let user_id = match user_id {
        Some(access_user) => access_user,
        None => AccessUser::get_access_user(ctx)?.id,
    };
    let object_storage = match os {
        Some(os) => os,
        None => ctx.data::<ObjectStorage>()?,
    };
    let db = match db {
        Some(db) => db,
        None => ctx.data::<Database>()?,
    };
    let (image_id, image_data) = image_processor(ctx, file, ratio)?;
    let url = object_storage
        .upload_file(user_id, &image_id, image_data)
        .await?;
    let uploaded_file = ActiveModel {
        id: Set(image_id),
        user_id: Set(user_id),
        url: Set(url),
        extension: Set("png".to_string()),
        ..Default::default()
    }
    .insert(db.get_connection())
    .await?;
    Ok(uploaded_file)
}

pub async fn find_one_by_id(db: &Database, id: &str) -> Result<Model, ServiceError> {
    tracing::info_span!("uploader_service::find_one_by_id", %id);
    let uploaded_file = Entity::find_by_id(id)
        .one(db.get_connection())
        .await
        .map_err(|e| ServiceError::internal_server_error(SOMETHING_WENT_WRONG, Some(e)))?;

    if let Some(file) = uploaded_file {
        tracing::info_span!("File found");
        return Ok(file);
    }

    Err(ServiceError::not_found::<AnyHowError>(
        "File not found",
        None,
    ))
}
