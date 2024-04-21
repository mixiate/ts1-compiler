use crate::error;
use crate::iff;
use crate::spr;

use anyhow::Context;

pub const PALT_COLOR_ENTRY_COUNT: u16 = 256;

fn create_palt_chunk(palette_id: iff::ChunkId, sprite_path: &std::path::Path) -> anyhow::Result<Vec<u8>> {
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
        palette.len() == usize::from(PALT_COLOR_ENTRY_COUNT),
        format!("{} does not have a 256 color palette", &sprite_path.display())
    );

    let palette: Vec<_> = palette.iter().flat_map(|entry| [entry[0], entry[1], entry[2]]).collect();

    let mut palt_chunk = std::vec::Vec::new();
    palt_chunk_header.write(&mut palt_chunk);
    palt_chunk.extend_from_slice(&PALT_VERSION.to_le_bytes());
    palt_chunk.extend_from_slice(&u32::from(PALT_COLOR_ENTRY_COUNT).to_le_bytes());
    palt_chunk.extend_from_slice(&0u64.to_le_bytes());
    palt_chunk.extend_from_slice(palette.as_slice());

    assert!(palt_chunk.len() == iff::IFF_CHUNK_HEADER_SIZE + PALT_CHUNK_DATA_SIZE);

    Ok(palt_chunk)
}

pub fn create_palt_chunks(source_directory: &std::path::Path, sprites: &[spr::Sprite]) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut palt_chunks = std::collections::HashMap::new();

    for sprite in sprites {
        match palt_chunks.entry(sprite.palette_chunk_id) {
            std::collections::hash_map::Entry::Occupied(_) => (),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let color_sprite_file_path = source_directory.join(
                    sprite
                        .sprite_frames
                        .first()
                        .with_context(|| format!("Failed to find color channel in sprite {}", sprite.chunk_label))?
                        .sprite_channel_file_path_relative(spr::SpriteChannelType::Color),
                );
                entry.insert(create_palt_chunk(sprite.palette_chunk_id, &color_sprite_file_path)?);
            }
        };
    }

    Ok(palt_chunks.into_values().collect())
}
