use crate::error;
use crate::iff;
use crate::quantizer;
use crate::sprite;

use anyhow::Context;

const MIN_OBJECT_DIMENSION: i32 = 1;
const MAX_OBJECT_DIMENSION: i32 = 32;

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize)]
struct ObjectDimensions {
    x: i32,
    y: i32,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct FrameDescription {
    name: String,
    sprite_id: iff::IffChunkId,
    palette_id: iff::IffChunkId,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct ObjectDescription {
    dimensions: ObjectDimensions,
    frames: Vec<FrameDescription>,
}

#[allow(clippy::too_many_arguments)]
fn split_sprite(
    full_sprites_directory: &std::path::Path,
    split_sprites_directory: &std::path::Path,
    object_dimensions: ObjectDimensions,
    frame_name: &str,
    rotation: sprite::Rotation,
    zoom_level: sprite::ZoomLevel,
    full_sprite_p: &image::GrayImage,
    full_sprite_a: &image::Rgb32FImage,
    depth_planes: &DepthPlanesView,
    palette: &[[u8; 3]],
    palette_id: iff::IffChunkId,
    transparent_color_index: u8,
) -> anyhow::Result<()> {
    let extra_tiles = (object_dimensions.x - 1) + (object_dimensions.y - 1);

    let (tile_width, tile_height, split_sprite_width, split_sprite_height) = {
        const SPRITE_WIDTH: i32 = 136;
        const SPRITE_HEIGHT: i32 = 384;
        const TILE_WIDTH: i32 = 128;
        const TILE_HEIGHT: i32 = 64;

        match zoom_level {
            sprite::ZoomLevel::Zero => (TILE_WIDTH, TILE_HEIGHT, SPRITE_WIDTH, SPRITE_HEIGHT),
            sprite::ZoomLevel::One => (TILE_WIDTH / 2, TILE_HEIGHT / 2, SPRITE_WIDTH / 2, SPRITE_HEIGHT / 2),
            sprite::ZoomLevel::Two => (TILE_WIDTH / 4, TILE_HEIGHT / 4, SPRITE_WIDTH / 4, SPRITE_HEIGHT / 4),
        }
    };

    let full_sprite_rotation_name = rotation.to_string();
    let full_sprite_z_file_name = zoom_level.to_string() + "_" + &full_sprite_rotation_name + "_depth.exr";
    let full_sprite_z_extra_file_name = zoom_level.to_string() + "_" + &full_sprite_rotation_name + "_depth_extra.exr";
    let full_sprite_z_file_path = full_sprites_directory.join(frame_name).join(full_sprite_z_file_name);
    let full_sprite_z_extra_file_path = full_sprites_directory.join(frame_name).join(full_sprite_z_extra_file_name);
    let mut full_sprite_z = image::open(&full_sprite_z_file_path)
        .with_context(|| error::file_read_error(&full_sprite_z_file_path))?
        .to_rgb32f();
    let mut full_sprite_z_extra = image::open(full_sprite_z_extra_file_path).map(|x| x.to_rgb32f()).ok();

    let mut full_sprite_p = full_sprite_p.clone();
    let mut full_sprite_a = full_sprite_a.clone();

    for tile_y in 0..object_dimensions.y {
        for tile_x in 0..object_dimensions.x {
            let (x_offset, y_offset) = {
                let x_offset_nw = -extra_tiles * (tile_width / 4);
                let y_offset_nw = (object_dimensions.y - object_dimensions.x) * (tile_height / 4);

                let x_offset_ne = (object_dimensions.y - object_dimensions.x) * (tile_width / 4);
                let y_offset_ne = extra_tiles * (tile_height / 4);

                let x_offset_se = extra_tiles * (tile_width / 4);
                let y_offset_se = -(object_dimensions.y - object_dimensions.x) * (tile_height / 4);

                let x_offset_sw = -(object_dimensions.y - object_dimensions.x) * (tile_width / 4);
                let y_offset_sw = -extra_tiles * (tile_height / 4);

                let x_offset_x = tile_x * (tile_width / 2);
                let x_offset_y = tile_y * (tile_width / 2);
                let y_offset_x = tile_x * (tile_height / 2);
                let y_offset_y = tile_y * (tile_height / 2);

                match rotation {
                    sprite::Rotation::NorthWest => (
                        x_offset_nw + x_offset_x + x_offset_y,
                        y_offset_nw + y_offset_x + -y_offset_y,
                    ),
                    sprite::Rotation::NorthEast => (
                        x_offset_ne + x_offset_x + -x_offset_y,
                        y_offset_ne + -y_offset_x + -y_offset_y,
                    ),
                    sprite::Rotation::SouthEast => (
                        x_offset_se + -x_offset_x + -x_offset_y,
                        y_offset_se + -y_offset_x + y_offset_y,
                    ),
                    sprite::Rotation::SouthWest => (
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
            const DEPTH_BOUND_FAR: f64 = 10000.0;

            let tile_depth_offset = -(y_offset as f64 / (tile_height as f64 / 2.0)) * TILE_DEPTH;

            let full_sprite_width = split_sprite_width + (extra_tiles * (tile_width / 2));
            let full_sprite_height = split_sprite_height + (extra_tiles * (tile_width / 2));

            let sub_sprite_x =
                u32::try_from((full_sprite_width / 2) + ((0 - (split_sprite_width / 2)) + x_offset)).unwrap();
            let sub_sprite_y =
                u32::try_from((full_sprite_height / 2) + ((0 - (split_sprite_height / 2)) + y_offset)).unwrap();

            let split_sprite_width = u32::try_from(split_sprite_width).unwrap();
            let split_sprite_height = u32::try_from(split_sprite_height).unwrap();

            use image::GenericImage;
            use image::GenericImageView;
            let mut full_sprite_p =
                full_sprite_p.sub_image(sub_sprite_x, sub_sprite_y, split_sprite_width, split_sprite_height);
            let mut full_sprite_z =
                full_sprite_z.sub_image(sub_sprite_x, sub_sprite_y, split_sprite_width, split_sprite_height);
            let mut full_sprite_a =
                full_sprite_a.sub_image(sub_sprite_x, sub_sprite_y, split_sprite_width, split_sprite_height);

            let mut full_sprite_z_extra = full_sprite_z_extra
                .as_mut()
                .map(|x| x.sub_image(sub_sprite_x, sub_sprite_y, split_sprite_width, split_sprite_height));

            let mut split_sprite_p = image::GrayImage::new(split_sprite_width, split_sprite_height);
            let mut split_sprite_z = image::GrayImage::new(split_sprite_width, split_sprite_height);
            let mut split_sprite_a = image::GrayImage::new(split_sprite_width, split_sprite_height);

            let (rotated_tile_x, rotated_tile_y) = match rotation {
                sprite::Rotation::NorthWest => (tile_x, tile_y),
                sprite::Rotation::NorthEast => (object_dimensions.y - 1 - tile_y, tile_x),
                sprite::Rotation::SouthEast => (object_dimensions.x - 1 - tile_x, object_dimensions.y - 1 - tile_y),
                sprite::Rotation::SouthWest => (tile_y, object_dimensions.x - 1 - tile_x),
            };
            let rotated_object_dimensions = match rotation {
                sprite::Rotation::NorthWest | sprite::Rotation::SouthEast => ObjectDimensions {
                    x: object_dimensions.x,
                    y: object_dimensions.y,
                },
                sprite::Rotation::NorthEast | sprite::Rotation::SouthWest => ObjectDimensions {
                    x: object_dimensions.y,
                    y: object_dimensions.x,
                },
            };

            for x in 0..split_sprite_width {
                for y in 0..split_sprite_height {
                    let alpha = quantizer::posterize_normalized(full_sprite_a.get_pixel(x, y)[0], 3);

                    let left_far_plane_depth = if rotated_tile_x > 0 {
                        depth_planes.left_far.get_pixel(x, y)[0] as f64 + tile_depth_offset
                    } else {
                        DEPTH_BOUND_FAR
                    };
                    let left_near_plane_depth = if rotated_tile_y > 0 {
                        depth_planes.left_near.get_pixel(x, y)[0] as f64 + tile_depth_offset
                    } else {
                        DEPTH_BOUND_NEAR
                    };
                    let right_far_plane_depth = if rotated_tile_y < rotated_object_dimensions.y - 1 {
                        depth_planes.right_far.get_pixel(x, y)[0] as f64 + tile_depth_offset
                    } else {
                        DEPTH_BOUND_FAR
                    };
                    let right_near_plane_depth = if rotated_tile_x < rotated_object_dimensions.x - 1 {
                        depth_planes.right_near.get_pixel(x, y)[0] as f64 + tile_depth_offset
                    } else {
                        DEPTH_BOUND_NEAR
                    };

                    let depth = full_sprite_z.get_pixel(x, y)[0] as f64;
                    let depth = {
                        if !(DEPTH_BOUND_NEAR..DEPTH_BOUND_FAR).contains(&depth) {
                            if let Some(ref full_sprite_z_extra) = full_sprite_z_extra {
                                full_sprite_z_extra.get_pixel(x, y)[0] as f64
                            } else {
                                depth
                            }
                        } else {
                            depth
                        }
                    };

                    if alpha > 0.0
                        && depth >= left_near_plane_depth
                        && depth <= left_far_plane_depth
                        && depth >= right_near_plane_depth
                        && depth <= right_far_plane_depth
                    {
                        split_sprite_p.put_pixel(x, y, full_sprite_p.get_pixel(x, y));

                        split_sprite_a.put_pixel(x, y, image::Luma([(alpha * 255.0) as u8]));

                        let depth_normalized =
                            (DISTANCE_TO_CENTER_FROM_CAMERA + tile_depth_offset + (TILE_DEPTH_FULL_SPAN / 2.0) - depth)
                                / TILE_DEPTH_FULL_SPAN;
                        let depth_u8 = 255 - (depth_normalized.clamp(0.0, 1.0) * 255.0) as u8;
                        split_sprite_z.put_pixel(x, y, image::Luma([depth_u8]));

                        full_sprite_p.put_pixel(x, y, image::Luma([0]));
                        full_sprite_z.put_pixel(x, y, image::Rgb([1.0, 1.0, 1.0]));
                        if let Some(ref mut full_sprite_z_extra) = full_sprite_z_extra {
                            full_sprite_z_extra.put_pixel(x, y, image::Rgb([1.0, 1.0, 1.0]));
                        }
                        full_sprite_a.put_pixel(x, y, image::Rgb([0.0, 0.0, 0.0]));
                    } else {
                        split_sprite_p.put_pixel(x, y, image::Luma([0]));
                        split_sprite_z.put_pixel(x, y, image::Luma([255]));
                    }
                }
            }

            let split_sprite_frame_directory = split_sprites_directory.join(format!("{frame_name} {tile_x}_{tile_y}"));

            if !split_sprite_frame_directory.is_dir() {
                std::fs::create_dir_all(&split_sprite_frame_directory).with_context(|| {
                    format!("Failed to create directory {}", split_sprite_frame_directory.display())
                })?;
            }

            let transmogrified_rotation = rotation.transmogrify();

            let split_sprite_p_file_path = sprite::sprite_channel_file_path(
                &split_sprite_frame_directory,
                zoom_level,
                transmogrified_rotation,
                sprite::Channel::Color,
            );
            let split_sprite_z_file_path = sprite::sprite_channel_file_path(
                &split_sprite_frame_directory,
                zoom_level,
                transmogrified_rotation,
                sprite::Channel::Depth,
            );
            let split_sprite_a_file_path = sprite::sprite_channel_file_path(
                &split_sprite_frame_directory,
                zoom_level,
                transmogrified_rotation,
                sprite::Channel::Alpha,
            );

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

            let sprite_image_description = sprite::calculate_sprite_image_description(
                &split_sprite_a,
                zoom_level,
                palette_id,
                transparent_color_index,
            );
            sprite::write_sprite_image_description_file(
                &sprite_image_description,
                &split_sprite_frame_directory,
                zoom_level,
                transmogrified_rotation,
            )?;
        }
    }

    Ok(())
}

struct DepthPlanes {
    left_far_large: image::Rgb32FImage,
    left_far_medium: image::Rgb32FImage,
    left_far_small: image::Rgb32FImage,
    left_near_large: image::Rgb32FImage,
    left_near_medium: image::Rgb32FImage,
    left_near_small: image::Rgb32FImage,
    right_far_large: image::Rgb32FImage,
    right_far_medium: image::Rgb32FImage,
    right_far_small: image::Rgb32FImage,
    right_near_large: image::Rgb32FImage,
    right_near_medium: image::Rgb32FImage,
    right_near_small: image::Rgb32FImage,
}

struct DepthPlanesView<'a> {
    left_far: &'a image::Rgb32FImage,
    left_near: &'a image::Rgb32FImage,
    right_far: &'a image::Rgb32FImage,
    right_near: &'a image::Rgb32FImage,
}

impl DepthPlanes {
    fn new() -> DepthPlanes {
        DepthPlanes {
            left_far_large: image::load_from_memory(include_bytes!("../res/depth plane left far large.exr"))
                .unwrap()
                .to_rgb32f(),
            left_far_medium: image::load_from_memory(include_bytes!("../res/depth plane left far medium.exr"))
                .unwrap()
                .to_rgb32f(),
            left_far_small: image::load_from_memory(include_bytes!("../res/depth plane left far small.exr"))
                .unwrap()
                .to_rgb32f(),
            left_near_large: image::load_from_memory(include_bytes!("../res/depth plane left near large.exr"))
                .unwrap()
                .to_rgb32f(),
            left_near_medium: image::load_from_memory(include_bytes!("../res/depth plane left near medium.exr"))
                .unwrap()
                .to_rgb32f(),
            left_near_small: image::load_from_memory(include_bytes!("../res/depth plane left near small.exr"))
                .unwrap()
                .to_rgb32f(),
            right_far_large: image::load_from_memory(include_bytes!("../res/depth plane right far large.exr"))
                .unwrap()
                .to_rgb32f(),
            right_far_medium: image::load_from_memory(include_bytes!("../res/depth plane right far medium.exr"))
                .unwrap()
                .to_rgb32f(),
            right_far_small: image::load_from_memory(include_bytes!("../res/depth plane right far small.exr"))
                .unwrap()
                .to_rgb32f(),
            right_near_large: image::load_from_memory(include_bytes!("../res/depth plane right near large.exr"))
                .unwrap()
                .to_rgb32f(),
            right_near_medium: image::load_from_memory(include_bytes!("../res/depth plane right near medium.exr"))
                .unwrap()
                .to_rgb32f(),
            right_near_small: image::load_from_memory(include_bytes!("../res/depth plane right near small.exr"))
                .unwrap()
                .to_rgb32f(),
        }
    }

    fn large(&self) -> DepthPlanesView {
        DepthPlanesView {
            left_far: &self.left_far_large,
            left_near: &self.left_near_large,
            right_far: &self.right_far_large,
            right_near: &self.right_near_large,
        }
    }

    fn medium(&self) -> DepthPlanesView {
        DepthPlanesView {
            left_far: &self.left_far_medium,
            left_near: &self.left_near_medium,
            right_far: &self.right_far_medium,
            right_near: &self.right_near_medium,
        }
    }

    fn small(&self) -> DepthPlanesView {
        DepthPlanesView {
            left_far: &self.left_far_small,
            left_near: &self.left_near_small,
            right_far: &self.right_far_small,
            right_near: &self.right_near_small,
        }
    }
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

fn sinc_normalized(x: f32) -> f32 {
    let x = x * std::f32::consts::PI;

    if x == 0.0 {
        1.0
    } else {
        x.sin() / x
    }
}

fn lanczos_kernel(x: f32, a: f32) -> f32 {
    if x.abs() < a {
        sinc_normalized(x) * sinc_normalized(x / a)
    } else {
        0.0
    }
}

fn downsample_vertical(color: &image::RgbImage, alpha: &image::Rgb32FImage) -> (image::RgbImage, image::Rgb32FImage) {
    let mut indices = Vec::with_capacity(16);
    let mut downsampled_color = image::RgbImage::new(color.width(), color.height() / 2);
    for y in 0..downsampled_color.height() {
        for x in 0..downsampled_color.width() {
            const RATIO: f32 = 2.0;
            const WINDOW: f32 = 3.0;

            let original_y = (y as f32 + 0.5) * RATIO;

            let top_bound = (original_y - (WINDOW * RATIO)).floor().clamp(0.0, color.height() as f32) as u32;
            let bottom_bound = (original_y + (WINDOW * RATIO)).ceil().clamp(0.0, color.height() as f32) as u32;

            let mut contribution_sum = 0.0;
            for y in top_bound..bottom_bound {
                if quantizer::posterize_normalized(alpha.get_pixel(x, y)[0], 3) > 0.0 {
                    let filter_position = (y as f32 - (original_y - 0.5)) / RATIO;
                    let contribution = lanczos_kernel(filter_position, WINDOW);
                    contribution_sum += contribution;
                    indices.push((x, y, contribution));
                }
            }

            let mut downsampled_pixel = [0.0, 0.0, 0.0];
            for (x, y, contribution) in &indices {
                let original_pixel = color.get_pixel(*x, *y);
                downsampled_pixel[0] += srgb_to_linear(original_pixel[0]) * (contribution / contribution_sum);
                downsampled_pixel[1] += srgb_to_linear(original_pixel[1]) * (contribution / contribution_sum);
                downsampled_pixel[2] += srgb_to_linear(original_pixel[2]) * (contribution / contribution_sum);
            }
            downsampled_color.put_pixel(
                x,
                y,
                image::Rgb([
                    linear_to_srgb(downsampled_pixel[0]),
                    linear_to_srgb(downsampled_pixel[1]),
                    linear_to_srgb(downsampled_pixel[2]),
                ]),
            );
            indices.clear();
        }
    }
    (
        downsampled_color,
        image::imageops::resize(
            alpha,
            alpha.width(),
            alpha.height() / 2,
            image::imageops::FilterType::Lanczos3,
        ),
    )
}

fn downsample_horizontal(color: &image::RgbImage, alpha: &image::Rgb32FImage) -> (image::RgbImage, image::Rgb32FImage) {
    let mut indices = Vec::with_capacity(16);
    let mut downsampled_color = image::RgbImage::new(color.width() / 2, color.height());
    for y in 0..downsampled_color.height() {
        for x in 0..downsampled_color.width() {
            const RATIO: f32 = 2.0;
            const WINDOW: f32 = 3.0;

            let original_x = (x as f32 + 0.5) * RATIO;

            let left_bound = (original_x - (WINDOW * RATIO)).floor().clamp(0.0, color.width() as f32) as u32;
            let right_bound = (original_x + (WINDOW * RATIO)).ceil().clamp(0.0, color.width() as f32) as u32;

            let mut contribution_sum = 0.0;
            for x in left_bound..right_bound {
                if quantizer::posterize_normalized(alpha.get_pixel(x, y)[0], 3) > 0.0 {
                    let filter_position = (x as f32 - (original_x - 0.5)) / RATIO;
                    let contribution = lanczos_kernel(filter_position, WINDOW);
                    contribution_sum += contribution;
                    indices.push((x, y, contribution));
                }
            }

            let mut downsampled_pixel = [0.0, 0.0, 0.0];
            for (x, y, contribution) in &indices {
                let original_pixel = color.get_pixel(*x, *y);
                downsampled_pixel[0] += srgb_to_linear(original_pixel[0]) * (contribution / contribution_sum);
                downsampled_pixel[1] += srgb_to_linear(original_pixel[1]) * (contribution / contribution_sum);
                downsampled_pixel[2] += srgb_to_linear(original_pixel[2]) * (contribution / contribution_sum);
            }
            downsampled_color.put_pixel(
                x,
                y,
                image::Rgb([
                    linear_to_srgb(downsampled_pixel[0]),
                    linear_to_srgb(downsampled_pixel[1]),
                    linear_to_srgb(downsampled_pixel[2]),
                ]),
            );
            indices.clear();
        }
    }
    (
        downsampled_color,
        image::imageops::resize(
            alpha,
            alpha.width() / 2,
            alpha.height(),
            image::imageops::FilterType::Lanczos3,
        ),
    )
}

fn downsample_sprites(color: &image::RgbImage, alpha: &image::Rgb32FImage) -> (image::RgbImage, image::Rgb32FImage) {
    let (color, alpha) = downsample_vertical(color, alpha);
    downsample_horizontal(&color, &alpha)
}

pub fn split(source_directory: &std::path::Path, object_name: &str, variant: Option<&str>) -> anyhow::Result<()> {
    let object_description = {
        let object_description_file_name = object_name.to_owned() + " - object description";
        let object_description_file_path = source_directory.join(object_description_file_name).with_extension("json");
        let json_string = std::fs::read_to_string(&object_description_file_path)
            .with_context(|| error::file_read_error(&object_description_file_path))?;

        serde_json::from_str::<ObjectDescription>(&json_string).with_context(|| {
            format!(
                "Failed to deserialize json file {}",
                object_description_file_path.display()
            )
        })?
    };

    anyhow::ensure!(
        object_description.dimensions.x >= MIN_OBJECT_DIMENSION,
        format!("Object dimension x must be at least {}", MIN_OBJECT_DIMENSION)
    );
    anyhow::ensure!(
        object_description.dimensions.y >= MIN_OBJECT_DIMENSION,
        format!("Object dimension y must be at least {}", MIN_OBJECT_DIMENSION)
    );
    anyhow::ensure!(
        object_description.dimensions.x <= MAX_OBJECT_DIMENSION,
        format!("Object dimension x must be {} or under", MAX_OBJECT_DIMENSION)
    );
    anyhow::ensure!(
        object_description.dimensions.y <= MAX_OBJECT_DIMENSION,
        format!("Object dimension y must be {} or under", MAX_OBJECT_DIMENSION)
    );

    anyhow::ensure!(
        !object_description.frames.is_empty(),
        "Failed to find any frames in object description"
    );

    let base_sprite_ids: std::collections::HashSet<_> = object_description.frames.iter().map(|x| x.sprite_id).collect();
    anyhow::ensure!(
        base_sprite_ids.len() == object_description.frames.len(),
        "Each frame must have a unique base sprite ID"
    );
    if base_sprite_ids.len() > 1 {
        let minimum_sprite_id_difference = object_description.dimensions.x * object_description.dimensions.y;
        for ids in base_sprite_ids.iter().collect::<Vec<_>>().windows(2) {
            anyhow::ensure!(
                (ids[0].as_i32() - ids[1].as_i32()).abs() >= minimum_sprite_id_difference,
                format!(
                    "Each base sprite ID must be at least {} apart",
                    minimum_sprite_id_difference
                )
            );
        }
    }

    let mut frame_palette_map = std::collections::HashMap::new();
    for frame_description in &object_description.frames {
        frame_palette_map
            .entry(frame_description.palette_id)
            .or_insert_with(Vec::new)
            .push((frame_description.name.as_str(), frame_description.sprite_id));
    }

    for frame_descriptions in frame_palette_map.values() {
        split_palette(
            source_directory,
            object_name,
            variant,
            object_description.dimensions,
            frame_descriptions,
            object_description.frames[0].palette_id,
        )?;
    }

    Ok(())
}

fn split_palette(
    source_directory: &std::path::Path,
    object_name: &str,
    variant: Option<&str>,
    object_dimensions: ObjectDimensions,
    frame_descriptions: &[(&str, iff::IffChunkId)],
    palette_id: iff::IffChunkId,
) -> anyhow::Result<()> {
    let depth_planes = DepthPlanes::new();

    let object_name = if let Some(variant) = variant {
        format!("{} - {}", object_name, variant)
    } else {
        object_name.to_owned()
    };
    let full_sprites_directory = source_directory.join(format!("{} - full sprites", object_name));
    let split_sprites_directory = source_directory.join(format!("{} - sprites", object_name));
    if split_sprites_directory.is_dir() {
        std::fs::remove_dir_all(&split_sprites_directory)
            .with_context(|| format!("Failed to remove {}", split_sprites_directory.display()))?;
    }

    let mut sprites = Vec::new();

    let mut histogram = quantizer::Histogram::new();

    for (frame_name, _) in frame_descriptions {
        let sprite_count = sprites.len();
        let rotations = [
            sprite::Rotation::NorthWest,
            sprite::Rotation::NorthEast,
            sprite::Rotation::SouthEast,
            sprite::Rotation::SouthWest,
        ];
        for rotation in rotations {
            let full_sprite_frame_directory = full_sprites_directory.join(frame_name);

            let color_sprite_file_name = rotation.to_string() + "_color.png";
            let color_sprite_file_path = full_sprite_frame_directory.join(color_sprite_file_name);
            if !color_sprite_file_path.is_file() {
                continue;
            }
            let color_sprite = image::open(&color_sprite_file_path)
                .with_context(|| error::file_read_error(&color_sprite_file_path))?
                .to_rgb8();

            let alpha_sprite_file_name = rotation.to_string() + "_alpha.exr";
            let alpha_sprite_file_path = full_sprite_frame_directory.join(alpha_sprite_file_name);
            let alpha_sprite = image::open(&alpha_sprite_file_path)
                .with_context(|| error::file_read_error(&alpha_sprite_file_path))?
                .to_rgb32f();

            let (color_sprite, alpha_sprite) = downsample_sprites(&color_sprite, &alpha_sprite);

            let dithered_color_sprite = quantizer::dither_color_sprite_to_r5g6b5(color_sprite.clone());

            histogram.add_colors(&dithered_color_sprite, &alpha_sprite);

            sprites.push((frame_name, rotation, color_sprite, alpha_sprite, dithered_color_sprite));
        }

        anyhow::ensure!(
            sprites.len() > sprite_count,
            "Failed to find any sprites to split in frame \"{}\"",
            frame_name
        );
    }

    let mut quantizer = histogram
        .finalize()
        .with_context(|| format!("No sprites found in {}", full_sprites_directory.display()))?;

    for (frame_name, rotation, color_sprite, alpha_sprite, dithered_color_sprite) in sprites {
        split_sprite(
            &full_sprites_directory,
            &split_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            sprite::ZoomLevel::Zero,
            &quantizer.quantize(&dithered_color_sprite, &alpha_sprite),
            &alpha_sprite,
            &depth_planes.large(),
            &quantizer.palette,
            palette_id,
            quantizer.transparent_color_index,
        )?;

        let (color_sprite, alpha_sprite) = downsample_sprites(&color_sprite, &alpha_sprite);
        let dithered_color_sprite = quantizer::dither_color_sprite_to_r5g6b5(color_sprite.clone());

        split_sprite(
            &full_sprites_directory,
            &split_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            sprite::ZoomLevel::One,
            &quantizer.quantize(&dithered_color_sprite, &alpha_sprite),
            &alpha_sprite,
            &depth_planes.medium(),
            &quantizer.palette,
            palette_id,
            quantizer.transparent_color_index,
        )?;

        let (color_sprite, alpha_sprite) = downsample_sprites(&color_sprite, &alpha_sprite);
        let dithered_color_sprite = quantizer::dither_color_sprite_to_r5g6b5(color_sprite.clone());

        split_sprite(
            &full_sprites_directory,
            &split_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            sprite::ZoomLevel::Two,
            &quantizer.quantize(&dithered_color_sprite, &alpha_sprite),
            &alpha_sprite,
            &depth_planes.small(),
            &quantizer.palette,
            palette_id,
            quantizer.transparent_color_index,
        )?;
    }

    for (frame_name, mut sprite_id) in frame_descriptions {
        for y in 0..object_dimensions.y {
            for x in 0..object_dimensions.x {
                let split_sprite_frame_directory = split_sprites_directory.join(format!("{frame_name} {x}_{y}"));
                if !split_sprite_frame_directory.is_dir() {
                    continue;
                }
                if is_tile_empty(&split_sprite_frame_directory)? {
                    std::fs::remove_dir_all(&split_sprite_frame_directory)
                        .with_context(|| format!("Failed to remove {}", split_sprite_frame_directory.display()))?;
                } else {
                    let tile_sprite_id_file_path =
                        split_sprite_frame_directory.join("sprite id").with_extension("json");
                    let json_string = serde_json::to_string_pretty(&sprite_id).with_context(|| {
                        format!("Failed to serialize json file {}", tile_sprite_id_file_path.display())
                    })?;
                    std::fs::write(&tile_sprite_id_file_path, json_string)
                        .with_context(|| error::file_write_error(&tile_sprite_id_file_path))?;
                }
                sprite_id.advance();
            }
        }
    }

    Ok(())
}

fn is_tile_empty(split_sprite_frame_tile_directory: &std::path::Path) -> anyhow::Result<bool> {
    let rotations = [
        sprite::Rotation::NorthWest,
        sprite::Rotation::NorthEast,
        sprite::Rotation::SouthEast,
        sprite::Rotation::SouthWest,
    ];
    let zoom_levels = [sprite::ZoomLevel::Zero, sprite::ZoomLevel::One, sprite::ZoomLevel::Two];
    for rotation in rotations {
        for zoom_level in zoom_levels {
            let split_sprite_a_file_path = sprite::sprite_channel_file_path(
                split_sprite_frame_tile_directory,
                zoom_level,
                rotation,
                sprite::Channel::Alpha,
            );
            if !split_sprite_a_file_path.is_file() {
                continue;
            }
            let split_sprite_a = image::open(&split_sprite_a_file_path)
                .with_context(|| error::file_read_error(&split_sprite_a_file_path))?
                .to_luma8();
            if split_sprite_a.pixels().any(|x| x[0] != 0) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
