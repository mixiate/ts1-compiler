use crate::objd;
use crate::palt;
use crate::xml;

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

    pub fn invalid() -> ChunkId {
        ChunkId(-1)
    }
}

#[derive(Clone, Copy)]
pub struct ChunkHeader {
    chunk_type: [u8; 4],
    size: u32,
    id: ChunkId,
    flags: u16,
    label: [u8; IFF_CHUNK_LABEL_SIZE],
}

impl ChunkHeader {
    pub fn new(chunk_type: &str, data_size: usize, id: ChunkId, label: &str) -> ChunkHeader {
        assert!(std::mem::size_of::<ChunkHeader>() == IFF_CHUNK_HEADER_SIZE);

        let label = {
            let mut label_buffer = [0u8; IFF_CHUNK_LABEL_SIZE];
            let label = std::ffi::CString::new(label).unwrap();
            let label = label.to_bytes_with_nul();
            assert!(label.len() < IFF_CHUNK_LABEL_SIZE);
            label_buffer[..label.len()].copy_from_slice(label);
            label_buffer
        };
        ChunkHeader {
            chunk_type: chunk_type.as_bytes().try_into().unwrap(),
            size: u32::try_from(IFF_CHUNK_HEADER_SIZE + data_size).unwrap(),
            id,
            flags: 0x10,
            label,
        }
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

    pub fn write(&self, writer: &mut impl std::io::Write) {
        writer.write_all(&self.chunk_type).unwrap();
        writer.write_all(&self.size.to_be_bytes()).unwrap();
        writer.write_all(&self.id.as_i16().to_be_bytes()).unwrap();
        writer.write_all(&self.flags.to_be_bytes()).unwrap();
        writer.write_all(&self.label).unwrap();
    }
}

pub fn rebuild_iff_file(
    source_directory: &std::path::Path,
    iff_description: &xml::IffDescription,
    input_iff_file_path: &std::path::Path,
    output_iff_file_path: &std::path::Path,
) {
    let input_iff_file_bytes = std::fs::read(input_iff_file_path).unwrap();

    let mut new_chunks = std::vec::Vec::new();

    // create OBJD chunks
    for object_definition in &iff_description.object_definitions.object_definitions {
        let mut objd_chunk = std::vec::Vec::new();
        object_definition.write(&mut objd_chunk);
        assert!(objd_chunk.len() == IFF_CHUNK_HEADER_SIZE + objd::OBJD_CHUNK_DATA_SIZE);
        new_chunks.push(objd_chunk);
    }

    // create SLOT chunks
    for slot in &iff_description.slots.slots {
        let mut slot_chunk = std::vec::Vec::new();
        slot.write(&mut slot_chunk);
        new_chunks.push(slot_chunk);
    }

    // create DGRP chunks
    for draw_group in &iff_description.draw_groups.draw_groups {
        let mut dgrp_chunk = std::vec::Vec::new();
        draw_group.write(&mut dgrp_chunk);
        new_chunks.push(dgrp_chunk);
    }

    // create PALT chunks
    let (palt_transparent_color_indexes, palt_chunks) =
        palt::create_palt_chunks(source_directory, &iff_description.sprites.sprites);
    new_chunks.extend(palt_chunks);

    // create SPR# and SPR2 chunks
    for sprite in &iff_description.sprites.sprites {
        new_chunks.push(sprite.to_chunk_bytes(
            source_directory,
            palt_transparent_color_indexes[&sprite.palette_chunk_id],
        ));
    }

    // create the output iff file, copying the header from the input file
    let mut output_iff_file_bytes = std::vec::Vec::new();
    output_iff_file_bytes.extend_from_slice(&input_iff_file_bytes[0..IFF_FILE_HEADER_SIZE]);

    // copy chunks from input to output iff, ignoring those that will be replaced
    let mut chunk_descs = std::collections::HashMap::new();
    {
        let mut i = output_iff_file_bytes.len();
        while i < input_iff_file_bytes.len() {
            let chunk_header =
                ChunkHeader::from_bytes(&input_iff_file_bytes[i..i + IFF_CHUNK_HEADER_SIZE].try_into().unwrap());
            let chunk_address_offset = u32::try_from(output_iff_file_bytes.len()).unwrap();
            let chunk_type = std::str::from_utf8(&chunk_header.chunk_type).unwrap();
            if !matches!(chunk_type, "DGRP" | "OBJD" | "PALT" | "SLOT" | "SPR#" | "SPR2" | "rsmp") {
                chunk_descs
                    .entry(chunk_header.chunk_type)
                    .or_insert_with(std::vec::Vec::new)
                    .push((chunk_header, chunk_address_offset));
                output_iff_file_bytes
                    .extend_from_slice(&input_iff_file_bytes[i..i + usize::try_from(chunk_header.size).unwrap()]);
            }
            i += usize::try_from(chunk_header.size).unwrap();
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

            let rsmp_chunk_header = ChunkHeader::new("rsmp", rsmp_data.len(), ChunkId(0), "");
            rsmp_chunk_header.write(&mut rsmp_chunk);

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

    std::fs::write(output_iff_file_path, &output_iff_file_bytes).unwrap();
}
