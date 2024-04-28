use crate::error;
use crate::iff;
use crate::iff_description;
use crate::spr;
use crate::sprite;

use anyhow::Context;

fn build_sprite_description(
    source_directory: &std::path::Path,
    tile_directory: &std::path::Path,
) -> anyhow::Result<spr::Sprite> {
    let mut sprite_frames = Vec::new();
    let mut sprite_frame_index = 0;
    let rotations = [
        sprite::Rotation::NorthWest,
        sprite::Rotation::NorthEast,
        sprite::Rotation::SouthEast,
        sprite::Rotation::SouthWest,
    ];
    let zoom_levels = [sprite::ZoomLevel::Zero, sprite::ZoomLevel::One, sprite::ZoomLevel::Two];
    for zoom_level in zoom_levels {
        for rotation in rotations {
            let sprite_description_file_path =
                sprite::get_sprite_image_description_file_path(tile_directory, zoom_level, rotation);
            if !sprite_description_file_path.is_file() {
                continue;
            }
            let sprite_description = sprite::read_sprite_image_description_file(&sprite_description_file_path)?;

            let sprite_p_file_name = zoom_level.to_string() + "_" + &rotation.to_string() + "_p";
            let sprite_z_file_name = zoom_level.to_string() + "_" + &rotation.to_string() + "_z";
            let sprite_a_file_name = zoom_level.to_string() + "_" + &rotation.to_string() + "_a";

            let sprite_p_file_path = tile_directory.join(&sprite_p_file_name).with_extension("bmp");
            let sprite_z_file_path = tile_directory.join(&sprite_z_file_name).with_extension("bmp");
            let sprite_a_file_path = tile_directory.join(&sprite_a_file_name).with_extension("bmp");

            let sprite_p_file_path = sprite_p_file_path.strip_prefix(source_directory).unwrap();
            let sprite_z_file_path = sprite_z_file_path.strip_prefix(source_directory).unwrap();
            let sprite_a_file_path = sprite_a_file_path.strip_prefix(source_directory).unwrap();

            sprite_frames.push(spr::SpriteFrame::new(
                sprite_frame_index,
                zoom_level,
                rotation,
                &sprite_description,
                sprite_p_file_path,
                sprite_z_file_path,
                sprite_a_file_path,
            ));
            sprite_frame_index += 1;
        }
    }
    let chunk_label = tile_directory.file_name().unwrap().to_str().unwrap();

    let chunk_id = {
        let sprite_id_file_path = tile_directory.join("sprite id").with_extension("json");
        let json_string = std::fs::read_to_string(&sprite_id_file_path)
            .with_context(|| error::file_read_error(&sprite_id_file_path))?;

        serde_json::from_str::<iff::IffChunkId>(&json_string)
            .with_context(|| format!("Failed to deserialize json file {}", sprite_id_file_path.display()))?
    };

    Ok(spr::Sprite::new(
        chunk_label,
        chunk_id,
        sprite_frames.first().unwrap().palette_chunk_id,
        sprite_frames,
    ))
}

pub fn update(source_directory: &std::path::Path, object_name: &str, variant: Option<&str>) -> anyhow::Result<()> {
    let xml_file_path = source_directory.join(object_name).with_extension("xml");

    let mut iff_description = iff_description::IffDescription::open(&xml_file_path)
        .with_context(|| format!("Failed to open xml file {}", xml_file_path.display()))?;

    let mut new_sprites = Vec::new();

    let split_sprites_directory = {
        let object_name = if let Some(variant) = variant {
            format!("{} - {}", object_name, variant)
        } else {
            object_name.to_owned()
        };
        source_directory.join(format!("{} - sprites", object_name))
    };
    for entry in std::fs::read_dir(split_sprites_directory)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        new_sprites.push(build_sprite_description(source_directory, &path)?);
    }

    let sprite_id_set: std::collections::HashSet<_> = new_sprites.iter().map(|x| x.chunk_id).collect();
    assert!(sprite_id_set.len() == new_sprites.len());
    iff_description.sprites.sprites.retain(|x| !sprite_id_set.contains(&x.chunk_id));
    iff_description.sprites.sprites.append(&mut new_sprites);
    iff_description.sprites.sprites.sort_by(|a, b| a.chunk_id.as_i16().cmp(&b.chunk_id.as_i16()));

    iff_description
        .save(&xml_file_path)
        .with_context(|| format!("Failed to save xml file {}", xml_file_path.display()))
}
