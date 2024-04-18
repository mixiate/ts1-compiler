use crate::dgrp;
use crate::error;
use crate::quantizer;
use crate::sprite;

use anyhow::Context;

fn rotation_names(rotation: dgrp::Rotation) -> (&'static str, &'static str) {
    match rotation {
        dgrp::Rotation::NorthWest => ("nw", "se"),
        dgrp::Rotation::NorthEast => ("ne", "ne"),
        dgrp::Rotation::SouthEast => ("se", "nw"),
        dgrp::Rotation::SouthWest => ("sw", "sw"),
    }
}

fn format_split_sprite_frame_name(object_dimensions: (i32, i32), frame_name: &str, x: i32, y: i32) -> String {
    let position = if object_dimensions.0 == 1 && object_dimensions.1 == 1 {
        "".to_owned()
    } else {
        format!(" {x}_{y}")
    };
    frame_name.to_owned() + &position
}

#[allow(clippy::too_many_arguments)]
fn split_sprite(
    full_sprites_directory: &std::path::Path,
    split_sprites_directory: &std::path::Path,
    object_dimensions: (i32, i32),
    frame_name: &str,
    rotation: dgrp::Rotation,
    zoom_level: dgrp::ZoomLevel,
    full_sprite_p: &image::RgbImage,
    full_sprite_a: &image::Rgb32FImage,
    depth_plane_far: &image::Rgb32FImage,
    depth_plane_near: &image::Rgb32FImage,
    quantizer: &imagequant::Attributes,
    quantization_result: &mut imagequant::QuantizationResult,
    palette: &[[u8; 3]],
) -> anyhow::Result<()> {
    let extra_tiles = (object_dimensions.0 - 1) + (object_dimensions.1 - 1);

    let (tile_width, tile_height, size, split_sprite_width, split_sprite_height) = {
        const SPRITE_WIDTH: i32 = 136;
        const SPRITE_HEIGHT: i32 = 384;
        const TILE_WIDTH: i32 = 128;
        const TILE_HEIGHT: i32 = 64;

        match zoom_level {
            dgrp::ZoomLevel::Zero => (TILE_WIDTH, TILE_HEIGHT, "large", SPRITE_WIDTH, SPRITE_HEIGHT),
            dgrp::ZoomLevel::One => (
                TILE_WIDTH / 2,
                TILE_HEIGHT / 2,
                "medium",
                SPRITE_WIDTH / 2,
                SPRITE_HEIGHT / 2,
            ),
            dgrp::ZoomLevel::Two => (
                TILE_WIDTH / 4,
                TILE_HEIGHT / 4,
                "small",
                SPRITE_WIDTH / 4,
                SPRITE_HEIGHT / 4,
            ),
        }
    };

    let (rotation_name, transmogrified_rotation_name) = rotation_names(rotation);

    let full_sprite_z_file_name = size.to_owned() + "_" + rotation_name + "_depth.exr";
    let full_sprite_z_extra_file_name = size.to_owned() + "_" + rotation_name + "_depth_extra.exr";
    let full_sprite_z_file_path = full_sprites_directory.join(frame_name).join(full_sprite_z_file_name);
    let full_sprite_z_extra_file_path = full_sprites_directory.join(frame_name).join(full_sprite_z_extra_file_name);
    let mut full_sprite_z = image::open(&full_sprite_z_file_path)
        .with_context(|| error::file_read_error(&full_sprite_z_file_path))?
        .to_rgb32f();
    let mut full_sprite_z_extra = image::open(full_sprite_z_extra_file_path).map(|x| x.to_rgb32f()).ok();

    let mut full_sprite_p = quantizer::dither_image(quantizer, quantization_result, full_sprite_p, full_sprite_a);
    let mut full_sprite_a = full_sprite_a.clone();

    for y in 0..object_dimensions.1 {
        for x in 0..object_dimensions.0 {
            let split_sprite_frame_name = format_split_sprite_frame_name(object_dimensions, frame_name, x, y);
            let split_sprite_frame_directory = split_sprites_directory.join(split_sprite_frame_name);

            let split_sprite_p_file_name = size.to_owned() + "_" + transmogrified_rotation_name + "_p.bmp";
            let split_sprite_z_file_name = size.to_owned() + "_" + transmogrified_rotation_name + "_z.bmp";
            let split_sprite_a_file_name = size.to_owned() + "_" + transmogrified_rotation_name + "_a.bmp";

            let split_sprite_p_file_path = split_sprite_frame_directory.join(&split_sprite_p_file_name);
            let split_sprite_z_file_path = split_sprite_frame_directory.join(&split_sprite_z_file_name);
            let split_sprite_a_file_path = split_sprite_frame_directory.join(&split_sprite_a_file_name);

            let (x_offset, y_offset) = {
                let x_offset_nw = -extra_tiles * (tile_width / 4);
                let y_offset_nw = (object_dimensions.1 - object_dimensions.0) * (tile_height / 4);

                let x_offset_ne = (object_dimensions.1 - object_dimensions.0) * (tile_width / 4);
                let y_offset_ne = extra_tiles * (tile_height / 4);

                let x_offset_se = extra_tiles * (tile_width / 4);
                let y_offset_se = -(object_dimensions.1 - object_dimensions.0) * (tile_height / 4);

                let x_offset_sw = -(object_dimensions.1 - object_dimensions.0) * (tile_width / 4);
                let y_offset_sw = -extra_tiles * (tile_height / 4);

                let x_offset_x = x * (tile_width / 2);
                let x_offset_y = y * (tile_width / 2);
                let y_offset_x = x * (tile_height / 2);
                let y_offset_y = y * (tile_height / 2);

                match rotation {
                    dgrp::Rotation::NorthWest => (
                        x_offset_nw + x_offset_x + x_offset_y,
                        y_offset_nw + y_offset_x + -y_offset_y,
                    ),
                    dgrp::Rotation::NorthEast => (
                        x_offset_ne + x_offset_x + -x_offset_y,
                        y_offset_ne + -y_offset_x + -y_offset_y,
                    ),
                    dgrp::Rotation::SouthEast => (
                        x_offset_se + -x_offset_x + -x_offset_y,
                        y_offset_se + -y_offset_x + y_offset_y,
                    ),
                    dgrp::Rotation::SouthWest => (
                        x_offset_sw + -x_offset_x + x_offset_y,
                        y_offset_sw + y_offset_x + y_offset_y,
                    ),
                }
            };

            const TILE_DISTANCE_TO_CENTER: f64 = 17.0;
            // √((TILE_DISTANCE_TO_CENTER²) + (TILE_DISTANCE_TO_CENTER²) + ((TILE_DISTANCE_TO_CENTER²) * (√(2/3))²))
            const DISTANCE_TO_CENTER_FROM_CAMERA: f64 = 27.760883751542684;
            const TILE_DEPTH: f64 = TILE_DISTANCE_TO_CENTER / DISTANCE_TO_CENTER_FROM_CAMERA;
            const TILE_DEPTH_FULL_SPAN: f64 = 3.2; // why?

            const DEPTH_BOUND_NEAR: f64 = 1.0;
            const DEPTH_BOUND_FAR: f64 = 100.0;

            let tile_depth_offset = -(y_offset as f64 / (tile_height as f64 / 2.0)) * TILE_DEPTH;

            let full_sprite_width = split_sprite_width + (extra_tiles * (tile_width / 2));
            let full_sprite_height = split_sprite_height + (extra_tiles * (tile_width / 2));

            let sub_sprite_left =
                u32::try_from((full_sprite_width / 2) + ((0 - (split_sprite_width / 2)) + x_offset)).unwrap();
            let sub_sprite_top =
                u32::try_from((full_sprite_height / 2) + ((0 - (split_sprite_height / 2)) + y_offset)).unwrap();

            let split_sprite_width = u32::try_from(split_sprite_width).unwrap();
            let split_sprite_height = u32::try_from(split_sprite_height).unwrap();
            let mut split_sprite_p = image::GrayImage::new(split_sprite_width, split_sprite_height);
            let mut split_sprite_z = image::GrayImage::new(split_sprite_width, split_sprite_height);
            let mut split_sprite_a = image::GrayImage::new(split_sprite_width, split_sprite_height);

            for full_x in sub_sprite_left..(sub_sprite_left + split_sprite_width) {
                for full_y in sub_sprite_top..(sub_sprite_top + split_sprite_height) {
                    let split_x = full_x - sub_sprite_left;
                    let split_y = full_y - sub_sprite_top;

                    let alpha = full_sprite_a.get_pixel(full_x, full_y)[0];

                    let (near_plane_depth, far_plane_depth) = if object_dimensions.0 == 1 && object_dimensions.1 == 1 {
                        (DEPTH_BOUND_NEAR, DEPTH_BOUND_FAR)
                    } else {
                        (
                            depth_plane_near.get_pixel(split_x, split_y)[0] as f64 + tile_depth_offset,
                            depth_plane_far.get_pixel(split_x, split_y)[0] as f64 + tile_depth_offset,
                        )
                    };

                    let depth = full_sprite_z.get_pixel(full_x, full_y)[0] as f64;
                    let depth = {
                        if !(DEPTH_BOUND_NEAR..DEPTH_BOUND_FAR).contains(&depth) {
                            if let Some(ref full_sprite_z_extra) = full_sprite_z_extra {
                                full_sprite_z_extra.get_pixel(full_x, full_y)[0] as f64
                            } else {
                                depth
                            }
                        } else {
                            depth
                        }
                    };

                    if alpha > 0.0 && depth >= near_plane_depth && depth <= far_plane_depth {
                        split_sprite_p.put_pixel(split_x, split_y, *full_sprite_p.get_pixel(full_x, full_y));

                        split_sprite_a.put_pixel(split_x, split_y, image::Luma([(alpha * 255.0) as u8]));

                        let depth_normalized =
                            (DISTANCE_TO_CENTER_FROM_CAMERA + tile_depth_offset + (TILE_DEPTH_FULL_SPAN / 2.0) - depth)
                                / TILE_DEPTH_FULL_SPAN;
                        let depth_u8 = 255 - (depth_normalized.clamp(0.0, 1.0) * 255.0) as u8;
                        split_sprite_z.put_pixel(split_x, split_y, image::Luma([depth_u8]));

                        full_sprite_p.put_pixel(full_x, full_y, image::Luma([0]));
                        full_sprite_z.put_pixel(full_x, full_y, image::Rgb([1.0, 1.0, 1.0]));
                        if let Some(ref mut full_sprite_z_extra) = full_sprite_z_extra {
                            full_sprite_z_extra.put_pixel(full_x, full_y, image::Rgb([1.0, 1.0, 1.0]));
                        }
                        full_sprite_a.put_pixel(full_x, full_y, image::Rgb([0.0, 0.0, 0.0]));
                    } else {
                        split_sprite_p.put_pixel(split_x, split_y, image::Luma([0]));
                        split_sprite_z.put_pixel(split_x, split_y, image::Luma([255]));
                    }
                }
            }

            if !split_sprite_frame_directory.is_dir() {
                std::fs::create_dir_all(&split_sprite_frame_directory).with_context(|| {
                    format!("Failed to create directory {}", split_sprite_frame_directory.display())
                })?;
            }

            {
                let mut output_buffer = Vec::new();
                let mut encoder = image::codecs::bmp::BmpEncoder::new(&mut output_buffer);
                encoder
                    .encode_with_palette(
                        split_sprite_p.as_raw(),
                        split_sprite_p.width(),
                        split_sprite_p.height(),
                        image::ExtendedColorType::L8,
                        Some(palette),
                    )
                    .unwrap();

                let mut file = std::fs::File::create(&split_sprite_p_file_path)
                    .with_context(|| error::file_write_error(&split_sprite_p_file_path))?;
                use std::io::Write;
                file.write_all(&output_buffer).with_context(|| error::file_write_error(&split_sprite_p_file_path))?;
            }
            split_sprite_z
                .save(&split_sprite_z_file_path)
                .with_context(|| error::file_write_error(&split_sprite_z_file_path))?;
            split_sprite_a
                .save(&split_sprite_a_file_path)
                .with_context(|| error::file_write_error(&split_sprite_a_file_path))?;

            let sprite_description = sprite::calculate_sprite_description(&split_sprite_a, zoom_level);
            sprite::write_sprite_description_file(&sprite_description, &split_sprite_p_file_path);
        }
    }
    Ok(())
}

struct DepthPlanes {
    far_large: image::Rgb32FImage,
    far_medium: image::Rgb32FImage,
    far_small: image::Rgb32FImage,
    near_large: image::Rgb32FImage,
    near_medium: image::Rgb32FImage,
    near_small: image::Rgb32FImage,
}

fn srgb_to_linear(srgb: u8) -> f32 {
    let srgb = srgb as f32 / 255.0;
    if srgb <= 0.040448237 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(linear: f32) -> u8 {
    if linear <= 0.0031306685 {
        ((linear * 12.92) * 255.0) as u8
    } else {
        ((1.055 * linear.powf(1.0 / 2.4) - 0.055) * 255.0) as u8
    }
}

fn downsize_color_sprite(color: &image::RgbImage, alpha: &image::Rgb32FImage) -> image::RgbImage {
    let mut downsized_color = image::RgbImage::new(color.width() / 2, color.height() / 2);
    let mut pixels = Vec::with_capacity(4);
    for y in 0..downsized_color.height() {
        for x in 0..downsized_color.width() {
            let original_x = x * 2;
            let original_y = y * 2;
            let indices = [
                (original_x, original_y),
                (original_x + 1, original_y),
                (original_x, original_y + 1),
                (original_x + 1, original_y + 1),
            ];
            for (x, y) in indices {
                if alpha.get_pixel(x, y)[0] > 0.0 {
                    pixels.push(color.get_pixel(x, y));
                }
            }
            let red = linear_to_srgb(
                pixels.iter().fold(0.0, |a, x| a + srgb_to_linear(x[0])) / std::cmp::max(pixels.len(), 1) as f32,
            );
            let green = linear_to_srgb(
                pixels.iter().fold(0.0, |a, x| a + srgb_to_linear(x[1])) / std::cmp::max(pixels.len(), 1) as f32,
            );
            let blue = linear_to_srgb(
                pixels.iter().fold(0.0, |a, x| a + srgb_to_linear(x[2])) / std::cmp::max(pixels.len(), 1) as f32,
            );
            downsized_color.put_pixel(x, y, image::Rgb([red, green, blue]));
            pixels.clear();
        }
    }
    downsized_color
}

fn downsize_alpha_sprite(alpha: &image::Rgb32FImage) -> image::Rgb32FImage {
    let mut downsized_alpha = image::Rgb32FImage::new(alpha.width() / 2, alpha.height() / 2);
    let mut pixels = Vec::with_capacity(4);
    for y in 0..downsized_alpha.height() {
        for x in 0..downsized_alpha.width() {
            let original_x = x * 2;
            let original_y = y * 2;
            let indices = [
                (original_x, original_y),
                (original_x + 1, original_y),
                (original_x, original_y + 1),
                (original_x + 1, original_y + 1),
            ];
            for (x, y) in indices {
                if alpha.get_pixel(x, y)[0] > 0.0 {
                    pixels.push(alpha.get_pixel(x, y)[0]);
                }
            }
            let average = pixels.iter().sum::<f32>() / std::cmp::max(pixels.len(), 1) as f32;
            downsized_alpha.put_pixel(x, y, image::Rgb([average, average, average]));
            pixels.clear();
        }
    }
    downsized_alpha
}

pub fn split(
    full_sprites_directory: &std::path::Path,
    split_sprites_directory: &std::path::Path,
    object_dimensions: (i32, i32),
    frame_names: &[String],
) -> anyhow::Result<()> {
    anyhow::ensure!(object_dimensions.0 > 0, "Object dimension x must be over 0");
    anyhow::ensure!(object_dimensions.0 <= 32, "Object dimension x must be 32 or under");
    anyhow::ensure!(object_dimensions.1 > 0, "Object dimension y must be over 0");
    anyhow::ensure!(object_dimensions.1 <= 32, "Object dimension y must be 32 or under");

    let depth_planes = DepthPlanes {
        far_large: image::load_from_memory(include_bytes!("../res/depth plane far large.exr")).unwrap().to_rgb32f(),
        far_medium: image::load_from_memory(include_bytes!("../res/depth plane far medium.exr")).unwrap().to_rgb32f(),
        far_small: image::load_from_memory(include_bytes!("../res/depth plane far small.exr")).unwrap().to_rgb32f(),
        near_large: image::load_from_memory(include_bytes!("../res/depth plane near large.exr")).unwrap().to_rgb32f(),
        near_medium: image::load_from_memory(include_bytes!("../res/depth plane near medium.exr")).unwrap().to_rgb32f(),
        near_small: image::load_from_memory(include_bytes!("../res/depth plane near small.exr")).unwrap().to_rgb32f(),
    };

    let mut sprites = Vec::new();

    let mut color_set = std::collections::HashSet::new();

    for frame_name in frame_names {
        let rotations = [
            dgrp::Rotation::NorthWest,
            dgrp::Rotation::NorthEast,
            dgrp::Rotation::SouthEast,
            dgrp::Rotation::SouthWest,
        ];
        for rotation in rotations {
            let (rotation_name, _) = rotation_names(rotation);

            let full_sprite_frame_directory = full_sprites_directory.join(frame_name);

            let color_sprite_file_name = rotation_name.to_owned() + "_color.png";
            let color_sprite_file_path = full_sprite_frame_directory.join(color_sprite_file_name);
            if !color_sprite_file_path.is_file() {
                continue;
            }
            let color_sprite = image::open(&color_sprite_file_path)
                .with_context(|| error::file_read_error(&color_sprite_file_path))?
                .to_rgb8();

            let alpha_sprite_file_name = rotation_name.to_owned() + "_alpha.exr";
            let alpha_sprite_file_path = full_sprite_frame_directory.join(alpha_sprite_file_name);
            let alpha_sprite = {
                let mut image_reader = image::io::Reader::open(&alpha_sprite_file_path)
                    .with_context(|| error::file_read_error(&alpha_sprite_file_path))?;
                image_reader.no_limits();
                let mut alpha_sprite =
                    image_reader.decode().with_context(|| error::file_read_error(&alpha_sprite_file_path))?.to_rgb32f();
                for pixel in alpha_sprite.pixels_mut() {
                    for channel in pixel.0.iter_mut() {
                        *channel = (*channel * 32.0).round() / 32.0
                    }
                }
                alpha_sprite
            };

            let color_sprite = downsize_color_sprite(&color_sprite, &alpha_sprite);
            let alpha_sprite = downsize_alpha_sprite(&alpha_sprite);

            for (rgb, a) in color_sprite.pixels().zip(alpha_sprite.pixels()) {
                if a[0] > 0.0 {
                    color_set.insert(*rgb);
                }
            }

            sprites.push((frame_name, rotation, color_sprite, alpha_sprite));
        }
    }

    anyhow::ensure!(
        !color_set.is_empty(),
        format!(
            "Failed to find any sprites in directory {}",
            full_sprites_directory.display()
        )
    );

    let mut quantizer = imagequant::new();
    quantizer.set_speed(1).unwrap();
    let (mut quantization_result, palette) = quantizer::create_color_palette(&color_set, &quantizer);

    for (frame_name, rotation, color_sprite, alpha_sprite) in sprites {
        split_sprite(
            full_sprites_directory,
            split_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            dgrp::ZoomLevel::Zero,
            &color_sprite,
            &alpha_sprite,
            &depth_planes.far_large,
            &depth_planes.near_large,
            &quantizer,
            &mut quantization_result,
            &palette,
        )?;

        let color_sprite = downsize_color_sprite(&color_sprite, &alpha_sprite);
        let alpha_sprite = downsize_alpha_sprite(&alpha_sprite);

        split_sprite(
            full_sprites_directory,
            split_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            dgrp::ZoomLevel::One,
            &color_sprite,
            &alpha_sprite,
            &depth_planes.far_medium,
            &depth_planes.near_medium,
            &quantizer,
            &mut quantization_result,
            &palette,
        )?;

        let color_sprite = downsize_color_sprite(&color_sprite, &alpha_sprite);
        let alpha_sprite = downsize_alpha_sprite(&alpha_sprite);

        split_sprite(
            full_sprites_directory,
            split_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            dgrp::ZoomLevel::Two,
            &color_sprite,
            &alpha_sprite,
            &depth_planes.far_small,
            &depth_planes.near_small,
            &quantizer,
            &mut quantization_result,
            &palette,
        )?;
    }
    Ok(())
}
