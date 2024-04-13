use crate::dgrp;
use crate::iff;

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SpriteIndex(u32);

impl SpriteIndex {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum SpriteType {
    #[serde(rename = "1")]
    Spr1,
    #[serde(rename = "2")]
    Spr2,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sprite {
    #[serde(rename = "@name")]
    pub chunk_label: String,
    #[serde(rename = "@id")]
    pub chunk_id: iff::ChunkId,
    #[serde(rename = "@type")]
    pub sprite_type: SpriteType,
    #[serde(rename = "@multitile")]
    multi_tile: i32,
    #[serde(rename = "@defaultpaletteid")]
    pub palette_chunk_id: iff::ChunkId,
    #[serde(rename = "@framecount")]
    pub frame_count: i32,
    #[serde(rename = "@iscustomwallstyle")]
    is_custom_wall_style: i32,
    #[serde(rename = "spriteframe")]
    pub sprite_frames: Vec<SpriteFrame>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteFrame {
    #[serde(rename = "@index")]
    pub index: SpriteIndex,
    #[serde(rename = "@zoom")]
    pub zoom_level: dgrp::ZoomLevel,
    #[serde(rename = "@rot")]
    rotation: i32,
    #[serde(rename = "@x")]
    pub bounds_left: i16,
    #[serde(rename = "@y")]
    pub bounds_top: i16,
    #[serde(skip)]
    pub bounds_right: i16,
    #[serde(skip)]
    pub bounds_bottom: i16,
    #[serde(rename = "@width")]
    pub width: i16,
    #[serde(rename = "@height")]
    pub height: i16,
    #[serde(rename = "@paletteid")]
    pub palette_chunk_id: iff::ChunkId,
    #[serde(rename = "@transparentpixel")]
    pub transparent_colour_index: u8,
    #[serde(rename = "spritechannel")]
    sprite_channels: Vec<SpriteChannel>,
}

impl SpriteFrame {
    pub fn sprite_channel_file_path_relative(&self, channel_type: SpriteChannelType) -> &str {
        &self.sprite_channels.iter().find(|x| x.channel_type == channel_type).unwrap().file_path_relative
    }
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum SpriteChannelType {
    #[serde(rename = "p")]
    Colour,
    #[serde(rename = "z")]
    Depth,
    #[serde(rename = "a")]
    Alpha,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct SpriteChannel {
    #[serde(rename = "@type")]
    channel_type: SpriteChannelType,
    #[serde(rename = "@filename")]
    file_path_relative: String,
}

impl Sprite {
    pub fn to_chunk_bytes(&self, source_directory: &std::path::Path) -> Vec<u8> {
        match self.sprite_type {
            SpriteType::Spr1 => panic!(),
            SpriteType::Spr2 => self.to_spr2_chunk_bytes(source_directory),
        }
    }

    fn to_spr2_chunk_bytes(&self, source_directory: &std::path::Path) -> Vec<u8> {
        assert!(self.sprite_type == SpriteType::Spr2);

        let mut frame_datas = std::vec::Vec::new();
        for frame in &self.sprite_frames {
            let width = u32::try_from(frame.bounds_right - frame.bounds_left).unwrap();
            let height = u32::try_from(frame.bounds_bottom - frame.bounds_top).unwrap();
            let (pixels_p, pixels_z, pixels_a) = {
                let file_path_p =
                    source_directory.join(frame.sprite_channel_file_path_relative(SpriteChannelType::Colour));
                let file_path_z =
                    source_directory.join(frame.sprite_channel_file_path_relative(SpriteChannelType::Depth));
                let file_path_a =
                    source_directory.join(frame.sprite_channel_file_path_relative(SpriteChannelType::Alpha));
                let bmp_buffer_p = std::io::BufReader::new(std::fs::File::open(&file_path_p).unwrap());
                let bmp_buffer_z = std::io::BufReader::new(std::fs::File::open(&file_path_z).unwrap());
                let bmp_buffer_a = std::io::BufReader::new(std::fs::File::open(&file_path_a).unwrap());
                let mut bmp_p = image::codecs::bmp::BmpDecoder::new(bmp_buffer_p).unwrap();
                let mut bmp_z = image::codecs::bmp::BmpDecoder::new(bmp_buffer_z).unwrap();
                let mut bmp_a = image::codecs::bmp::BmpDecoder::new(bmp_buffer_a).unwrap();
                bmp_p.set_indexed_color(true);
                bmp_z.set_indexed_color(true);
                bmp_a.set_indexed_color(true);
                let mut pixels_p = vec![0u8; usize::try_from(width * height).unwrap()];
                let mut pixels_z = vec![0u8; usize::try_from(width * height).unwrap()];
                let mut pixels_a = vec![0u8; usize::try_from(width * height).unwrap()];

                let x = u32::try_from(frame.bounds_left).unwrap();
                let y = u32::try_from(frame.bounds_top).unwrap();
                use image::ImageDecoderRect;
                bmp_p.read_rect(x, y, width, height, &mut pixels_p, usize::try_from(width).unwrap()).unwrap();
                bmp_z.read_rect(x, y, width, height, &mut pixels_z, usize::try_from(width).unwrap()).unwrap();
                bmp_a.read_rect(x, y, width, height, &mut pixels_a, usize::try_from(width).unwrap()).unwrap();

                (pixels_p, pixels_z, pixels_a)
            };

            const SPRITE_FLAGS: u32 = 0b0111;
            let mut frame_data = std::vec::Vec::<u8>::new();
            frame_data.extend_from_slice(&u16::try_from(width).unwrap().to_le_bytes());
            frame_data.extend_from_slice(&u16::try_from(height).unwrap().to_le_bytes());
            frame_data.extend_from_slice(&SPRITE_FLAGS.to_le_bytes());
            frame_data.extend_from_slice(&frame.palette_chunk_id.as_i16().to_le_bytes());
            frame_data.extend_from_slice(&u16::from(frame.transparent_colour_index).to_le_bytes());
            frame_data.extend_from_slice(&u16::try_from(frame.bounds_top).unwrap().to_le_bytes());
            frame_data.extend_from_slice(&u16::try_from(frame.bounds_left).unwrap().to_le_bytes());

            let width = usize::try_from(width).unwrap();
            let height = usize::try_from(height).unwrap();

            enum RowCommand {
                Start,
                Opaque,
                Translucent,
                Transparent,
                TransparentRows,
                End,
            }

            fn row_command(command: RowCommand, size_or_length: u16) -> u16 {
                assert!(size_or_length <= 0b0001111111111111);
                let row_command_bits = match command {
                    RowCommand::Start => 0b0000000000000000,
                    RowCommand::Opaque => 0b0010000000000000,
                    RowCommand::Translucent => 0b0100000000000000,
                    RowCommand::Transparent => 0b0110000000000000,
                    RowCommand::TransparentRows => 0b1000000000000000,
                    RowCommand::End => 0b1010000000000000,
                };
                row_command_bits | size_or_length
            }

            let mut y = 0;
            while y < height {
                let mut row_commands = std::vec::Vec::new();

                let row_index = y * width;

                if let Some(i) = pixels_p[row_index..].iter().position(|x| *x != frame.transparent_colour_index) {
                    let transparent_row_count = i / width;
                    if transparent_row_count >= 1 {
                        let row_command_length = u16::try_from(transparent_row_count).unwrap();
                        let row_command = row_command(RowCommand::TransparentRows, row_command_length);
                        frame_data.extend_from_slice(&row_command.to_le_bytes());

                        y += transparent_row_count;
                        continue;
                    }
                }

                let mut x = 0;
                while x < width {
                    let color_pixel = pixels_p[row_index + x];
                    let alpha_pixel = pixels_a[row_index + x] >> 3;

                    if color_pixel == frame.transparent_colour_index {
                        let mut transparent_width = 1;
                        while x + transparent_width < width {
                            let color_pixel = pixels_p[row_index + x + transparent_width];
                            if color_pixel == frame.transparent_colour_index {
                                transparent_width += 1;
                            } else {
                                break;
                            }
                        }

                        let row_command_length = u16::try_from(transparent_width).unwrap();
                        let row_command = row_command(RowCommand::Transparent, row_command_length);
                        row_commands.extend_from_slice(&row_command.to_le_bytes());

                        x += transparent_width;
                    } else if alpha_pixel < 31 {
                        let mut translucent_color_width = 1;
                        while x + translucent_color_width < width {
                            let color_pixel = pixels_p[row_index + x + translucent_color_width];
                            let alpha_pixel = pixels_a[row_index + x + translucent_color_width] >> 3;

                            if color_pixel != frame.transparent_colour_index && alpha_pixel != 31 {
                                translucent_color_width += 1;
                            } else {
                                break;
                            }
                        }

                        let row_command_length = u16::try_from(translucent_color_width).unwrap();
                        let row_command = row_command(RowCommand::Translucent, row_command_length);
                        row_commands.extend_from_slice(&row_command.to_le_bytes());

                        for x in x..x + translucent_color_width {
                            row_commands.push(pixels_z[row_index + x]);
                            row_commands.push(pixels_p[row_index + x]);
                            row_commands.push(pixels_a[row_index + x] >> 3);
                        }

                        if translucent_color_width % 2 == 1 {
                            row_commands.push(0);
                        }

                        x += translucent_color_width;
                    } else {
                        let mut color_width = 1;
                        while x + color_width < width {
                            let color_pixel = pixels_p[row_index + x + color_width];
                            let alpha_pixel = pixels_a[row_index + x + color_width] >> 3;

                            if color_pixel != frame.transparent_colour_index && alpha_pixel == 31 {
                                color_width += 1;
                            } else {
                                break;
                            }
                        }

                        let row_command_length = u16::try_from(color_width).unwrap();
                        let row_command = row_command(RowCommand::Opaque, row_command_length);
                        row_commands.extend_from_slice(&row_command.to_le_bytes());

                        for x in x..x + color_width {
                            row_commands.push(pixels_z[row_index + x]);
                            row_commands.push(pixels_p[row_index + x]);
                        }

                        x += color_width;
                    }
                }

                let row_command_length = 2 + u16::try_from(row_commands.len()).unwrap();
                let row_command = row_command(RowCommand::Start, row_command_length);
                frame_data.extend_from_slice(&row_command.to_le_bytes());

                frame_data.extend_from_slice(row_commands.as_slice());

                y += 1;
            }

            let row_command = row_command(RowCommand::End, 0);
            frame_data.extend_from_slice(&row_command.to_le_bytes());

            frame_datas.push(frame_data);
        }

        const SPR2_VERSION: u32 = 1000;

        let mut spr2_data = Vec::new();
        spr2_data.extend_from_slice(&SPR2_VERSION.to_le_bytes());
        spr2_data.extend_from_slice(&u32::try_from(self.sprite_frames.len()).unwrap().to_le_bytes());
        spr2_data.extend_from_slice(&self.palette_chunk_id.as_i32().to_le_bytes());

        let sprites_offset = (std::mem::size_of::<u32>() * self.sprite_frames.len()) + spr2_data.len();
        let mut frame_address = u32::try_from(sprites_offset).unwrap();
        for frame_data in &frame_datas {
            spr2_data.extend_from_slice(&frame_address.to_le_bytes());
            frame_address += u32::try_from(frame_data.len()).unwrap();
        }

        for frame_data in &frame_datas {
            spr2_data.extend_from_slice(frame_data.as_slice());
        }

        let mut spr2_chunk = std::vec::Vec::new();
        let spr2_chunk_header = iff::ChunkHeader::new("SPR2", spr2_data.len(), self.chunk_id, &self.chunk_label);
        spr2_chunk_header.write(&mut spr2_chunk);
        spr2_chunk.extend_from_slice(spr2_data.as_slice());

        spr2_chunk
    }
}
