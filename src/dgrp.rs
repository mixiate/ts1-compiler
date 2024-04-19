use crate::iff;
use crate::spr;

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

fn deserialize_draw_group_rotation<'de, D>(deserializer: D) -> Result<Rotation, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let string = String::deserialize(deserializer)?;

    const FIELDS: &[&str] = &["1", "4", "16", "64"];
    match string.as_str() {
        "1" => Ok(Rotation::SouthEast),
        "4" => Ok(Rotation::NorthEast),
        "16" => Ok(Rotation::NorthWest),
        "64" => Ok(Rotation::SouthWest),
        _ => Err(serde::de::Error::unknown_field(&string, FIELDS)),
    }
}

fn serialize_draw_group_rotation<S>(rotation: &Rotation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    match rotation {
        Rotation::SouthEast => 1i32.serialize(serializer),
        Rotation::NorthEast => 4i32.serialize(serializer),
        Rotation::NorthWest => 16i32.serialize(serializer),
        Rotation::SouthWest => 64i32.serialize(serializer),
    }
}

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

fn deserialize_draw_group_zoom_level<'de, D>(deserializer: D) -> Result<ZoomLevel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let string = String::deserialize(deserializer)?;

    const FIELDS: &[&str] = &["1", "2", "3"];
    match string.as_str() {
        "1" => Ok(ZoomLevel::Zero),
        "2" => Ok(ZoomLevel::One),
        "3" => Ok(ZoomLevel::Two),
        _ => Err(serde::de::Error::unknown_field(&string, FIELDS)),
    }
}

fn serialize_draw_group_zoom_level<S>(zoom_level: &ZoomLevel, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    match zoom_level {
        ZoomLevel::Zero => 1i32.serialize(serializer),
        ZoomLevel::One => 2i32.serialize(serializer),
        ZoomLevel::Two => 3i32.serialize(serializer),
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawGroupItemList {
    #[serde(
        deserialize_with = "deserialize_draw_group_rotation",
        serialize_with = "serialize_draw_group_rotation",
        rename = "@dirflags"
    )]
    pub rotation: Rotation,
    #[serde(
        deserialize_with = "deserialize_draw_group_zoom_level",
        serialize_with = "serialize_draw_group_zoom_level",
        rename = "@zoom"
    )]
    pub zoom_level: ZoomLevel,
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
    pub fn write(&self, writer: &mut impl std::io::Write) {
        const DGRP_HEADER_VERSION: u16 = 20004u16;
        const DGRP_HEADER_IMAGE_COUNT: u32 = 12;

        let mut dgrp_data = std::vec::Vec::new();

        dgrp_data.extend_from_slice(&DGRP_HEADER_VERSION.to_le_bytes());
        dgrp_data.extend_from_slice(&DGRP_HEADER_IMAGE_COUNT.to_le_bytes());

        for draw_group_item_list in &self.draw_group_item_lists {
            let rotation = match draw_group_item_list.rotation {
                Rotation::NorthWest => 16u32,
                Rotation::NorthEast => 4u32,
                Rotation::SouthEast => 1u32,
                Rotation::SouthWest => 64u32,
            };
            let zoom_level = match draw_group_item_list.zoom_level {
                ZoomLevel::Zero => 1u32,
                ZoomLevel::One => 2u32,
                ZoomLevel::Two => 3u32,
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

        let dgrp_chunk_header = iff::ChunkHeader::new("DGRP", dgrp_data.len(), self.chunk_id, &self.chunk_label);
        dgrp_chunk_header.write(writer);

        writer.write_all(&dgrp_data).unwrap();
    }
}

/// Returns the original and transmogrified rotation strings from a rotation
pub fn rotation_names(rotation: Rotation) -> (&'static str, &'static str) {
    match rotation {
        Rotation::NorthWest => ("nw", "se"),
        Rotation::NorthEast => ("ne", "ne"),
        Rotation::SouthEast => ("se", "nw"),
        Rotation::SouthWest => ("sw", "sw"),
    }
}
