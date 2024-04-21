use crate::iff;
use crate::spr;
use crate::sprite;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawGroup {
    #[serde(rename = "@name")]
    pub chunk_label: String,
    #[serde(rename = "@id")]
    pub chunk_id: iff::ChunkId,
    #[serde(rename = "drawgroupitemlist")]
    pub draw_group_item_lists: [DrawGroupItemList; 12],
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawGroupItemList {
    #[serde(
        deserialize_with = "deserialize_draw_group_rotation",
        serialize_with = "serialize_draw_group_rotation",
        rename = "@dirflags"
    )]
    pub rotation: sprite::Rotation,
    #[serde(
        deserialize_with = "deserialize_draw_group_zoom_level",
        serialize_with = "serialize_draw_group_zoom_level",
        rename = "@zoom"
    )]
    pub zoom_level: sprite::ZoomLevel,
    #[serde(rename = "drawgroupitem")]
    pub draw_group_items: Vec<DrawGroupItem>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawGroupItem {
    #[serde(rename = "@spriteid")]
    pub sprite_chunk_id: iff::ChunkId,
    #[serde(rename = "@spritenum")]
    pub sprite_index: spr::SpriteIndex,
    #[serde(rename = "@pixelx")]
    pub sprite_offset_x: i32,
    #[serde(rename = "@pixely")]
    pub sprite_offset_y: i32,
    #[serde(rename = "@xoffset")]
    pub object_offset_x: f32,
    #[serde(rename = "@yoffset")]
    pub object_offset_y: f32,
    #[serde(rename = "@zoffset")]
    pub object_offset_z: f32,
    #[serde(rename = "@flags")]
    pub flags: u32,
}

impl DrawGroup {
    pub fn to_chunk_bytes(&self) -> Vec<u8> {
        const DGRP_HEADER_VERSION: u16 = 20004u16;
        const DGRP_HEADER_IMAGE_COUNT: u32 = 12;

        let mut dgrp_data = Vec::new();

        dgrp_data.extend_from_slice(&DGRP_HEADER_VERSION.to_le_bytes());
        dgrp_data.extend_from_slice(&DGRP_HEADER_IMAGE_COUNT.to_le_bytes());

        for draw_group_item_list in &self.draw_group_item_lists {
            let rotation = match draw_group_item_list.rotation {
                sprite::Rotation::NorthWest => 16u32,
                sprite::Rotation::NorthEast => 4u32,
                sprite::Rotation::SouthEast => 1u32,
                sprite::Rotation::SouthWest => 64u32,
            };
            let zoom_level = match draw_group_item_list.zoom_level {
                sprite::ZoomLevel::Zero => 1u32,
                sprite::ZoomLevel::One => 2u32,
                sprite::ZoomLevel::Two => 3u32,
            };
            let sprite_count = u32::try_from(draw_group_item_list.draw_group_items.len()).unwrap();

            dgrp_data.extend_from_slice(&rotation.to_le_bytes());
            dgrp_data.extend_from_slice(&zoom_level.to_le_bytes());
            dgrp_data.extend_from_slice(&sprite_count.to_le_bytes());

            for draw_group_item in &draw_group_item_list.draw_group_items {
                dgrp_data.extend_from_slice(&draw_group_item.sprite_chunk_id.as_i32().to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.sprite_index.as_u32().to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.sprite_offset_x.to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.sprite_offset_y.to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.object_offset_z.to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.flags.to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.object_offset_x.to_le_bytes());
                dgrp_data.extend_from_slice(&draw_group_item.object_offset_y.to_le_bytes());
            }
        }

        let mut dgrp_chunk = Vec::with_capacity(iff::IFF_CHUNK_HEADER_SIZE + dgrp_data.len());
        let dgrp_chunk_header = iff::ChunkHeader::new("DGRP", dgrp_data.len(), self.chunk_id, &self.chunk_label);
        dgrp_chunk.extend_from_slice(&dgrp_chunk_header.to_bytes());
        dgrp_chunk.extend_from_slice(dgrp_data.as_slice());
        dgrp_chunk
    }
}

fn deserialize_draw_group_rotation<'de, D>(deserializer: D) -> Result<sprite::Rotation, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let string = String::deserialize(deserializer)?;

    const FIELDS: &[&str] = &["1", "4", "16", "64"];
    match string.as_str() {
        "1" => Ok(sprite::Rotation::SouthEast),
        "4" => Ok(sprite::Rotation::NorthEast),
        "16" => Ok(sprite::Rotation::NorthWest),
        "64" => Ok(sprite::Rotation::SouthWest),
        _ => Err(serde::de::Error::unknown_field(&string, FIELDS)),
    }
}

fn serialize_draw_group_rotation<S>(rotation: &sprite::Rotation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    match rotation {
        sprite::Rotation::SouthEast => 1i32.serialize(serializer),
        sprite::Rotation::NorthEast => 4i32.serialize(serializer),
        sprite::Rotation::NorthWest => 16i32.serialize(serializer),
        sprite::Rotation::SouthWest => 64i32.serialize(serializer),
    }
}

fn deserialize_draw_group_zoom_level<'de, D>(deserializer: D) -> Result<sprite::ZoomLevel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let string = String::deserialize(deserializer)?;

    const FIELDS: &[&str] = &["1", "2", "3"];
    match string.as_str() {
        "1" => Ok(sprite::ZoomLevel::Zero),
        "2" => Ok(sprite::ZoomLevel::One),
        "3" => Ok(sprite::ZoomLevel::Two),
        _ => Err(serde::de::Error::unknown_field(&string, FIELDS)),
    }
}

fn serialize_draw_group_zoom_level<S>(zoom_level: &sprite::ZoomLevel, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    match zoom_level {
        sprite::ZoomLevel::Zero => 1i32.serialize(serializer),
        sprite::ZoomLevel::One => 2i32.serialize(serializer),
        sprite::ZoomLevel::Two => 3i32.serialize(serializer),
    }
}
