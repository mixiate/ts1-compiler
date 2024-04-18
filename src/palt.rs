use crate::iff;
use crate::spr;

pub const PALT_COLOR_ENTRY_COUNT: u32 = 256;

fn create_palt_chunk(palette_id: iff::ChunkId, sprite_path: &std::path::Path) -> (u8, Vec<u8>) {
    const PALT_CHUNK_DATA_SIZE: usize = 784;
    const PALT_VERSION: u32 = 1;

    let palt_chunk_header = iff::ChunkHeader::new("PALT", PALT_CHUNK_DATA_SIZE, palette_id, "");

    let bmp_buffer = std::io::BufReader::new(std::fs::File::open(sprite_path).unwrap());
    let sprite_bmp = image::codecs::bmp::BmpDecoder::new(bmp_buffer).unwrap();
    let palette = sprite_bmp.get_palette().unwrap().to_vec();
    assert!(palette.len() == PALT_COLOR_ENTRY_COUNT as usize);
    const TRANSPARENT_COLOR: [u8; 3] = [255, 255, 0];
    let transparent_color_index =
        u8::try_from(palette.iter().position(|entry| *entry == TRANSPARENT_COLOR).unwrap()).unwrap();
    let palette: Vec<_> = palette.iter().flat_map(|entry| [entry[0], entry[1], entry[2]]).collect();

    let mut palt_chunk = std::vec::Vec::new();
    palt_chunk_header.write(&mut palt_chunk);
    palt_chunk.extend_from_slice(&PALT_VERSION.to_le_bytes());
    palt_chunk.extend_from_slice(&PALT_COLOR_ENTRY_COUNT.to_le_bytes());
    palt_chunk.extend_from_slice(&0u64.to_le_bytes());
    palt_chunk.extend_from_slice(palette.as_slice());

    assert!(palt_chunk.len() == iff::IFF_CHUNK_HEADER_SIZE + PALT_CHUNK_DATA_SIZE);

    (transparent_color_index, palt_chunk)
}

pub fn create_palt_chunks(
    source_directory: &std::path::Path,
    sprites: &[spr::Sprite],
) -> (std::collections::HashMap<iff::ChunkId, u8>, Vec<Vec<u8>>) {
    let mut palt_transparent_color_indexes = std::collections::HashMap::new();
    let mut palt_chunks = Vec::new();

    const INVALID_TRANSPARENT_COLOR_INDEX: u8 = 255;
    palt_transparent_color_indexes
        .entry(iff::ChunkId::invalid())
        .or_insert_with(|| INVALID_TRANSPARENT_COLOR_INDEX);

    for sprite in sprites {
        palt_transparent_color_indexes.entry(sprite.palette_chunk_id).or_insert_with(|| {
            let color_sprite_file_path = source_directory.join(
                sprite
                    .sprite_frames
                    .first()
                    .unwrap()
                    .sprite_channel_file_path_relative(spr::SpriteChannelType::Color),
            );
            let (transparent_color_index, palt_chunk) =
                create_palt_chunk(sprite.palette_chunk_id, &color_sprite_file_path);
            palt_chunks.push(palt_chunk);
            transparent_color_index
        });
    }

    (palt_transparent_color_indexes, palt_chunks)
}
