use crate::error;
use crate::iff_description;
use crate::palt;

use anyhow::Context;

pub const IFF_FILE_HEADER_SIZE: usize = 64;
pub const IFF_CHUNK_HEADER_SIZE: usize = 76;
pub const IFF_CHUNK_LABEL_SIZE: usize = 64;

#[derive(Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize, serde::Serialize)]
pub struct ChunkId(i16);

impl ChunkId {
    pub fn as_i16(self) -> i16 {
        self.0
    }

    pub fn as_i32(self) -> i32 {
        i32::from(self.0)
    }

    pub fn from_be_bytes(bytes: [u8; 2]) -> ChunkId {
        ChunkId(i16::from_be_bytes(bytes))
    }
}

#[derive(Clone)]
pub struct ChunkHeader {
    chunk_type: [u8; 4],
    size: u32,
    id: ChunkId,
    flags: u16,
    label: [u8; IFF_CHUNK_LABEL_SIZE],
}

impl ChunkHeader {
    pub fn new(chunk_type: &str, data_size: usize, id: ChunkId, label: &str) -> anyhow::Result<ChunkHeader> {
        assert!(std::mem::size_of::<ChunkHeader>() == IFF_CHUNK_HEADER_SIZE);

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
        Ok(ChunkHeader {
            chunk_type: chunk_type.as_bytes().try_into().unwrap(),
            size: u32::try_from(IFF_CHUNK_HEADER_SIZE + data_size).unwrap(),
            id,
            flags: 0x10,
            label,
        })
    }

    pub fn from_bytes(chunk_bytes: &[u8; IFF_CHUNK_HEADER_SIZE]) -> ChunkHeader {
        ChunkHeader {
            chunk_type: chunk_bytes[0..4].try_into().unwrap(),
            size: u32::from_be_bytes([chunk_bytes[4], chunk_bytes[5], chunk_bytes[6], chunk_bytes[7]]),
            id: ChunkId::from_be_bytes([chunk_bytes[8], chunk_bytes[9]]),
            flags: u16::from_be_bytes([chunk_bytes[10], chunk_bytes[11]]),
            label: chunk_bytes[12..12 + IFF_CHUNK_LABEL_SIZE].try_into().unwrap(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(IFF_CHUNK_HEADER_SIZE);
        bytes.extend_from_slice(&self.chunk_type);
        bytes.extend_from_slice(&self.size.to_be_bytes());
        bytes.extend_from_slice(&self.id.as_i16().to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_be_bytes());
        bytes.extend_from_slice(&self.label);
        bytes
    }
}

fn get_guids_from_iff_file_bytes(iff_file_bytes: &[u8]) -> anyhow::Result<std::collections::HashMap<ChunkId, i32>> {
    let mut guids = std::collections::HashMap::new();
    let mut i = IFF_FILE_HEADER_SIZE;
    while i < iff_file_bytes.len() {
        let chunk_header = ChunkHeader::from_bytes(&iff_file_bytes[i..i + IFF_CHUNK_HEADER_SIZE].try_into()?);
        let chunk_type = std::str::from_utf8(&chunk_header.chunk_type)?;
        if chunk_type == "OBJD" {
            const GUID_ADDRESS_OFFSET: usize = 28;
            let guid_address = i + IFF_CHUNK_HEADER_SIZE + GUID_ADDRESS_OFFSET;
            let guid = i32::from_le_bytes(iff_file_bytes[guid_address..guid_address + 4].try_into()?);
            guids.entry(chunk_header.id).or_insert(guid);
        }
        i += usize::try_from(chunk_header.size)?;
    }
    anyhow::ensure!(!guids.is_empty(), "Failed to find any GUIDs");
    Ok(guids)
}

pub fn rebuild_iff_file(
    source_directory: &std::path::Path,
    iff_description: &iff_description::IffDescription,
    input_iff_file_path: &std::path::Path,
    output_iff_file_path: &std::path::Path,
) -> anyhow::Result<()> {
    let input_iff_file_bytes =
        std::fs::read(input_iff_file_path).with_context(|| error::file_read_error(input_iff_file_path))?;

    let (input_guids, output_guids) = {
        let output_iff_file_bytes =
            std::fs::read(output_iff_file_path).with_context(|| error::file_read_error(output_iff_file_path))?;
        (
            get_guids_from_iff_file_bytes(&input_iff_file_bytes)
                .with_context(|| iff_decode_error(input_iff_file_path))?,
            get_guids_from_iff_file_bytes(&output_iff_file_bytes)
                .with_context(|| iff_decode_error(output_iff_file_path))?,
        )
    };

    let mut new_chunks = std::vec::Vec::new();

    // create OBJD chunks
    for object_definition in &iff_description.object_definitions.object_definitions {
        let replacement_guid = *output_guids.get(&object_definition.chunk_id).with_context(|| {
            format!(
                "Failed to find replacement GUID for object {} {}",
                object_definition.chunk_id.as_i32(),
                object_definition.chunk_label
            )
        })?;
        new_chunks.push(object_definition.to_bytes(Some(replacement_guid))?);
    }

    // create SLOT chunks
    for slot in &iff_description.slots.slots {
        new_chunks.push(slot.to_bytes()?);
    }

    // create DGRP chunks
    for draw_group in &iff_description.draw_groups.draw_groups {
        new_chunks.push(draw_group.to_bytes()?);
    }

    // create PALT chunks
    let palt_chunks = palt::create_palt_chunks(source_directory, &iff_description.sprites.sprites)?;
    new_chunks.extend(palt_chunks);

    // create SPR# and SPR2 chunks
    for sprite in &iff_description.sprites.sprites {
        new_chunks.push(sprite.to_bytes(source_directory)?);
    }

    // create the output iff file, copying the header from the input file
    let mut output_iff_file_bytes = std::vec::Vec::new();
    output_iff_file_bytes.extend_from_slice(&input_iff_file_bytes[0..IFF_FILE_HEADER_SIZE]);

    // copy chunks from input to output iff, ignoring those that will be replaced
    let mut chunk_descs = std::collections::HashMap::new();
    {
        let mut i = output_iff_file_bytes.len();
        while i < input_iff_file_bytes.len() {
            let chunk_header = ChunkHeader::from_bytes(
                &input_iff_file_bytes[i..i + IFF_CHUNK_HEADER_SIZE]
                    .try_into()
                    .with_context(|| iff_decode_error(input_iff_file_path))?,
            );
            let chunk_address_offset =
                u32::try_from(output_iff_file_bytes.len()).with_context(|| iff_decode_error(input_iff_file_path))?;
            let chunk_type =
                std::str::from_utf8(&chunk_header.chunk_type).with_context(|| iff_decode_error(input_iff_file_path))?;
            let chunk_header_size = chunk_header.size;
            if !matches!(chunk_type, "DGRP" | "OBJD" | "PALT" | "SLOT" | "SPR#" | "SPR2" | "rsmp") {
                chunk_descs
                    .entry(chunk_header.chunk_type)
                    .or_insert_with(std::vec::Vec::new)
                    .push((chunk_header, chunk_address_offset));
                output_iff_file_bytes.extend_from_slice(
                    &input_iff_file_bytes[i..i + usize::try_from(chunk_header_size)
                        .with_context(|| iff_decode_error(input_iff_file_path))?],
                );
            }
            i += usize::try_from(chunk_header_size).with_context(|| iff_decode_error(input_iff_file_path))?;
        }
    }

    // add our replacement chunks to the output iff
    for new_chunk in new_chunks {
        let chunk_header = ChunkHeader::from_bytes(&new_chunk[0..IFF_CHUNK_HEADER_SIZE].try_into().unwrap());
        let chunk_address_offset = u32::try_from(output_iff_file_bytes.len()).unwrap();
        chunk_descs
            .entry(chunk_header.chunk_type)
            .or_insert_with(std::vec::Vec::new)
            .push((chunk_header, chunk_address_offset));

        output_iff_file_bytes.extend_from_slice(new_chunk.as_slice());
    }

    // create the rsmp chunk for the output iff
    {
        let rsmp_chunk = {
            let mut rsmp_data = std::vec::Vec::new();
            rsmp_data.extend_from_slice(&0u32.to_le_bytes()); //reserved
            rsmp_data.extend_from_slice(&0u32.to_le_bytes()); //version
            rsmp_data.extend_from_slice("pmsr".as_bytes()); //magic string
            rsmp_data.extend_from_slice(&0u32.to_le_bytes()); //size
            rsmp_data.extend_from_slice(&u32::try_from(chunk_descs.len()).unwrap().to_le_bytes()); //chunk type count

            for (chunk_type, chunks) in &chunk_descs {
                let chunk_type = {
                    let mut chunk_type = *chunk_type;
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

            let mut rsmp_chunk = std::vec::Vec::new();

            let rsmp_chunk_header = ChunkHeader::new("rsmp", rsmp_data.len(), ChunkId(0), "").unwrap();
            rsmp_chunk.extend_from_slice(&rsmp_chunk_header.to_bytes());

            rsmp_chunk.extend_from_slice(rsmp_data.as_slice());

            let rsmp_chunk_size_address = IFF_CHUNK_HEADER_SIZE + 4 + 4 + 4;
            let rsmp_chunk_size = u32::try_from(rsmp_chunk.len()).unwrap();
            rsmp_chunk[rsmp_chunk_size_address..rsmp_chunk_size_address + 4]
                .copy_from_slice(&rsmp_chunk_size.to_le_bytes());

            rsmp_chunk
        };

        let rsmp_address = u32::try_from(output_iff_file_bytes.len()).unwrap();
        output_iff_file_bytes.extend_from_slice(rsmp_chunk.as_slice());
        output_iff_file_bytes[60..64].copy_from_slice(&rsmp_address.to_be_bytes());
    }

    // replace the guids in the BHAV code of the output iff
    {
        let mut i = IFF_FILE_HEADER_SIZE;
        while i < output_iff_file_bytes.len() {
            let chunk_header =
                ChunkHeader::from_bytes(&output_iff_file_bytes[i..i + IFF_CHUNK_HEADER_SIZE].try_into().unwrap());
            let chunk_size = usize::try_from(chunk_header.size).unwrap();
            let chunk_type = std::str::from_utf8(&chunk_header.chunk_type).unwrap();
            if chunk_type == "BHAV" {
                const INSTRUCTION_SIZE: usize = 12;
                const PARAMETER_OFFSET: usize = 4;
                assert!((chunk_size - IFF_CHUNK_HEADER_SIZE) % INSTRUCTION_SIZE == 0);
                let instruction_count = (chunk_size - IFF_CHUNK_HEADER_SIZE) / INSTRUCTION_SIZE;
                for j in 0..instruction_count {
                    let instruction_address = i + IFF_CHUNK_HEADER_SIZE + (j * INSTRUCTION_SIZE);
                    let guid_address = instruction_address + PARAMETER_OFFSET;
                    if matches!(output_iff_file_bytes[instruction_address], 31 | 32 | 42) {
                        let guid = i32::from_le_bytes(
                            output_iff_file_bytes[guid_address..guid_address + 4].try_into().unwrap(),
                        );
                        if let Some((objd_id, _)) = input_guids.iter().find(|(_, main_guid)| **main_guid == guid) {
                            let output_guid = output_guids.get(objd_id).unwrap();
                            output_iff_file_bytes[guid_address..guid_address + 4]
                                .copy_from_slice(&output_guid.to_le_bytes());
                        }
                    }
                }
            }
            i += chunk_size;
        }
    }

    std::fs::write(output_iff_file_path, &output_iff_file_bytes)
        .with_context(|| error::file_write_error(output_iff_file_path))?;

    Ok(())
}

fn iff_decode_error(file_path: &std::path::Path) -> String {
    format!("Failed to decode iff file {}", file_path.display())
}
