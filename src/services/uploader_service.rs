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

use async_graphql::{Context, Error, Upload};
use image::{GenericImageView, ImageOutputFormat::Jpeg};
use uuid::Uuid;

use crate::{
    dtos::ratio::Ratio,
    providers::{Jwt, ObjectStorage},
    startup::AuthTokens,
};

use super::helpers::AccessUser;

fn load_file_data(mut file: File) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).map_err(|_| Error::from(""))?;
    Ok(buffer)
}

fn image_processor(
    ctx: &Context<'_>,
    file: Upload,
    ratio: Ratio,
) -> Result<(String, Vec<u8>), Error> {
    let file_info = file.value(ctx).map_err(|_| Error::from(""))?;
    let file_type = file_info.content_type.ok_or(Error::from(""))?;

    if !file_type.contains("image") {
        return Err(Error::from("File is not an image"));
    }

    let image_data = load_file_data(file_info.content)?;
    let image_control = image::load_from_memory(&image_data).map_err(|_| Error::from(""))?;
    let (width, height) = image_control.dimensions();

    let cropped_image = match ratio {
        Ratio::None => image_control,
        Ratio::Square => {
            let size = min(width, height);
            image_control.crop_imm((width - size) / 2, (height - size) / 2, size, size)
        }
        Ratio::Landscape => {
            let height_size = height;
            let width_size = (height * 16) / 9;
            let x_offset = if width_size > width {
                0
            } else {
                (width - width_size) / 2
            };
            let y_offset = 0;
            image_control.crop_imm(x_offset, y_offset, min(width_size, width), height_size)
        }
        Ratio::Portrait => {
            let width_size = width;
            let height_size = (width * 9) / 16;
            let x_offset = 0;
            let y_offset = if height_size > height {
                0
            } else {
                (height - height_size) / 2
            };
            image_control.crop_imm(x_offset, y_offset, width_size, min(height_size, height))
        }
    };
    let mut compressed_buffer = Cursor::new(Vec::<u8>::new());

    cropped_image
        .write_to(&mut compressed_buffer, Jpeg(80))
        .map_err(|_| Error::from(""))?;

    let image_id = Uuid::new_v4().to_string().replace("-", "");
    let image_name = format!("{}.jpg", image_id);
    Ok((image_name, compressed_buffer.into_inner()))
}

pub async fn upload_image(ctx: &Context<'_>, file: Upload, ratio: Ratio) -> Result<String, Error> {
    let (image_key, image_data) = image_processor(ctx, file, ratio)?;
    let tokens = ctx.data::<AuthTokens>()?;
    let jwt = ctx.data::<Jwt>()?;
    let access_user = AccessUser::get_access_user(tokens, jwt)?;
    let file_storage = ctx.data::<ObjectStorage>()?;
    file_storage
        .upload_file(&access_user.id.to_string(), &image_key, image_data)
        .await
}
