use crate::dgrp;
use crate::error;

use anyhow::Context;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SpriteBounds {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SpriteOffsets {
    pub x: i32,
    pub y: i32,
    pub x_flipped: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SpriteImageDescription {
    pub bounds: SpriteBounds,
    pub offsets: SpriteOffsets,
    pub transparent_color_index: u8,
}

fn get_sprite_image_description_file_path(
    sprite_frame_directory: &std::path::Path,
    zoom_level: dgrp::ZoomLevel,
    rotation: dgrp::Rotation,
) -> std::path::PathBuf {
    let description_file_name = format!("{zoom_level}_{rotation} description",);
    sprite_frame_directory.join(description_file_name).with_extension("json")
}

pub fn read_sprite_image_description_file(
    sprite_frame_directory: &std::path::Path,
    zoom_level: dgrp::ZoomLevel,
    rotation: dgrp::Rotation,
) -> anyhow::Result<SpriteImageDescription> {
    let sprite_image_description_file_path =
        get_sprite_image_description_file_path(sprite_frame_directory, zoom_level, rotation);
    let json_string = std::fs::read_to_string(&sprite_image_description_file_path)
        .with_context(|| error::file_read_error(&sprite_image_description_file_path))?;

    serde_json::from_str::<SpriteImageDescription>(&json_string).with_context(|| {
        format!(
            "Failed to deserialize json file {}",
            sprite_image_description_file_path.display()
        )
    })
}

pub fn write_sprite_image_description_file(
    sprite_image_description: &SpriteImageDescription,
    sprite_frame_directory: &std::path::Path,
    zoom_level: dgrp::ZoomLevel,
    rotation: dgrp::Rotation,
) -> anyhow::Result<()> {
    let sprite_image_description_file_path =
        get_sprite_image_description_file_path(sprite_frame_directory, zoom_level, rotation);
    let json_string = serde_json::to_string_pretty(&sprite_image_description).with_context(|| {
        format!(
            "Failed to serialize json file {}",
            sprite_image_description_file_path.display()
        )
    })?;
    std::fs::write(&sprite_image_description_file_path, json_string)
        .with_context(|| error::file_write_error(&sprite_image_description_file_path))
}

pub fn calculate_sprite_image_description(
    alpha_sprite: &image::GrayImage,
    zoom_level: dgrp::ZoomLevel,
    transparent_color_index: u8,
) -> SpriteImageDescription {
    let bounds_left = {
        let mut bounds_left = 0;
        'outer: for x in 0..alpha_sprite.width() {
            for y in 0..alpha_sprite.height() {
                if alpha_sprite.get_pixel(x, y).0[0] != 0 {
                    bounds_left = x;
                    break 'outer;
                }
            }
        }
        bounds_left
    };
    let bounds_top = {
        let mut bounds_top = 0;
        'outer: for y in 0..alpha_sprite.height() {
            for x in 0..alpha_sprite.width() {
                if alpha_sprite.get_pixel(x, y).0[0] != 0 {
                    bounds_top = y;
                    break 'outer;
                }
            }
        }
        bounds_top
    };
    let bounds_right = {
        let mut bounds_right = 0;
        'outer: for x in (0..alpha_sprite.width()).rev() {
            for y in 0..alpha_sprite.height() {
                if alpha_sprite.get_pixel(x, y).0[0] != 0 {
                    bounds_right = x;
                    break 'outer;
                }
            }
        }
        bounds_right + 1
    };
    let bounds_bottom = {
        let mut bounds_bottom = 0;
        'outer: for y in (0..alpha_sprite.height()).rev() {
            for x in 0..alpha_sprite.width() {
                if alpha_sprite.get_pixel(x, y).0[0] != 0 {
                    bounds_bottom = y;
                    break 'outer;
                }
            }
        }
        bounds_bottom + 1
    };

    let left_bound_flipped = i32::try_from(alpha_sprite.width()).unwrap() - i32::try_from(bounds_right).unwrap();
    const SPRITE_CENTER_X: i32 = 68;
    const SPRITE_CENTER_Y: i32 = 348;
    let (sprite_center_x, sprite_center_y) = match zoom_level {
        dgrp::ZoomLevel::Zero => (SPRITE_CENTER_X, SPRITE_CENTER_Y),
        dgrp::ZoomLevel::One => (SPRITE_CENTER_X / 2, SPRITE_CENTER_Y / 2),
        dgrp::ZoomLevel::Two => (SPRITE_CENTER_X / 4, SPRITE_CENTER_Y / 4),
    };
    let offset_x = 0 - (sprite_center_x - i32::try_from(bounds_left).unwrap());
    let offset_y = 0 - (sprite_center_y - i32::try_from(bounds_bottom).unwrap());
    let offset_x_flipped = 0 - (sprite_center_x - left_bound_flipped);

    SpriteImageDescription {
        bounds: SpriteBounds {
            left: i16::try_from(bounds_left).unwrap(),
            top: i16::try_from(bounds_top).unwrap(),
            right: i16::try_from(bounds_right).unwrap(),
            bottom: i16::try_from(bounds_bottom).unwrap(),
        },
        offsets: SpriteOffsets {
            x: offset_x,
            y: offset_y,
            x_flipped: offset_x_flipped,
        },
        transparent_color_index,
    }
}
