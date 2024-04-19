use crate::dgrp;
use crate::error;

use anyhow::Context;

pub struct SpriteBounds {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

pub struct SpriteOffsets {
    pub x: i32,
    pub y: i32,
    pub x_flipped: i32,
}

pub struct SpriteDescription {
    pub bounds: SpriteBounds,
    pub offsets: SpriteOffsets,
}

fn get_sprite_description_file_path(sprite_file_path: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
    let sprite_file_path = sprite_file_path.to_str().unwrap();

    let sprite_file_path = sprite_file_path
        .strip_suffix("_p.bmp")
        .or_else(|| sprite_file_path.strip_suffix("_z.bmp").or_else(|| sprite_file_path.strip_suffix("_a.bmp")))
        .with_context(|| format!("Failed to find sprite description file path for {}", sprite_file_path))?;
    let sprite_file_path = sprite_file_path.to_owned() + " description.txt";
    Ok(sprite_file_path.into())
}

pub fn read_sprite_description_file(sprite_file_path: &std::path::Path) -> anyhow::Result<SpriteDescription> {
    let sprite_description_file_path = get_sprite_description_file_path(sprite_file_path)?;
    let sprite_description = std::fs::read_to_string(&sprite_description_file_path)
        .with_context(|| error::file_read_error(&sprite_description_file_path))?;
    let sprite_description: Vec<i16> = sprite_description
        .split(' ')
        .map(|x| {
            x.parse::<i16>()
                .with_context(|| format!("Failed to parse {}", sprite_description_file_path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    #[allow(clippy::get_first)]
    Ok(SpriteDescription {
        bounds: SpriteBounds {
            left: *sprite_description.get(0).unwrap(),
            top: *sprite_description.get(1).unwrap(),
            right: *sprite_description.get(2).unwrap(),
            bottom: *sprite_description.get(3).unwrap(),
        },
        offsets: SpriteOffsets {
            x: i32::from(*sprite_description.get(4).unwrap()),
            y: i32::from(*sprite_description.get(5).unwrap()),
            x_flipped: i32::from(*sprite_description.get(6).unwrap()),
        },
    })
}

pub fn write_sprite_description_file(
    sprite_description: &SpriteDescription,
    sprite_file_path: &std::path::Path,
) -> anyhow::Result<()> {
    let sprite_description_file_path = get_sprite_description_file_path(sprite_file_path)?;
    std::fs::write(
        &sprite_description_file_path,
        format!(
            "{} {} {} {} {} {} {}",
            sprite_description.bounds.left,
            sprite_description.bounds.top,
            sprite_description.bounds.right,
            sprite_description.bounds.bottom,
            sprite_description.offsets.x,
            sprite_description.offsets.y,
            sprite_description.offsets.x_flipped,
        ),
    )
    .with_context(|| error::file_write_error(&sprite_description_file_path))
}

pub fn calculate_sprite_description(alpha_sprite: &image::GrayImage, zoom_level: dgrp::ZoomLevel) -> SpriteDescription {
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

    SpriteDescription {
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
    }
}
