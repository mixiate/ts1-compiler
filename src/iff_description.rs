use crate::dgrp;
use crate::error;
use crate::objd;
use crate::slot;
use crate::spr;
use crate::sprite;

use anyhow::Context;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct IffDescription {
    #[serde(rename = "@objectfilename")]
    pub iff_file_path_relative: String,
    #[serde(rename = "@exportobjectdefinitions")]
    exportobjectdefinitions: i32,
    #[serde(rename = "@exportslots")]
    exportslots: i32,
    #[serde(rename = "@exportdrawgroups")]
    exportdrawgroups: i32,
    #[serde(rename = "@exportbitmaps")]
    exportbitmaps: i32,
    #[serde(rename = "@exportsprites")]
    exportsprites: i32,
    #[serde(rename = "@justchangecolors")]
    justchangecolors: i32,
    #[serde(rename = "@exportallzooms")]
    exportallzooms: i32,
    #[serde(rename = "@smoothsmallzoomcolors")]
    smoothsmallzoomcolors: i32,
    #[serde(rename = "@smoothsmallzoomedges")]
    smoothsmallzoomedges: i32,
    #[serde(rename = "@exportexpanded")]
    exportexpanded: i32,
    #[serde(rename = "@exportp")]
    exportp: i32,
    #[serde(rename = "@exportz")]
    exportz: i32,
    #[serde(rename = "@generatez")]
    generatez: i32,
    #[serde(rename = "@generatezfar")]
    generatezfar: i32,
    #[serde(rename = "@exporta")]
    exporta: i32,
    #[serde(rename = "@generatea")]
    generatea: i32,
    #[serde(rename = "@generateasoft")]
    generateasoft: i32,
    #[serde(rename = "@compressbitmaps")]
    compressbitmaps: i32,
    #[serde(rename = "@createsubdirectories")]
    createsubdirectories: i32,
    #[serde(rename = "@thingstodo")]
    thingstodo: i32,
    #[serde(rename = "objectdefinitions", deserialize_with = "deserialize_object_definitions")]
    pub object_definitions: ObjectDefinitions,
    #[serde(rename = "slots")]
    pub slots: Slots,
    #[serde(rename = "drawgroups", deserialize_with = "deserialize_draw_groups")]
    pub draw_groups: DrawGroups,
    #[serde(rename = "sprites", deserialize_with = "spr::deserialize_sprites")]
    pub sprites: Sprites,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectDefinitions {
    #[serde(default, rename = "objectdefinition")]
    pub object_definitions: Vec<objd::ObjectDefinition>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Slots {
    #[serde(default, rename = "slot")]
    pub slots: Vec<slot::Slot>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawGroups {
    #[serde(default, rename = "drawgroup")]
    pub draw_groups: Vec<dgrp::DrawGroup>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sprites {
    #[serde(default, rename = "sprite")]
    pub sprites: Vec<spr::Sprite>,
}

impl IffDescription {
    pub fn open(xml_file_path: &std::path::Path) -> anyhow::Result<IffDescription> {
        let iff_description = std::fs::read_to_string(xml_file_path)?;
        let iff_description = quick_xml::de::from_str::<IffDescription>(&iff_description)?;

        let object_definitions = &iff_description.object_definitions.object_definitions;
        let slots = &iff_description.slots.slots;
        let draw_groups = &iff_description.draw_groups.draw_groups;
        let sprites = &iff_description.sprites.sprites;

        let slot_ids = slots.iter().map(|slot| slot.chunk_id).collect::<std::collections::HashSet<_>>();
        let draw_group_ids = draw_groups.iter().map(|x| x.chunk_id).collect::<std::collections::HashSet<_>>();
        let sprite_ids = sprites.iter().map(|x| x.chunk_id).collect::<std::collections::HashSet<_>>();

        for object_definition in object_definitions {
            if object_definition.slot_chunk_id.as_i16() != 0 {
                anyhow::ensure!(
                    slot_ids.contains(&object_definition.slot_chunk_id),
                    "failed to find slot {} used in object definition {} {}",
                    object_definition.slot_chunk_id.as_i16(),
                    object_definition.chunk_id.as_i16(),
                    object_definition.chunk_label
                );
            }

            if object_definition.subindex != -1 {
                for i in 0..object_definition.draw_group_count {
                    let draw_group_chunk_id = object_definition.base_draw_group_chunk_id + i;
                    anyhow::ensure!(
                        draw_group_ids.contains(&draw_group_chunk_id),
                        "failed to find draw group {} used in object definition {} {}",
                        draw_group_chunk_id.as_i16(),
                        object_definition.chunk_id.as_i16(),
                        object_definition.chunk_label
                    );
                }
                for i in 0..object_definition.dynamic_sprite_count {
                    let sprite_chunk_id = object_definition.base_dynamic_sprite_chunk_id + i;
                    anyhow::ensure!(
                        sprite_ids.contains(&sprite_chunk_id),
                        "failed to find dynamic sprite {} used in object definition {} {}",
                        sprite_chunk_id.as_i16(),
                        object_definition.chunk_id.as_i16(),
                        object_definition.chunk_label
                    );
                }
            }
        }

        for draw_group in draw_groups {
            for (i, draw_group_item_list) in draw_group.draw_group_item_lists.iter().enumerate() {
                for draw_group_item in &draw_group_item_list.draw_group_items {
                    if let Some(sprite) = sprites.iter().find(|x| x.chunk_id == draw_group_item.sprite_chunk_id) {
                        anyhow::ensure!(
                            sprite.sprite_frame_count > draw_group_item.sprite_index.as_i32(),
                            "failed to find frame {} of sprite {} used in draw group {} {} item list {}",
                            draw_group_item.sprite_index.as_i32(),
                            draw_group_item.sprite_chunk_id.as_i16(),
                            draw_group.chunk_id.as_i16(),
                            draw_group.chunk_label,
                            i,
                        );
                    } else {
                        anyhow::bail!(
                            "failed to find sprite {} used in draw group {} {} item list {}",
                            draw_group_item.sprite_chunk_id.as_i16(),
                            draw_group.chunk_id.as_i16(),
                            draw_group.chunk_label,
                            i,
                        );
                    }
                }
            }
        }

        Ok(iff_description)
    }

    pub fn save(&self, xml_file_path: &std::path::Path) -> anyhow::Result<()> {
        let xml_header = include_str!("../res/header.xml");

        let mut buffer = xml_header.to_owned();
        let mut serializer = quick_xml::se::Serializer::with_root(&mut buffer, Some("objectsexportedfromthesims"))?;
        serializer.indent(' ', 2);
        use serde::Serialize;
        self.serialize(serializer)?;

        Ok(std::fs::write(xml_file_path, &buffer)?)
    }

    pub fn update_sprite_variants(&mut self, variant_original: &str, variant_new: &str) -> anyhow::Result<()> {
        let variant_original = " - ".to_owned() + variant_original + " - sprites";
        let variant_new = " - ".to_owned() + variant_new + " - sprites";

        for sprite in &mut self.sprites.sprites {
            if sprite.sprite_type == spr::SpriteType::Spr1 {
                continue;
            }
            for frame in &mut sprite.sprite_frames {
                let sprite_file_path =
                    frame.sprite_channel_file_path_relative_mut(spr::SpriteChannelType::Color, sprite.chunk_id)?;
                *sprite_file_path = sprite_file_path.replacen(&variant_original, &variant_new, 1);
            }
        }
        Ok(())
    }

    pub fn update_sprite_positions(&mut self, source_directory: &std::path::Path) -> anyhow::Result<()> {
        for sprite in &mut self.sprites.sprites {
            if sprite.sprite_type == spr::SpriteType::Spr1 {
                continue;
            }
            for frame in &mut sprite.sprite_frames {
                frame.palette_chunk_id = sprite.palette_chunk_id;

                let alpha_sprite_file_path = source_directory
                    .join(frame.sprite_channel_file_path_relative(spr::SpriteChannelType::Alpha, sprite.chunk_id)?);
                let sprite_frame_directory = alpha_sprite_file_path.parent().with_context(|| {
                    format!(
                        "Failed to get sprite frame directory from sprite file path {}",
                        alpha_sprite_file_path.display()
                    )
                })?;

                let sprite_description_file_path = sprite::get_sprite_image_description_file_path(
                    sprite_frame_directory,
                    frame.zoom_level,
                    frame.rotation,
                );

                let sprite_image_description = if sprite_description_file_path.is_file() {
                    sprite::read_sprite_image_description_file(&sprite_description_file_path)?
                } else {
                    let sprite_image = image::open(&alpha_sprite_file_path)
                        .with_context(|| error::file_read_error(&alpha_sprite_file_path))?
                        .to_luma8();
                    let sprite_image_description = sprite::calculate_sprite_image_description(
                        &sprite_image,
                        frame.zoom_level,
                        sprite.palette_chunk_id,
                        frame.transparent_color_index,
                    );
                    frame.bounds_left = sprite_image_description.bounds.left;
                    frame.bounds_top = sprite_image_description.bounds.top;
                    frame.bounds_right = sprite_image_description.bounds.right;
                    frame.bounds_bottom = sprite_image_description.bounds.bottom;
                    continue;
                };

                sprite.palette_chunk_id = sprite_image_description.palette_id;
                frame.palette_chunk_id = sprite_image_description.palette_id;
                frame.transparent_color_index = sprite_image_description.transparent_color_index;

                frame.bounds_left = sprite_image_description.bounds.left;
                frame.bounds_top = sprite_image_description.bounds.top;
                frame.bounds_right = sprite_image_description.bounds.right;
                frame.bounds_bottom = sprite_image_description.bounds.bottom;

                for draw_group in self.draw_groups.draw_groups.iter_mut() {
                    for draw_group_item_list in &mut draw_group.draw_group_item_lists {
                        for draw_group_item in &mut draw_group_item_list.draw_group_items {
                            if sprite.chunk_id == draw_group_item.sprite_chunk_id
                                && draw_group_item.sprite_index == frame.index
                            {
                                let offset_x = if draw_group_item.flags & 0b1 == 0 {
                                    sprite_image_description.offsets.x
                                } else {
                                    sprite_image_description.offsets.x_flipped
                                };
                                draw_group_item.sprite_offset_x = offset_x;
                                draw_group_item.sprite_offset_y = sprite_image_description.offsets.y;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn deserialize_object_definitions<'de, D>(deserializer: D) -> Result<ObjectDefinitions, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let object_definitions = ObjectDefinitions::deserialize(deserializer)?;

    let objds = &object_definitions.object_definitions;

    if objds.is_empty() {
        return Err(serde::de::Error::custom("no object definitions found"));
    }

    let chunk_ids: std::collections::HashSet<_> = objds.iter().map(|x| x.chunk_id).collect();
    if chunk_ids.len() != objds.len() {
        return Err(serde::de::Error::custom(
            "object definitions contain entries with the same chunk ID",
        ));
    }

    let guids: std::collections::HashSet<_> = objds.iter().map(|x| x.guid).collect();
    if guids.len() != objds.len() {
        return Err(serde::de::Error::custom(
            "object definitions contain entries with the same GUID",
        ));
    }

    Ok(object_definitions)
}

fn deserialize_draw_groups<'de, D>(deserializer: D) -> Result<DrawGroups, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let draw_groups = DrawGroups::deserialize(deserializer)?;

    let rotations = [
        sprite::Rotation::SouthEast,
        sprite::Rotation::NorthEast,
        sprite::Rotation::NorthWest,
        sprite::Rotation::SouthWest,
    ];
    let zoom_levels = [sprite::ZoomLevel::Zero, sprite::ZoomLevel::One, sprite::ZoomLevel::Two];

    for draw_group in &draw_groups.draw_groups {
        for (i, draw_group_item_list) in draw_group.draw_group_item_lists.iter().enumerate() {
            if draw_group_item_list.rotation != rotations[i % 4] {
                return Err(serde::de::Error::custom(format!(
                    "incorrect rotation in draw group {} {} item list {}",
                    draw_group.chunk_id.as_i16(),
                    draw_group.chunk_label,
                    i,
                )));
            }
            if draw_group_item_list.zoom_level != zoom_levels[i / 4] {
                return Err(serde::de::Error::custom(format!(
                    "incorrect zoom level in draw group {} {} item list {}",
                    draw_group.chunk_id.as_i16(),
                    draw_group.chunk_label,
                    i,
                )));
            }
        }
    }

    Ok(draw_groups)
}
