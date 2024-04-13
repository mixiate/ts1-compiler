use crate::dgrp;
use crate::iff;

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SpriteIndex(u32);

impl SpriteIndex {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum SpriteType {
    #[serde(rename = "1")]
    Spr1,
    #[serde(rename = "2")]
    Spr2,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sprite {
    #[serde(rename = "@name")]
    pub chunk_label: String,
    #[serde(rename = "@id")]
    pub chunk_id: iff::ChunkId,
    #[serde(rename = "@type")]
    pub sprite_type: SpriteType,
    #[serde(rename = "@multitile")]
    multi_tile: i32,
    #[serde(rename = "@defaultpaletteid")]
    pub palette_chunk_id: iff::ChunkId,
    #[serde(rename = "@framecount")]
    pub frame_count: i32,
    #[serde(rename = "@iscustomwallstyle")]
    is_custom_wall_style: i32,
    #[serde(rename = "spriteframe")]
    pub sprite_frames: Vec<SpriteFrame>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteFrame {
    #[serde(rename = "@index")]
    pub index: SpriteIndex,
    #[serde(rename = "@zoom")]
    pub zoom_level: dgrp::ZoomLevel,
    #[serde(rename = "@rot")]
    rotation: i32,
    #[serde(rename = "@x")]
    pub bounds_left: i16,
    #[serde(rename = "@y")]
    pub bounds_top: i16,
    #[serde(skip)]
    pub bounds_right: i16,
    #[serde(skip)]
    pub bounds_bottom: i16,
    #[serde(rename = "@width")]
    pub width: i16,
    #[serde(rename = "@height")]
    pub height: i16,
    #[serde(rename = "@paletteid")]
    pub palette_chunk_id: iff::ChunkId,
    #[serde(rename = "@transparentpixel")]
    pub transparent_pixel_index: u8,
    #[serde(rename = "spritechannel")]
    pub sprite_channels: Vec<SpriteChannel>,
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum SpriteChannelType {
    #[serde(rename = "p")]
    Colour,
    #[serde(rename = "z")]
    Depth,
    #[serde(rename = "a")]
    Alpha,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteChannel {
    #[serde(rename = "@type")]
    pub channel_type: SpriteChannelType,
    #[serde(rename = "@filename")]
    pub file_path_relative: String,
}
