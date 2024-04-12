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
pub enum Direction {
    #[serde(rename = "1")]
    SouthEast,
    #[serde(rename = "4")]
    NorthEast,
    #[serde(rename = "16")]
    NorthWest,
    #[serde(rename = "64")]
    SouthWest,
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

fn serialize_draw_group_zoom_level<S>(
    zoom_level: &ZoomLevel,
    serializer: S,
) -> Result<S::Ok, S::Error>
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
    #[serde(rename = "@dirflags")]
    pub direction: Direction,
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
    pub x: i32,
    #[serde(rename = "@pixely")]
    pub y: i32,
    #[serde(rename = "@xoffset")]
    pub offset_x: f32,
    #[serde(rename = "@yoffset")]
    pub offset_y: f32,
    #[serde(rename = "@zoffset")]
    pub offset_z: f32,
    #[serde(rename = "@flags")]
    pub flags: u32,
}
