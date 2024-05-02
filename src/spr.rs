use crate::error;
use crate::iff;
use crate::iff_description;
use crate::sprite;

use anyhow::Context;
use serde_with::serde_as;
use serde_with::BoolFromInt;

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SpriteIndex(i32);

impl SpriteIndex {
    pub fn as_i32(self) -> i32 {
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

#[serde_as]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sprite {
    #[serde(rename = "@name")]
    pub chunk_label: String,
    #[serde(rename = "@id")]
    pub chunk_id: iff::IffChunkId,
    #[serde(rename = "@type")]
    pub sprite_type: SpriteType,
    #[serde(rename = "@multitile")]
    multi_tile: i32,
    #[serde(rename = "@defaultpaletteid")]
    pub palette_chunk_id: iff::IffChunkId,
    #[serde(rename = "@framecount")]
    pub sprite_frame_count: i32,
    #[serde(rename = "@iscustomwallstyle")]
    #[serde_as(as = "BoolFromInt")]
    is_custom_wall_style: bool,
    #[serde(rename = "spriteframe")]
    pub sprite_frames: Vec<SpriteFrame>,
}

impl Sprite {
    pub fn new(
        chunk_label: &str,
        chunk_id: iff::IffChunkId,
        palette_chunk_id: iff::IffChunkId,
        sprite_frames: Vec<SpriteFrame>,
    ) -> Sprite {
        Sprite {
            chunk_label: chunk_label.to_owned(),
            chunk_id,
            sprite_type: SpriteType::Spr2,
            multi_tile: 0,
            palette_chunk_id,
            sprite_frame_count: sprite_frames.len().try_into().unwrap(),
            is_custom_wall_style: false,
            sprite_frames,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteFrame {
    #[serde(rename = "@index")]
    pub index: SpriteIndex,
    #[serde(rename = "@zoom")]
    pub zoom_level: sprite::ZoomLevel,
    #[serde(rename = "@rot")]
    pub rotation: sprite::Rotation,
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
    pub palette_chunk_id: iff::IffChunkId,
    #[serde(rename = "@transparentpixel")]
    pub transparent_color_index: u8,
    #[serde(rename = "spritechannel")]
    sprite_channels: Vec<SpriteChannel>,
}

impl SpriteFrame {
    pub fn new(
        index: i32,
        zoom_level: sprite::ZoomLevel,
        rotation: sprite::Rotation,
        sprite_image_description: &sprite::SpriteImageDescription,
        sprite_channel_p_relative_path: &std::path::Path,
        sprite_channel_z_relative_path: &std::path::Path,
        sprite_channel_a_relative_path: &std::path::Path,
    ) -> SpriteFrame {
        let sprite_channels = vec![
            SpriteChannel {
                channel_type: SpriteChannelType::Color,
                file_path_relative: sprite_channel_p_relative_path.to_str().unwrap().to_owned(),
            },
            SpriteChannel {
                channel_type: SpriteChannelType::Depth,
                file_path_relative: sprite_channel_z_relative_path.to_str().unwrap().to_owned(),
            },
            SpriteChannel {
                channel_type: SpriteChannelType::Alpha,
                file_path_relative: sprite_channel_a_relative_path.to_str().unwrap().to_owned(),
            },
        ];
        SpriteFrame {
            index: SpriteIndex(index),
            zoom_level,
            rotation,
            bounds_left: sprite_image_description.bounds.left,
            bounds_top: sprite_image_description.bounds.top,
            bounds_right: sprite_image_description.bounds.right,
            bounds_bottom: sprite_image_description.bounds.bottom,
            width: sprite_image_description.width,
            height: sprite_image_description.height,
            palette_chunk_id: sprite_image_description.palette_id,
            transparent_color_index: sprite_image_description.transparent_color_index,
            sprite_channels,
        }
    }

    pub fn sprite_channel_file_path_relative(
        &self,
        channel_type: SpriteChannelType,
        sprite_id: iff::IffChunkId,
    ) -> anyhow::Result<&str> {
        Ok(&self
            .sprite_channels
            .iter()
            .find(|x| x.channel_type == channel_type)
            .with_context(|| sprite_channel_error(channel_type, sprite_id, self.index))?
            .file_path_relative)
    }

    pub fn sprite_channel_file_path_relative_mut(
        &mut self,
        channel_type: SpriteChannelType,
        sprite_id: iff::IffChunkId,
    ) -> anyhow::Result<&mut String> {
        Ok(&mut self
            .sprite_channels
            .iter_mut()
            .find(|x| x.channel_type == channel_type)
            .with_context(|| sprite_channel_error(channel_type, sprite_id, self.index))?
            .file_path_relative)
    }
}

fn sprite_channel_error(
    channel_type: SpriteChannelType,
    sprite_id: iff::IffChunkId,
    frame_index: SpriteIndex,
) -> String {
    format!(
        "Failed to find {} channel in sprite id: {} frame: {}",
        channel_type,
        sprite_id.as_i16(),
        frame_index.0
    )
}

#[derive(Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum SpriteChannelType {
    #[serde(rename = "p")]
    Color,
    #[serde(rename = "z")]
    Depth,
    #[serde(rename = "a")]
    Alpha,
}

impl std::fmt::Display for SpriteChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            SpriteChannelType::Color => "color",
            SpriteChannelType::Depth => "depth",
            SpriteChannelType::Alpha => "alpha",
        };
        write!(f, "{}", string)
    }
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
    pub fn to_chunk(&self, source_directory: &std::path::Path) -> anyhow::Result<iff::IffChunk> {
        match self.sprite_type {
            SpriteType::Spr1 => self.to_spr1_chunk(source_directory),
            SpriteType::Spr2 => self.to_spr2_chunk(source_directory),
        }
    }

    fn to_spr1_chunk(&self, source_directory: &std::path::Path) -> anyhow::Result<iff::IffChunk> {
        assert!(self.sprite_type == SpriteType::Spr1);

        let mut frame_datas = std::vec::Vec::new();
        for frame in &self.sprite_frames {
            let (width, height, pixels) = {
                let channel_type = if self.is_custom_wall_style {
                    SpriteChannelType::Depth
                } else {
                    SpriteChannelType::Color
                };
                let file_path =
                    source_directory.join(frame.sprite_channel_file_path_relative(channel_type, self.chunk_id)?);
                let mut bmp = read_bmp(&file_path)?;
                bmp.set_indexed_color(true);
                let (width, height) = bmp.dimensions();
                let mut pixels = vec![0u8; usize::try_from(width * height).unwrap()];
                use image::ImageDecoder;
                bmp.read_image(&mut pixels).with_context(|| error::file_read_error(&file_path))?;
                (width, height, pixels)
            };

            let mut frame_data = std::vec::Vec::<u8>::new();
            frame_data.extend_from_slice(&0u32.to_le_bytes());
            frame_data.extend_from_slice(&u16::try_from(height).unwrap().to_le_bytes());
            frame_data.extend_from_slice(&u16::try_from(width).unwrap().to_le_bytes());

            let width = usize::try_from(width).unwrap();
            let height = usize::try_from(height).unwrap();

            let transparent_color_index = if self.is_custom_wall_style {
                255
            } else {
                frame.transparent_color_index
            };

            let pixels = {
                // the transmogrifier exports these sprites with pixels of either 128 or 255
                // but the original iff file is either 0 or 255
                let mut pixels = pixels;
                if self.is_custom_wall_style {
                    for pixel in pixels.iter_mut() {
                        if *pixel != transparent_color_index {
                            *pixel = 0;
                        }
                    }
                }
                pixels
            };

            enum RowCommand {
                StartSprite,
                Start,
                Opaque,
                OpaqueRepeat,
                Transparent,
                TransparentRows,
                EndSprite,
            }

            fn row_command(command: RowCommand) -> u8 {
                match command {
                    RowCommand::StartSprite => 0,
                    RowCommand::Start => 4,
                    RowCommand::Opaque => 3,
                    RowCommand::OpaqueRepeat => 2,
                    RowCommand::Transparent => 1,
                    RowCommand::TransparentRows => 9,
                    RowCommand::EndSprite => 5,
                }
            }

            let start_sprite_command = row_command(RowCommand::StartSprite);
            frame_data.extend_from_slice(&start_sprite_command.to_le_bytes());
            frame_data.extend_from_slice(&0u8.to_le_bytes());

            let mut y = 0;
            while y < height {
                let mut row_commands = std::vec::Vec::new();

                let row_index = y * width;

                if let Some(i) = pixels[row_index..].iter().position(|x| *x != transparent_color_index) {
                    let transparent_row_count = i / width;
                    if transparent_row_count >= 1 {
                        let row_command_length = u8::try_from(transparent_row_count).unwrap();
                        let row_command = row_command(RowCommand::TransparentRows);
                        frame_data.extend_from_slice(&row_command.to_le_bytes());
                        frame_data.extend_from_slice(&row_command_length.to_le_bytes());

                        y += transparent_row_count;
                        continue;
                    }
                }

                let mut x = 0;
                let mut ongoing_unique_range: Option<Vec<u8>> = None;
                const REPEAT_THRESHOLD: usize = 8;
                while x < width {
                    if pixels[row_index + x] == transparent_color_index {
                        let mut transparent_width = 1;
                        while x + transparent_width < width {
                            let color_pixel = pixels[row_index + x + transparent_width];
                            if color_pixel == transparent_color_index {
                                transparent_width += 1;
                            } else {
                                break;
                            }
                        }

                        if x + transparent_width == width {
                            break;
                        }

                        let row_command_length = u8::try_from(transparent_width).unwrap();
                        let row_command = row_command(RowCommand::Transparent);
                        row_commands.extend_from_slice(&row_command.to_le_bytes());
                        row_commands.extend_from_slice(&row_command_length.to_le_bytes());

                        x += transparent_width;
                    } else {
                        let mut range_x = x;
                        while range_x < width {
                            let first_pixel = pixels[row_index + range_x];
                            if first_pixel == transparent_color_index {
                                break;
                            }
                            if range_x + 1 == width {
                                let mut unique_range = ongoing_unique_range.unwrap_or_default();
                                unique_range.push(pixels[row_index + x]);
                                ongoing_unique_range = Some(unique_range);

                                range_x += 1;
                                break;
                            }
                            let next_pixel = pixels[row_index + range_x + 1];

                            if next_pixel == transparent_color_index {
                                let mut unique_range = ongoing_unique_range.unwrap_or_default();
                                unique_range.push(pixels[row_index + x]);
                                ongoing_unique_range = Some(unique_range);

                                range_x += 1;
                                break;
                            }

                            if first_pixel == next_pixel {
                                let mut repeated_width = 1;
                                while range_x + repeated_width < width {
                                    let color_pixel = pixels[row_index + range_x + repeated_width];
                                    if color_pixel == first_pixel {
                                        repeated_width += 1;
                                    } else {
                                        break;
                                    }
                                }

                                if repeated_width >= REPEAT_THRESHOLD && ongoing_unique_range.is_some() {
                                    break;
                                } else if repeated_width >= REPEAT_THRESHOLD && ongoing_unique_range.is_none() {
                                    let row_command_length = u8::try_from(repeated_width).unwrap();
                                    let row_command = row_command(RowCommand::OpaqueRepeat);
                                    row_commands.extend_from_slice(&row_command.to_le_bytes());
                                    row_commands.extend_from_slice(&row_command_length.to_le_bytes());

                                    if self.palette_chunk_id.as_i16().is_positive() {
                                        row_commands.push(pixels[row_index + range_x + x]);
                                    } else {
                                        row_commands.push(0);
                                    }
                                    row_commands.push(0);
                                } else {
                                    let mut unique_range = ongoing_unique_range.unwrap_or_default();
                                    if self.palette_chunk_id.as_i16().is_positive() {
                                        unique_range.extend_from_slice(
                                            &pixels[row_index + range_x..row_index + range_x + repeated_width],
                                        );
                                    } else {
                                        unique_range.resize(unique_range.len() + repeated_width, 0);
                                    }
                                    ongoing_unique_range = Some(unique_range);
                                }

                                range_x += repeated_width;
                            } else {
                                let mut unique_width = 1;
                                let mut previous_pixel = first_pixel;
                                while range_x + unique_width < width {
                                    let color_pixel = pixels[row_index + range_x + unique_width];
                                    if color_pixel != previous_pixel && color_pixel != transparent_color_index {
                                        unique_width += 1;
                                    } else {
                                        break;
                                    }
                                    previous_pixel = color_pixel;
                                }

                                let mut unique_range = ongoing_unique_range.unwrap_or_default();
                                unique_range.extend_from_slice(&pixels[row_index + x..row_index + x + unique_width]);
                                ongoing_unique_range = Some(unique_range);

                                range_x += unique_width;
                            }
                        }

                        x = range_x;
                    }
                    if let Some(range) = ongoing_unique_range.as_mut() {
                        let row_command_length = u8::try_from(range.len()).unwrap();
                        let row_command = row_command(RowCommand::Opaque);
                        row_commands.extend_from_slice(&row_command.to_le_bytes());
                        row_commands.extend_from_slice(&row_command_length.to_le_bytes());

                        row_commands.append(range);
                        if row_command_length % 2 != 0 {
                            row_commands.push(0);
                        }

                        ongoing_unique_range = None;
                    }
                }

                let start_command_length = 2 + u8::try_from(row_commands.len()).unwrap();
                let start_command = row_command(RowCommand::Start);
                frame_data.extend_from_slice(&start_command.to_le_bytes());
                frame_data.extend_from_slice(&start_command_length.to_le_bytes());

                frame_data.extend_from_slice(row_commands.as_slice());

                y += 1;
            }

            let end_sprite_command = row_command(RowCommand::EndSprite);
            frame_data.extend_from_slice(&end_sprite_command.to_le_bytes());
            frame_data.extend_from_slice(&0u8.to_le_bytes());

            frame_datas.push(frame_data);
        }

        const SPR1_VERSION: u32 = 504;

        let mut spr1_data = std::vec::Vec::<u8>::new();
        spr1_data.extend_from_slice(&SPR1_VERSION.to_le_bytes());
        spr1_data.extend_from_slice(&u32::try_from(self.sprite_frame_count).unwrap().to_le_bytes());
        spr1_data.extend_from_slice(&self.palette_chunk_id.as_i32().to_le_bytes());

        let mut frame_address = u32::try_from(
            spr1_data.len() + (usize::try_from(self.sprite_frame_count).unwrap() * std::mem::size_of::<u32>()),
        )
        .unwrap();
        for frame_data in &frame_datas {
            spr1_data.extend_from_slice(&frame_address.to_le_bytes());
            frame_address += u32::try_from(frame_data.len()).unwrap();
        }

        for frame_data in &frame_datas {
            spr1_data.extend_from_slice(frame_data.as_slice());
        }

        let spr1_chunk_header = iff::IffChunkHeader::new(b"SPR#", spr1_data.len(), self.chunk_id, &self.chunk_label)?;

        Ok(iff::IffChunk {
            header: spr1_chunk_header,
            data: spr1_data,
        })
    }

    fn to_spr2_chunk(&self, source_directory: &std::path::Path) -> anyhow::Result<iff::IffChunk> {
        assert!(self.sprite_type == SpriteType::Spr2);

        let mut frame_datas = std::vec::Vec::new();
        for frame in &self.sprite_frames {
            let width = u32::try_from(frame.bounds_right - frame.bounds_left).unwrap();
            let height = u32::try_from(frame.bounds_bottom - frame.bounds_top).unwrap();
            let (pixels_p, pixels_z, pixels_a) = {
                let file_path_p = source_directory
                    .join(frame.sprite_channel_file_path_relative(SpriteChannelType::Color, self.chunk_id)?);
                let file_path_z = source_directory
                    .join(frame.sprite_channel_file_path_relative(SpriteChannelType::Depth, self.chunk_id)?);
                let file_path_a = source_directory
                    .join(frame.sprite_channel_file_path_relative(SpriteChannelType::Alpha, self.chunk_id)?);

                let x = u32::try_from(frame.bounds_left).unwrap();
                let y = u32::try_from(frame.bounds_top).unwrap();

                let mut bmp_p = read_bmp(&file_path_p)?;
                let mut bmp_z = read_bmp(&file_path_z)?;
                let mut bmp_a = read_bmp(&file_path_a)?;

                let pixels_p = read_bmp_rect(&mut bmp_p, x, y, width, height)
                    .with_context(|| error::file_read_error(&file_path_p))?;
                let pixels_z = read_bmp_rect(&mut bmp_z, x, y, width, height)
                    .with_context(|| error::file_read_error(&file_path_z))?;
                let pixels_a = read_bmp_rect(&mut bmp_a, x, y, width, height)
                    .with_context(|| error::file_read_error(&file_path_a))?;

                (pixels_p, pixels_z, pixels_a)
            };

            const SPRITE_FLAGS: u32 = 0b0111;
            let mut frame_data = std::vec::Vec::<u8>::new();
            frame_data.extend_from_slice(&u16::try_from(width).unwrap().to_le_bytes());
            frame_data.extend_from_slice(&u16::try_from(height).unwrap().to_le_bytes());
            frame_data.extend_from_slice(&SPRITE_FLAGS.to_le_bytes());
            frame_data.extend_from_slice(&frame.palette_chunk_id.as_i16().to_le_bytes());
            frame_data.extend_from_slice(&u16::from(frame.transparent_color_index).to_le_bytes());
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

                if let Some(i) = pixels_a[row_index..].iter().position(|x| *x != 0) {
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
                    let alpha_pixel = pixels_a[row_index + x] >> 3;

                    if alpha_pixel == 0 {
                        let mut transparent_width = 1;
                        while x + transparent_width < width {
                            let alpha_pixel = pixels_a[row_index + x + transparent_width];
                            if alpha_pixel == 0 {
                                transparent_width += 1;
                            } else {
                                break;
                            }
                        }
                        if x + transparent_width == width {
                            break;
                        }

                        let row_command_length = u16::try_from(transparent_width).unwrap();
                        let row_command = row_command(RowCommand::Transparent, row_command_length);
                        row_commands.extend_from_slice(&row_command.to_le_bytes());

                        x += transparent_width;
                    } else if alpha_pixel < 31 {
                        let mut translucent_color_width = 1;
                        while x + translucent_color_width < width {
                            let alpha_pixel = pixels_a[row_index + x + translucent_color_width] >> 3;

                            if alpha_pixel > 0 && alpha_pixel < 31 {
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
                            let alpha_pixel = pixels_a[row_index + x + color_width] >> 3;

                            if alpha_pixel == 31 {
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

        let spr2_chunk_header = iff::IffChunkHeader::new(b"SPR2", spr2_data.len(), self.chunk_id, &self.chunk_label)?;

        Ok(iff::IffChunk {
            header: spr2_chunk_header,
            data: spr2_data,
        })
    }
}

fn read_bmp(
    file_path: &std::path::Path,
) -> anyhow::Result<image::codecs::bmp::BmpDecoder<std::io::BufReader<std::fs::File>>> {
    let bmp_buffer =
        std::io::BufReader::new(std::fs::File::open(file_path).with_context(|| error::file_read_error(file_path))?);
    let bmp = image::codecs::bmp::BmpDecoder::new(bmp_buffer).with_context(|| error::file_read_error(file_path))?;
    anyhow::ensure!(
        bmp.get_palette().is_some(),
        format!("{} is not an 8-bit bmp", file_path.display())
    );
    Ok(bmp)
}

fn read_bmp_rect(
    bmp: &mut image::codecs::bmp::BmpDecoder<std::io::BufReader<std::fs::File>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> anyhow::Result<Vec<u8>> {
    bmp.set_indexed_color(true);
    let mut pixels = vec![0u8; usize::try_from(width * height).unwrap()];
    use image::ImageDecoderRect;
    bmp.read_rect(x, y, width, height, &mut pixels, usize::try_from(width).unwrap())?;
    Ok(pixels)
}

pub fn deserialize_sprites<'de, D>(deserializer: D) -> Result<iff_description::Sprites, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let sprites = iff_description::Sprites::deserialize(deserializer)?;

    for sprite in &sprites.sprites {
        if let Ok(sprite_frames_len) = i32::try_from(sprite.sprite_frames.len()) {
            if sprite.sprite_frame_count != sprite_frames_len {
                return Err(serde::de::Error::custom(format!(
                    "frame count of {} does not match amount of frames in sprite {} {}",
                    sprite.sprite_frame_count,
                    sprite.chunk_id.as_i16(),
                    sprite.chunk_label,
                )));
            }
        } else {
            return Err(serde::de::Error::custom(format!(
                "sprite {} {} has too many frames",
                sprite.chunk_id.as_i16(),
                sprite.chunk_label,
            )));
        }

        for (frame, index) in sprite.sprite_frames.iter().zip(0i32..) {
            if frame.index.as_i32() != index {
                return Err(serde::de::Error::custom(format!(
                    "index of {} is incorrect for frame {} of sprite {} {}",
                    frame.index.as_i32(),
                    index,
                    sprite.chunk_id.as_i16(),
                    sprite.chunk_label,
                )));
            }

            match sprite.sprite_type {
                SpriteType::Spr1 => {
                    if frame.sprite_channels.len() != 1 {
                        return Err(serde::de::Error::custom(format!(
                            "expected 1 channel in frame {} of sprite {} {}",
                            frame.index.as_i32(),
                            sprite.chunk_id.as_i16(),
                            sprite.chunk_label,
                        )));
                    } else if frame.sprite_channels[0].channel_type != SpriteChannelType::Depth {
                        return Err(serde::de::Error::custom(format!(
                            "expected depth channel in frame {} of sprite {} {}",
                            frame.index.as_i32(),
                            sprite.chunk_id.as_i16(),
                            sprite.chunk_label,
                        )));
                    }
                }
                SpriteType::Spr2 => {
                    if frame.sprite_channels.len() != 3 {
                        return Err(serde::de::Error::custom(format!(
                            "expected 3 channels in frame {} of sprite {} {}",
                            frame.index.as_i32(),
                            sprite.chunk_id.as_i16(),
                            sprite.chunk_label,
                        )));
                    } else {
                        let channel_types = [
                            SpriteChannelType::Color,
                            SpriteChannelType::Depth,
                            SpriteChannelType::Alpha,
                        ];
                        for (i, channel_type) in channel_types.iter().enumerate() {
                            if frame.sprite_channels[i].channel_type != *channel_type {
                                return Err(serde::de::Error::custom(format!(
                                    "expected {} channel in channel {} of frame {} of sprite {} {}",
                                    channel_type,
                                    i,
                                    frame.index.as_i32(),
                                    sprite.chunk_id.as_i16(),
                                    sprite.chunk_label,
                                )));
                            }
                        }
                    }
                }
            };

            for channel in &frame.sprite_channels {
                if channel.file_path_relative.is_empty() {
                    return Err(serde::de::Error::custom(format!(
                        "no file path found in {} channel of frame {} of sprite {} {}",
                        channel.channel_type,
                        frame.index.as_i32(),
                        sprite.chunk_id.as_i16(),
                        sprite.chunk_label,
                    )));
                }
            }
        }
    }

    Ok(sprites)
}
