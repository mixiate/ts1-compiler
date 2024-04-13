use crate::iff;
use crate::spr;

fn create_palt_chunk(palette_id: iff::ChunkId, sprite_path: &std::path::Path) -> Vec<u8> {
    const PALT_CHUNK_DATA_SIZE: usize = 784;
    const PALT_VERSION: u32 = 1;
    const PALT_COLOUR_ENTRY_COUNT: u32 = 256;

    let palt_chunk_header = iff::ChunkHeader::new("PALT", PALT_CHUNK_DATA_SIZE, palette_id, "");

    let bmp_buffer = std::io::BufReader::new(std::fs::File::open(sprite_path).unwrap());
    let sprite_bmp = image::codecs::bmp::BmpDecoder::new(bmp_buffer).unwrap();
    let palette_bytes = sprite_bmp.get_palette().unwrap().to_vec();
    assert!(palette_bytes.len() == PALT_COLOUR_ENTRY_COUNT as usize);
    let palette_bytes: Vec<_> = palette_bytes.iter().flat_map(|entry| [entry[0], entry[1], entry[2]]).collect();

    let mut palt_chunk = std::vec::Vec::new();
    palt_chunk_header.write(&mut palt_chunk);
    palt_chunk.extend_from_slice(&PALT_VERSION.to_le_bytes());
    palt_chunk.extend_from_slice(&PALT_COLOUR_ENTRY_COUNT.to_le_bytes());
    palt_chunk.extend_from_slice(&0u64.to_le_bytes());
    palt_chunk.extend_from_slice(palette_bytes.as_slice());

    assert!(palt_chunk.len() == iff::IFF_CHUNK_HEADER_SIZE + PALT_CHUNK_DATA_SIZE);

    palt_chunk
}

pub fn create_palt_chunks(
    source_directory: &std::path::Path,
    sprites: &[spr::Sprite],
) -> std::collections::HashMap<iff::ChunkId, Vec<u8>> {
    let mut palette_chunks = std::collections::HashMap::new();

    for sprite in sprites {
        palette_chunks.entry(sprite.palette_chunk_id).or_insert_with(|| {
            let colour_sprite_file_path = source_directory.join(
                sprite
                    .sprite_frames
                    .first()
                    .unwrap()
                    .sprite_channel_file_path_relative(spr::SpriteChannelType::Colour),
            );
            create_palt_chunk(sprite.palette_chunk_id, &colour_sprite_file_path)
        });
    }

    palette_chunks
}
