use crate::iff;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    #[serde(rename = "@name")]
    pub chunk_label: String,
    #[serde(rename = "@id")]
    pub chunk_id: iff::ChunkId,
    #[serde(rename = "slotdescriptor")]
    pub slot_descriptors: Vec<SlotDescriptor>,
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum SlotType {
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
    #[serde(rename = "3")]
    Three,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SlotDescriptor {
    #[serde(rename = "@type")]
    pub r#type: SlotType,
    #[serde(rename = "@xoffset")]
    pub xoffset: f32,
    #[serde(rename = "@yoffset")]
    pub yoffset: f32,
    #[serde(rename = "@altoffset")]
    pub altoffset: f32,
    #[serde(rename = "@standing", skip_serializing_if = "Option::is_none")]
    pub standing: Option<i32>,
    #[serde(rename = "@sitting", skip_serializing_if = "Option::is_none")]
    pub sitting: Option<i32>,
    #[serde(rename = "@ground", skip_serializing_if = "Option::is_none")]
    pub ground: Option<i32>,
    #[serde(rename = "@rsflags", skip_serializing_if = "Option::is_none")]
    pub rsflags: Option<i32>,
    #[serde(rename = "@snaptargetslot", skip_serializing_if = "Option::is_none")]
    pub snaptargetslot: Option<i32>,
    #[serde(rename = "@minproximity", skip_serializing_if = "Option::is_none")]
    pub minproximity: Option<i32>,
    #[serde(rename = "@maxproximity", skip_serializing_if = "Option::is_none")]
    pub maxproximity: Option<i32>,
    #[serde(rename = "@optimalproximity", skip_serializing_if = "Option::is_none")]
    pub optimalproximity: Option<i32>,
    #[serde(rename = "@maxsize", skip_serializing_if = "Option::is_none")]
    pub maxsize: Option<i32>,
    #[serde(rename = "@flags", skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
    #[serde(rename = "@gradient", skip_serializing_if = "Option::is_none")]
    pub gradient: Option<f32>,
    #[serde(rename = "@height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(rename = "@facing", skip_serializing_if = "Option::is_none")]
    pub facing: Option<i32>,
    #[serde(rename = "@resolution", skip_serializing_if = "Option::is_none")]
    pub resolution: Option<i32>,
}
