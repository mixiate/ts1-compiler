use crate::error;
use crate::iff;

use anyhow::Context;

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum ZoomLevel {
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
    #[serde(rename = "2")]
    Two,
}

impl std::fmt::Display for ZoomLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ZoomLevel::Zero => "large",
            ZoomLevel::One => "medium",
            ZoomLevel::Two => "small",
        };
        write!(f, "{}", string)
    }
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Rotation {
    #[serde(rename = "0")]
    NorthWest,
    #[serde(rename = "1")]
    NorthEast,
    #[serde(rename = "2")]
    SouthEast,
    #[serde(rename = "3")]
    SouthWest,
}

impl Rotation {
    pub fn transmogrify(&self) -> Rotation {
        match self {
            Rotation::NorthWest => Rotation::SouthEast,
            Rotation::NorthEast => Rotation::NorthEast,
            Rotation::SouthEast => Rotation::NorthWest,
            Rotation::SouthWest => Rotation::SouthWest,
        }
    }
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Rotation::NorthWest => "nw",
            Rotation::NorthEast => "ne",
            Rotation::SouthEast => "se",
            Rotation::SouthWest => "sw",
        };
        write!(f, "{}", string)
    }
}

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
    pub width: i16,
    pub height: i16,
    pub bounds: SpriteBounds,
    pub offsets: SpriteOffsets,
    pub palette_id: iff::IffChunkId,
    pub transparent_color_index: u8,
}

pub fn get_sprite_image_description_file_path(
    sprite_frame_directory: &std::path::Path,
    zoom_level: ZoomLevel,
    rotation: Rotation,
) -> std::path::PathBuf {
    let description_file_name = format!("{zoom_level}_{rotation} description",);
    sprite_frame_directory.join(description_file_name).with_extension("json")
}

pub fn read_sprite_image_description_file(
    sprite_image_description_file_path: &std::path::Path,
) -> anyhow::Result<SpriteImageDescription> {
    let json_string = std::fs::read_to_string(sprite_image_description_file_path)
        .with_context(|| error::file_read_error(sprite_image_description_file_path))?;

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
    zoom_level: ZoomLevel,
    rotation: Rotation,
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
    zoom_level: ZoomLevel,
    palette_id: iff::IffChunkId,
    transparent_color_index: u8,
) -> SpriteImageDescription {
    if !alpha_sprite.pixels().any(|x| x[0] != 0) {
        return SpriteImageDescription {
            width: 0,
            height: 0,
            bounds: SpriteBounds {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            offsets: SpriteOffsets {
                x: 0,
                y: 0,
                x_flipped: 0,
            },
            palette_id,
            transparent_color_index,
        };
    }

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
        ZoomLevel::Zero => (SPRITE_CENTER_X, SPRITE_CENTER_Y),
        ZoomLevel::One => (SPRITE_CENTER_X / 2, SPRITE_CENTER_Y / 2),
        ZoomLevel::Two => (SPRITE_CENTER_X / 4, SPRITE_CENTER_Y / 4),
    };
    let offset_x = 0 - (sprite_center_x - i32::try_from(bounds_left).unwrap());
    let offset_y = 0 - (sprite_center_y - i32::try_from(bounds_bottom).unwrap());
    let offset_x_flipped = 0 - (sprite_center_x - left_bound_flipped);

    SpriteImageDescription {
        width: alpha_sprite.width().try_into().unwrap(),
        height: alpha_sprite.height().try_into().unwrap(),
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
        palette_id,
        transparent_color_index,
    }
}
