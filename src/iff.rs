use crate::error;
use crate::iff_description;
use crate::palt;
use crate::spr;

use anyhow::Context;

pub const IFF_HEADER_SIZE: usize = 64;

pub const IFF_CHUNK_HEADER_SIZE: usize = 76;
pub const IFF_CHUNK_LABEL_SIZE: usize = 64;

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, binrw::BinRead, binrw::BinWrite, serde::Deserialize, serde::Serialize,
)]
pub struct IffChunkId(i16);

impl IffChunkId {
    pub fn as_i16(self) -> i16 {
        self.0
    }

    pub fn as_i32(self) -> i32 {
        i32::from(self.0)
    }

    pub fn advance(&mut self) {
        self.0 += 1;
    }
}

impl std::ops::Add<i16> for IffChunkId {
    type Output = IffChunkId;

    fn add(self, other: i16) -> IffChunkId {
        IffChunkId(self.0 + other)
    }
}

#[derive(Clone, Debug, binrw::BinRead, binrw::BinWrite)]
#[brw(big)]
#[brw(assert(label.iter().any(|x|*x == b'\0')))]
pub struct IffChunkHeader {
    chunk_type: [u8; 4],
    size: u32,
    id: IffChunkId,
    flags: i16,
    label: [u8; IFF_CHUNK_LABEL_SIZE],
}

impl IffChunkHeader {
    pub fn new(chunk_type: &[u8; 4], data_size: usize, id: IffChunkId, label: &str) -> anyhow::Result<IffChunkHeader> {
        let label = {
            let mut label_buffer = [0u8; IFF_CHUNK_LABEL_SIZE];
            let cstring_label = std::ffi::CString::new(label).unwrap();
            let cstring_label = cstring_label.to_bytes_with_nul();
            anyhow::ensure!(
                cstring_label.len() < IFF_CHUNK_LABEL_SIZE,
                format!(
                    "Chunk label \"{}\" is larger than {} bytes",
                    label, IFF_CHUNK_LABEL_SIZE
                )
            );
            label_buffer[..cstring_label.len()].copy_from_slice(cstring_label);
            label_buffer
        };
        Ok(IffChunkHeader {
            chunk_type: *chunk_type,
            size: u32::try_from(IFF_CHUNK_HEADER_SIZE + data_size).unwrap(),
            id,
            flags: 0x10,
            label,
        })
    }
}

#[derive(Clone, binrw::BinRead, binrw::BinWrite)]
pub struct IffChunk {
    pub header: IffChunkHeader,
    #[br(count = header.size - (IFF_CHUNK_HEADER_SIZE as u32))]
    pub data: Vec<u8>,
}

#[derive(binrw::BinRead, binrw::BinWrite)]
#[brw(magic = b"IFF FILE 2.5:TYPE FOLLOWED BY SIZE\0 JAMIE DOORNBOS & MAXIS 1")]
struct Iff {
    #[brw(big)]
    rsmp_address: u32,
    #[br(parse_with = binrw::helpers::until_eof)]
    chunks: Vec<IffChunk>,
}

fn read_iff_file(iff_file_path: &std::path::Path) -> anyhow::Result<Iff> {
    let mut iff_file = std::fs::File::open(iff_file_path).with_context(|| error::file_read_error(iff_file_path))?;
    use binrw::BinReaderExt;
    let iff: Iff = iff_file.read_ne().with_context(|| iff_decode_error(iff_file_path))?;

    let chunk_sizes = iff.chunks.iter().fold(0u32, |acc, x| acc + x.header.size);
    use std::io::Seek;
    assert!(IFF_HEADER_SIZE as u64 + u64::from(chunk_sizes) == iff_file.seek(std::io::SeekFrom::End(0)).unwrap());

    Ok(iff)
}

fn map_guids(chunks: &[IffChunk]) -> std::collections::HashMap<IffChunkId, i32> {
    let mut guids = std::collections::HashMap::new();
    for chunk in chunks {
        if &chunk.header.chunk_type == b"OBJD" {
            const GUID_ADDRESS_OFFSET: usize = 28;
            let guid = i32::from_le_bytes(
                chunk.data.get(GUID_ADDRESS_OFFSET..GUID_ADDRESS_OFFSET + 4).unwrap().try_into().unwrap(),
            );
            guids.entry(chunk.header.id).or_insert(guid);
        }
    }
    guids
}

fn create_rsmp_chunk(chunks: &[IffChunk]) -> IffChunk {
    let mut chunk_descriptions = std::collections::HashMap::new();
    chunks.iter().fold(IFF_HEADER_SIZE as u32, |address, chunk| {
        chunk_descriptions
            .entry(chunk.header.chunk_type)
            .or_insert_with(Vec::new)
            .push((chunk.header.clone(), address));
        address + chunk.header.size
    });

    let mut rsmp_data = std::vec::Vec::new();
    for (chunk_type, chunks) in &chunk_descriptions {
        let chunk_type = {
            let mut chunk_type: [u8; 4] = *chunk_type;
            chunk_type.reverse();
            chunk_type
        };
        rsmp_data.extend_from_slice(&chunk_type);
        rsmp_data.extend_from_slice(&u32::try_from(chunks.len()).unwrap().to_le_bytes());
        for chunk in chunks {
            let label_length = chunk.0.label.iter().position(|x| *x == 0).unwrap() + 1;
            let label_length = label_length + (label_length % 2);

            rsmp_data.extend_from_slice(&chunk.1.to_le_bytes());
            rsmp_data.extend_from_slice(&chunk.0.id.as_i16().to_le_bytes());
            rsmp_data.extend_from_slice(&chunk.0.flags.to_le_bytes());
            rsmp_data.extend_from_slice(&chunk.0.label[0..label_length]);
        }
    }

    const RSMP_HEADER_SIZE: usize = 20;
    let rsmp_size = IFF_CHUNK_HEADER_SIZE + RSMP_HEADER_SIZE + rsmp_data.len();

    let mut rsmp_chunk = std::vec::Vec::new();

    rsmp_chunk.extend_from_slice(&0u32.to_le_bytes());
    rsmp_chunk.extend_from_slice(&0u32.to_le_bytes());
    rsmp_chunk.extend_from_slice(b"pmsr");
    rsmp_chunk.extend_from_slice(&u32::try_from(rsmp_size).unwrap().to_le_bytes());
    rsmp_chunk.extend_from_slice(&u32::try_from(chunk_descriptions.len()).unwrap().to_le_bytes());

    rsmp_chunk.extend_from_slice(rsmp_data.as_slice());

    let rsmp_chunk_header = IffChunkHeader::new(b"rsmp", rsmp_chunk.len(), IffChunkId(0), "").unwrap();

    IffChunk {
        header: rsmp_chunk_header,
        data: rsmp_chunk,
    }
}

fn replace_guids_in_bhavs(
    chunks: &mut [IffChunk],
    input_guids: &std::collections::HashMap<IffChunkId, i32>,
    output_guids: &std::collections::HashMap<IffChunkId, i32>,
) {
    for chunk in chunks {
        if &chunk.header.chunk_type == b"BHAV" {
            const INSTRUCTION_SIZE: usize = 12;
            const PARAMETER_OFFSET: usize = 4;
            assert!(chunk.data.len() % INSTRUCTION_SIZE == 0);
            let instruction_count = chunk.data.len() / INSTRUCTION_SIZE;
            for j in 0..instruction_count {
                let instruction_address = j * INSTRUCTION_SIZE;
                let guid_address = instruction_address + PARAMETER_OFFSET;
                if matches!(chunk.data.get(instruction_address).unwrap(), 31 | 32 | 42) {
                    let guid =
                        i32::from_le_bytes(chunk.data.get(guid_address..guid_address + 4).unwrap().try_into().unwrap());
                    if let Some((objd_id, _)) = input_guids.iter().find(|(_, input_guid)| **input_guid == guid) {
                        let output_guid = output_guids.get(objd_id).unwrap();
                        chunk
                            .data
                            .get_mut(guid_address..guid_address + 4)
                            .unwrap()
                            .copy_from_slice(&output_guid.to_le_bytes());
                    }
                }
            }
        }
    }
}

pub fn rebuild_iff_file(
    source_directory: &std::path::Path,
    iff_description: &iff_description::IffDescription,
    input_iff_file_path: &std::path::Path,
    output_iff_file_path: &std::path::Path,
) -> anyhow::Result<()> {
    let mut iff = read_iff_file(input_iff_file_path)?;

    let (input_guids, output_guids) = {
        let output_iff = read_iff_file(output_iff_file_path)?;
        (map_guids(&iff.chunks), map_guids(&output_iff.chunks))
    };
    anyhow::ensure!(
        !input_guids.is_empty(),
        format!("Failed to find any GUIDs in {}", input_iff_file_path.display())
    );
    anyhow::ensure!(
        !output_guids.is_empty(),
        format!("Failed to find any GUIDs in {}", output_iff_file_path.display())
    );
    anyhow::ensure!(
        input_guids.len() == output_guids.len() && input_guids.keys().all(|k| output_guids.contains_key(k)),
        format!(
            "GUIDs in {} do not match GUIDs in {}",
            input_iff_file_path.display(),
            output_iff_file_path.display()
        )
    );
    if input_iff_file_path != output_iff_file_path {
        anyhow::ensure!(
            input_guids != output_guids,
            "GUIDs in iff files match. Variant objects must have unique GUIDs"
        );
    }

    iff.chunks.retain(|x| {
        !matches!(
            &x.header.chunk_type,
            b"DGRP" | b"OBJD" | b"PALT" | b"SLOT" | b"SPR#" | b"SPR2" | b"rsmp"
        )
    });

    for object_definition in &iff_description.object_definitions.object_definitions {
        let replacement_guid = *output_guids.get(&object_definition.chunk_id).with_context(|| {
            format!(
                "Failed to find replacement GUID for object {} {}",
                object_definition.chunk_id.as_i32(),
                object_definition.chunk_label
            )
        })?;
        iff.chunks.push(object_definition.to_chunk(Some(replacement_guid))?);
    }

    for slot in &iff_description.slots.slots {
        iff.chunks.push(slot.to_chunk()?);
    }

    for draw_group in &iff_description.draw_groups.draw_groups {
        iff.chunks.push(draw_group.to_chunk()?);
    }

    let used_sprite_ids = {
        let mut used_sprite_ids = std::collections::HashSet::new();
        for draw_group in &iff_description.draw_groups.draw_groups {
            for draw_group_item_list in &draw_group.draw_group_item_lists {
                for draw_group_item in &draw_group_item_list.draw_group_items {
                    used_sprite_ids.insert(draw_group_item.sprite_chunk_id);
                }
            }
        }
        used_sprite_ids
    };

    let palt_chunks = palt::create_palt_chunks(source_directory, &iff_description.sprites.sprites)?;
    iff.chunks.extend(palt_chunks);

    for sprite in &iff_description.sprites.sprites {
        if sprite.sprite_type == spr::SpriteType::Spr2 && !used_sprite_ids.contains(&sprite.chunk_id) {
            continue;
        }
        iff.chunks.push(sprite.to_chunk(source_directory)?);
    }

    iff.chunks.push(create_rsmp_chunk(&iff.chunks));

    iff.rsmp_address = iff
        .chunks
        .iter()
        .try_fold(IFF_HEADER_SIZE as u32, |acc, x| {
            if &x.header.chunk_type != b"rsmp" {
                Ok(acc + x.header.size)
            } else {
                Err(acc)
            }
        })
        .unwrap_err();

    replace_guids_in_bhavs(&mut iff.chunks, &input_guids, &output_guids);

    let mut output_iff_file =
        std::fs::File::create(output_iff_file_path).with_context(|| error::file_write_error(output_iff_file_path))?;
    use binrw::BinWriterExt;
    output_iff_file.write_ne(&iff).with_context(|| error::file_write_error(output_iff_file_path))?;

    Ok(())
}

fn iff_decode_error(file_path: &std::path::Path) -> String {
    format!("Failed to decode iff file {}", file_path.display())
}
