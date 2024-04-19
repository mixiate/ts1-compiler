use crate::error;
use crate::iff;
use crate::spr;

use anyhow::Context;

pub const PALT_COLOR_ENTRY_COUNT: u32 = 256;

fn create_palt_chunk(palette_id: iff::ChunkId, sprite_path: &std::path::Path) -> anyhow::Result<(u8, Vec<u8>)> {
    const PALT_CHUNK_DATA_SIZE: usize = 784;
    const PALT_VERSION: u32 = 1;

    let palt_chunk_header = iff::ChunkHeader::new("PALT", PALT_CHUNK_DATA_SIZE, palette_id, "");

    let bmp_buffer = std::fs::File::open(sprite_path).with_context(|| error::file_read_error(sprite_path))?;
    let bmp_buffer = std::io::BufReader::new(&bmp_buffer);
    let sprite_bmp =
        image::codecs::bmp::BmpDecoder::new(bmp_buffer).with_context(|| error::file_read_error(sprite_path))?;

    let palette = sprite_bmp
        .get_palette()
        .with_context(|| format!("{} is not in 8-bit color", sprite_path.display()))?
        .to_vec();
    anyhow::ensure!(
        palette.len() == PALT_COLOR_ENTRY_COUNT as usize,
        format!("{} does not have a 256 color palette", &sprite_path.display())
    );

    const TRANSPARENT_COLOR: [u8; 3] = [255, 255, 0];
    let transparent_color_index = u8::try_from(
        palette
            .iter()
            .position(|entry| *entry == TRANSPARENT_COLOR)
            .with_context(|| format!("Failed to find transparent color in {}", sprite_path.display()))?,
    )
    .unwrap();

    let palette: Vec<_> = palette.iter().flat_map(|entry| [entry[0], entry[1], entry[2]]).collect();

    let mut palt_chunk = std::vec::Vec::new();
    palt_chunk_header.write(&mut palt_chunk);
    palt_chunk.extend_from_slice(&PALT_VERSION.to_le_bytes());
    palt_chunk.extend_from_slice(&PALT_COLOR_ENTRY_COUNT.to_le_bytes());
    palt_chunk.extend_from_slice(&0u64.to_le_bytes());
    palt_chunk.extend_from_slice(palette.as_slice());

    assert!(palt_chunk.len() == iff::IFF_CHUNK_HEADER_SIZE + PALT_CHUNK_DATA_SIZE);

    Ok((transparent_color_index, palt_chunk))
}

pub struct PaltChunks {
    pub transparent_color_indexes: std::collections::HashMap<iff::ChunkId, u8>,
    pub chunks: Vec<Vec<u8>>,
}

pub fn create_palt_chunks(source_directory: &std::path::Path, sprites: &[spr::Sprite]) -> anyhow::Result<PaltChunks> {
    let mut transparent_color_indexes = std::collections::HashMap::new();
    let mut palt_chunks = Vec::new();

    const INVALID_TRANSPARENT_COLOR_INDEX: u8 = 255;
    transparent_color_indexes
        .entry(iff::ChunkId::invalid())
        .or_insert_with(|| INVALID_TRANSPARENT_COLOR_INDEX);

    for sprite in sprites {
        match transparent_color_indexes.entry(sprite.palette_chunk_id) {
            std::collections::hash_map::Entry::Occupied(_) => (),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let color_sprite_file_path = source_directory.join(
                    sprite
                        .sprite_frames
                        .first()
                        .with_context(|| format!("Failed to find color channel in sprite {}", sprite.chunk_label))?
                        .sprite_channel_file_path_relative(spr::SpriteChannelType::Color),
                );
                let (transparent_color_index, palt_chunk) =
                    create_palt_chunk(sprite.palette_chunk_id, &color_sprite_file_path)?;
                palt_chunks.push(palt_chunk);
                entry.insert(transparent_color_index);
            }
        };
    }

    Ok(PaltChunks {
        transparent_color_indexes,
        chunks: palt_chunks,
    })
}
