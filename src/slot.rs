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
    pub slot_type: SlotType,
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

impl Slot {
    pub fn write(&self, writer: &mut impl std::io::Write) {
        let mut slot_data = std::vec::Vec::new();

        const SLOT_HEADER_VERSION: u32 = 10;
        const SLOT_HEADER_SIZE: usize = 16;
        const SLOT_DESCRIPTOR_SIZE: usize = 70;

        slot_data.extend_from_slice(&0u32.to_le_bytes());
        slot_data.extend_from_slice(&SLOT_HEADER_VERSION.to_le_bytes());
        slot_data.extend_from_slice("TOLS".as_bytes());
        slot_data.extend_from_slice(&u32::try_from(self.slot_descriptors.len()).unwrap().to_le_bytes());

        for slot_descriptor in &self.slot_descriptors {
            let slot_type = match slot_descriptor.slot_type {
                SlotType::Zero => 0i16,
                SlotType::One => 1i16,
                SlotType::Three => 3i16,
            };
            slot_data.extend_from_slice(&slot_type.to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.xoffset.to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.yoffset.to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.altoffset.to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.standing.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.sitting.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.ground.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.rsflags.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.snaptargetslot.unwrap_or(-1).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.minproximity.unwrap_or(16).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.maxproximity.unwrap_or(16).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.optimalproximity.unwrap_or(16).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.maxsize.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.flags.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.gradient.unwrap_or(0.1875).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.height.unwrap_or(0).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.facing.unwrap_or(-2).to_le_bytes());
            slot_data.extend_from_slice(&slot_descriptor.resolution.unwrap_or(16).to_le_bytes());
        }

        assert!(slot_data.len() == SLOT_HEADER_SIZE + (self.slot_descriptors.len() * SLOT_DESCRIPTOR_SIZE));

        let slot_chunk_header = iff::ChunkHeader::new("SLOT", slot_data.len(), self.chunk_id, &self.chunk_label);
        slot_chunk_header.write(writer);

        writer.write_all(&slot_data).unwrap();
    }
}
