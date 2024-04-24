use crate::error;
use crate::quantizer;
use crate::sprite;

use anyhow::Context;

const MIN_OBJECT_DIMENSION: i32 = 1;
const MAX_OBJECT_DIMENSION: i32 = 32;

#[derive(Copy, Clone)]
pub struct ObjectDimensions {
    pub x: i32,
    pub y: i32,
}

#[allow(clippy::too_many_arguments)]
fn split_sprite(
    full_sprites_directory: &std::path::Path,
    split_sprites_directory: &std::path::Path,
    clipped_sprites_directory: &std::path::Path,
    object_dimensions: ObjectDimensions,
    frame_name: &str,
    rotation: sprite::Rotation,
    zoom_level: sprite::ZoomLevel,
    full_sprite_p: &image::GrayImage,
    full_sprite_a: &image::Rgb32FImage,
    depth_plane_far: &image::Rgb32FImage,
    depth_plane_near: &image::Rgb32FImage,
    palette: &[[u8; 3]],
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

    let transmogrified_rotation = rotation.transmogrify();

    for y in 0..object_dimensions.y {
        for x in 0..object_dimensions.x {
            let split_sprite_frame_directory = split_sprites_directory.join(format!("{frame_name} {x}_{y}"));

            let transmogrified_rotation_name = transmogrified_rotation.to_string();
            let split_sprite_p_file_name = zoom_level.to_string() + "_" + &transmogrified_rotation_name + "_p.bmp";
            let split_sprite_z_file_name = zoom_level.to_string() + "_" + &transmogrified_rotation_name + "_z.bmp";
            let split_sprite_a_file_name = zoom_level.to_string() + "_" + &transmogrified_rotation_name + "_a.bmp";

            let split_sprite_p_file_path = split_sprite_frame_directory.join(&split_sprite_p_file_name);
            let split_sprite_z_file_path = split_sprite_frame_directory.join(&split_sprite_z_file_name);
            let split_sprite_a_file_path = split_sprite_frame_directory.join(&split_sprite_a_file_name);

            let (x_offset, y_offset) = {
                let x_offset_nw = -extra_tiles * (tile_width / 4);
                let y_offset_nw = (object_dimensions.y - object_dimensions.x) * (tile_height / 4);

                let x_offset_ne = (object_dimensions.y - object_dimensions.x) * (tile_width / 4);
                let y_offset_ne = extra_tiles * (tile_height / 4);

                let x_offset_se = extra_tiles * (tile_width / 4);
                let y_offset_se = -(object_dimensions.y - object_dimensions.x) * (tile_height / 4);

                let x_offset_sw = -(object_dimensions.y - object_dimensions.x) * (tile_width / 4);
                let y_offset_sw = -extra_tiles * (tile_height / 4);

                let x_offset_x = x * (tile_width / 2);
                let x_offset_y = y * (tile_width / 2);
                let y_offset_x = x * (tile_height / 2);
                let y_offset_y = y * (tile_height / 2);

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
            const DEPTH_BOUND_FAR: f64 = 100.0;

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

            let mut split_sprite_p = image::GrayImage::new(split_sprite_width, split_sprite_height);
            let mut split_sprite_z = image::GrayImage::new(split_sprite_width, split_sprite_height);
            let mut split_sprite_a = image::GrayImage::new(split_sprite_width, split_sprite_height);

            for x in 0..split_sprite_width {
                for y in 0..split_sprite_height {
                    let alpha = quantizer::posterize_normalized(full_sprite_a.get_pixel(x, y)[0], 3);

                    let (near_plane_depth, far_plane_depth) = if object_dimensions.x == 1 && object_dimensions.y == 1 {
                        (DEPTH_BOUND_NEAR, DEPTH_BOUND_FAR)
                    } else {
                        (
                            depth_plane_near.get_pixel(x, y)[0] as f64 + tile_depth_offset,
                            depth_plane_far.get_pixel(x, y)[0] as f64 + tile_depth_offset,
                        )
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

                    if alpha > 0.0 && depth >= near_plane_depth && depth <= far_plane_depth {
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

            let sprite_image_description =
                sprite::calculate_sprite_image_description(&split_sprite_a, zoom_level, transparent_color_index);
            sprite::write_sprite_image_description_file(
                &sprite_image_description,
                &split_sprite_frame_directory,
                zoom_level,
                transmogrified_rotation,
            )?;
        }
    }

    write_clipped_sprite(
        clipped_sprites_directory,
        frame_name,
        rotation,
        zoom_level,
        full_sprite_a,
    )?;

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
    for y in 0..downsized_alpha.height() {
        for x in 0..downsized_alpha.width() {
            let alpha_values = [
                alpha.get_pixel(x * 2, y * 2)[0],
                alpha.get_pixel((x * 2) + 1, y * 2)[0],
                alpha.get_pixel(x * 2, (y * 2) + 1)[0],
                alpha.get_pixel((x * 2) + 1, (y * 2) + 1)[0],
            ];
            let average = alpha_values.iter().sum::<f32>() / 4.0;
            downsized_alpha.put_pixel(x, y, image::Rgb([average, average, average]));
        }
    }
    downsized_alpha
}

pub fn split(
    source_directory: &std::path::Path,
    object_name: &str,
    variant: Option<&str>,
    object_dimensions: ObjectDimensions,
    frame_names: &[String],
) -> anyhow::Result<()> {
    anyhow::ensure!(
        object_dimensions.x >= MIN_OBJECT_DIMENSION,
        format!("Object dimension x must be at least {}", MIN_OBJECT_DIMENSION)
    );
    anyhow::ensure!(
        object_dimensions.y >= MIN_OBJECT_DIMENSION,
        format!("Object dimension y must be at least {}", MIN_OBJECT_DIMENSION)
    );
    anyhow::ensure!(
        object_dimensions.x <= MAX_OBJECT_DIMENSION,
        format!("Object dimension x must be {} or under", MAX_OBJECT_DIMENSION)
    );
    anyhow::ensure!(
        object_dimensions.y <= MAX_OBJECT_DIMENSION,
        format!("Object dimension y must be {} or under", MAX_OBJECT_DIMENSION)
    );

    let depth_planes = DepthPlanes {
        far_large: image::load_from_memory(include_bytes!("../res/depth plane far large.exr")).unwrap().to_rgb32f(),
        far_medium: image::load_from_memory(include_bytes!("../res/depth plane far medium.exr")).unwrap().to_rgb32f(),
        far_small: image::load_from_memory(include_bytes!("../res/depth plane far small.exr")).unwrap().to_rgb32f(),
        near_large: image::load_from_memory(include_bytes!("../res/depth plane near large.exr")).unwrap().to_rgb32f(),
        near_medium: image::load_from_memory(include_bytes!("../res/depth plane near medium.exr")).unwrap().to_rgb32f(),
        near_small: image::load_from_memory(include_bytes!("../res/depth plane near small.exr")).unwrap().to_rgb32f(),
    };

    let object_name = if let Some(variant) = variant {
        format!("{} - {}", object_name, variant)
    } else {
        object_name.to_owned()
    };
    let full_sprites_directory = source_directory.join(format!("{} - Full Sprites", object_name));
    let split_sprites_directory = source_directory.join(format!("{} - Sprites", object_name));
    let clipped_sprites_directory = source_directory.join(format!("{} - Clipped Sprites", object_name));

    let mut sprites = Vec::new();

    let mut histogram = quantizer::Histogram::new();

    for frame_name in frame_names {
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

            let color_sprite = downsize_color_sprite(&color_sprite, &alpha_sprite);
            let alpha_sprite = downsize_alpha_sprite(&alpha_sprite);

            let dithered_color_sprite = quantizer::dither_color_sprite_to_r5g6b5(color_sprite.clone());

            histogram.add_colors(&dithered_color_sprite, &alpha_sprite);

            sprites.push((frame_name, rotation, color_sprite, alpha_sprite, dithered_color_sprite));
        }

        for y in 0..MAX_OBJECT_DIMENSION {
            for x in 0..MAX_OBJECT_DIMENSION {
                let split_sprite_frame_directory = split_sprites_directory.join(format!("{frame_name} {x}_{y}"));
                if split_sprite_frame_directory.is_dir() {
                    std::fs::remove_dir_all(&split_sprite_frame_directory)
                        .with_context(|| format!("Failed to remove {}", split_sprite_frame_directory.display()))?;
                }
            }
        }
    }

    let mut quantizer = histogram
        .finalize()
        .with_context(|| format!("No sprites found in {}", full_sprites_directory.display()))?;

    for (frame_name, rotation, color_sprite, alpha_sprite, dithered_color_sprite) in sprites {
        split_sprite(
            &full_sprites_directory,
            &split_sprites_directory,
            &clipped_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            sprite::ZoomLevel::Zero,
            &quantizer.quantize(&dithered_color_sprite, &alpha_sprite),
            &alpha_sprite,
            &depth_planes.far_large,
            &depth_planes.near_large,
            &quantizer.palette,
            quantizer.transparent_color_index,
        )?;

        let color_sprite = downsize_color_sprite(&color_sprite, &alpha_sprite);
        let alpha_sprite = downsize_alpha_sprite(&alpha_sprite);
        let dithered_color_sprite = quantizer::dither_color_sprite_to_r5g6b5(color_sprite.clone());

        split_sprite(
            &full_sprites_directory,
            &split_sprites_directory,
            &clipped_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            sprite::ZoomLevel::One,
            &quantizer.quantize(&dithered_color_sprite, &alpha_sprite),
            &alpha_sprite,
            &depth_planes.far_medium,
            &depth_planes.near_medium,
            &quantizer.palette,
            quantizer.transparent_color_index,
        )?;

        let color_sprite = downsize_color_sprite(&color_sprite, &alpha_sprite);
        let alpha_sprite = downsize_alpha_sprite(&alpha_sprite);
        let dithered_color_sprite = quantizer::dither_color_sprite_to_r5g6b5(color_sprite.clone());

        split_sprite(
            &full_sprites_directory,
            &split_sprites_directory,
            &clipped_sprites_directory,
            object_dimensions,
            frame_name,
            rotation,
            sprite::ZoomLevel::Two,
            &quantizer.quantize(&dithered_color_sprite, &alpha_sprite),
            &alpha_sprite,
            &depth_planes.far_small,
            &depth_planes.near_small,
            &quantizer.palette,
            quantizer.transparent_color_index,
        )?;
    }

    for y in 0..object_dimensions.y {
        for x in 0..object_dimensions.x {
            for frame_name in frame_names {
                let split_sprite_frame_directory = split_sprites_directory.join(format!("{frame_name} {x}_{y}"));
                if is_tile_empty(&split_sprite_frame_directory)? {
                    std::fs::remove_dir_all(&split_sprite_frame_directory)
                        .with_context(|| format!("Failed to remove {}", split_sprite_frame_directory.display()))?;
                }
            }
        }
    }

    if clipped_sprites_directory.is_dir() {
        remove_empty_directories(&clipped_sprites_directory)
            .with_context(|| format!("Failed to remove {}", clipped_sprites_directory.display()))?;
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
            let split_sprite_a_file_name = zoom_level.to_string() + "_" + &rotation.to_string() + "_a.bmp";
            let split_sprite_a_file_path = split_sprite_frame_tile_directory.join(&split_sprite_a_file_name);
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

fn write_clipped_sprite(
    clipped_sprites_directory: &std::path::Path,
    frame_name: &str,
    rotation: sprite::Rotation,
    zoom_level: sprite::ZoomLevel,
    mut alpha: image::Rgb32FImage,
) -> anyhow::Result<()> {
    for pixel in alpha.pixels_mut() {
        pixel.0.iter_mut().for_each(|x| *x = quantizer::posterize_normalized(*x, 3));
    }

    let clipped_sprites_frame_directory = clipped_sprites_directory.join(frame_name);

    let clipped_sprite_file_name = zoom_level.to_string() + "_" + &rotation.to_string() + "_alpha.png";
    let clipped_sprite_file_path = clipped_sprites_directory.join(frame_name).join(clipped_sprite_file_name);

    if clipped_sprite_file_path.is_file() {
        std::fs::remove_file(&clipped_sprite_file_path)
            .with_context(|| format!("Failed to remove {}", clipped_sprite_file_path.display()))?;
    }

    if alpha.pixels().any(|x| x[0] != 0.0) {
        if !clipped_sprites_frame_directory.is_dir() {
            std::fs::create_dir_all(&clipped_sprites_frame_directory).with_context(|| {
                format!(
                    "Failed to create directory {}",
                    clipped_sprites_frame_directory.display()
                )
            })?;
        }
        image::DynamicImage::ImageRgb32F(alpha)
            .into_luma8()
            .save(&clipped_sprite_file_path)
            .with_context(|| error::file_write_error(&clipped_sprite_file_path))?;
    }

    Ok(())
}

fn remove_empty_directories(path: &std::path::Path) -> std::io::Result<()> {
    for entry in path.read_dir()? {
        let entry = entry?;
        if entry.path().is_file() {
            return Ok(());
        }
        remove_empty_directories(&entry.path())?;
    }
    if path.read_dir()?.next().is_none() {
        std::fs::remove_dir(path)?;
    }
    Ok(())
}
