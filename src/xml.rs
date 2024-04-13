use crate::dgrp;
use crate::objd;
use crate::slot;
use crate::spr;
use crate::sprite;

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
    #[serde(rename = "objectdefinitions")]
    pub object_definitions: ObjectDefinitions,
    #[serde(rename = "slots")]
    pub slots: Slots,
    #[serde(rename = "drawgroups")]
    pub draw_groups: DrawGroups,
    #[serde(rename = "sprites")]
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
    pub fn update_sprite_positions(&mut self, source_directory: &std::path::Path) {
        for sprite in &mut self.sprites.sprites {
            if sprite.sprite_type == spr::SpriteType::Spr1 {
                continue;
            }
            for frame in &mut sprite.sprite_frames {
                frame.palette_chunk_id = sprite.palette_chunk_id;

                let alpha_sprite_file_path = source_directory.join(
                    &frame
                        .sprite_channels
                        .iter()
                        .find(|x| x.channel_type == spr::SpriteChannelType::Alpha)
                        .unwrap()
                        .file_path_relative,
                );
                let sprite_description =
                    sprite::read_sprite_description_file(&alpha_sprite_file_path).unwrap_or_else(|| {
                        let sprite_image = image::open(&alpha_sprite_file_path).unwrap().to_luma8();
                        sprite::calculate_sprite_description(&sprite_image, frame.zoom_level)
                    });

                frame.bounds_left = sprite_description.bounds.left;
                frame.bounds_top = sprite_description.bounds.top;
                frame.bounds_right = sprite_description.bounds.right;
                frame.bounds_bottom = sprite_description.bounds.bottom;

                for draw_group in self.draw_groups.draw_groups.iter_mut() {
                    for draw_group_item_list in &mut draw_group.draw_group_item_lists {
                        for draw_group_item in &mut draw_group_item_list.draw_group_items {
                            if sprite.chunk_id == draw_group_item.sprite_chunk_id
                                && draw_group_item.sprite_index == frame.index
                            {
                                let offset_x = if draw_group_item.flags & 0b1 == 0 {
                                    sprite_description.offsets.x
                                } else {
                                    sprite_description.offsets.x_flipped
                                };
                                draw_group_item.x = offset_x;
                                draw_group_item.y = sprite_description.offsets.y;
                            }
                        }
                    }
                }
            }
        }
    }
}
