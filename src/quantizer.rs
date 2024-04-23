use crate::palt;

pub fn posterize(color: u8, bits: u8) -> u8 {
    (color & !((1 << bits) - 1)) | (color >> (8 - bits))
}

pub fn posterize_normalized(color: f32, bits: u8) -> f32 {
    let color = (color * 255.0) as u8;
    posterize(color, bits) as f32 / 255.0
}

pub struct R5g6b5Image(image::RgbImage);

pub fn dither_color_sprite_to_r5g6b5(image: image::RgbImage) -> R5g6b5Image {
    let mut image = image::DynamicImage::ImageRgb8(image).into_rgb32f();
    for y in 0..image.height() {
        for x in 0..image.width() {
            let old_pixel = *image.get_pixel(x, y);
            let new_pixel = image::Rgb([
                posterize_normalized(old_pixel[0], 3),
                posterize_normalized(old_pixel[1], 2),
                posterize_normalized(old_pixel[2], 3),
            ]);

            let error = [
                old_pixel[0] - new_pixel[0],
                old_pixel[1] - new_pixel[1],
                old_pixel[2] - new_pixel[2],
            ];

            image.put_pixel(x, y, new_pixel);

            if x + 1 < image.width() {
                let pixel = image.get_pixel_mut(x + 1, y);
                pixel[0] += error[0] * 7.0 / 16.0;
                pixel[1] += error[1] * 7.0 / 16.0;
                pixel[2] += error[2] * 7.0 / 16.0;
            }

            if x != 0 && y + 1 < image.height() {
                let pixel = image.get_pixel_mut(x - 1, y + 1);
                pixel[0] += error[0] * 3.0 / 16.0;
                pixel[1] += error[1] * 3.0 / 16.0;
                pixel[2] += error[2] * 3.0 / 16.0;
            }

            if y + 1 < image.height() {
                let pixel = image.get_pixel_mut(x, y + 1);
                pixel[0] += error[0] * 5.0 / 16.0;
                pixel[1] += error[1] * 5.0 / 16.0;
                pixel[2] += error[2] * 5.0 / 16.0;
            }

            if x + 1 < image.width() && y + 1 < image.height() {
                let pixel = image.get_pixel_mut(x + 1, y + 1);
                pixel[0] += error[0] * 1.0 / 16.0;
                pixel[1] += error[1] * 1.0 / 16.0;
                pixel[2] += error[2] * 1.0 / 16.0;
            }
        }
    }
    R5g6b5Image(image::DynamicImage::ImageRgb32F(image).into_rgb8())
}

const QUANTIZER_TRANSPARENT_COLOR: imagequant::RGBA = imagequant::RGBA::new(255, 255, 0, 1);
const TRANSPARENT_COLOR_INDEX: u8 = 0;

pub struct Histogram {
    quantizer: imagequant::Attributes,
    histogram: imagequant::Histogram,
    colors: std::collections::HashMap<imagequant::RGBA, u32>,
}

impl Histogram {
    pub fn new() -> Self {
        let mut quantizer = imagequant::new();
        quantizer.set_max_colors(u32::from(palt::PALT_COLOR_ENTRY_COUNT) - 1).unwrap();
        let histogram = imagequant::Histogram::new(&quantizer);
        Histogram {
            quantizer,
            colors: std::collections::HashMap::new(),
            histogram,
        }
    }

    pub fn add_colors(&mut self, color: &R5g6b5Image, alpha: &image::Rgb32FImage) {
        for (rgb, a) in color.0.pixels().zip(alpha.pixels()) {
            if a[0] > 0.0 {
                self.colors
                    .entry(imagequant::RGBA::new(rgb[0], rgb[1], rgb[2], 255))
                    .and_modify(|x| *x += 1)
                    .or_insert(1u32);
            }
        }
    }

    pub fn finalize(mut self) -> anyhow::Result<Quantizer> {
        anyhow::ensure!(!self.colors.is_empty(), "No colors added to histogram");
        let histogram_colors: Vec<_> = self
            .colors
            .iter()
            .map(|(color, count)| imagequant::HistogramEntry {
                color: *color,
                count: *count,
            })
            .collect();
        self.histogram.add_colors(&histogram_colors, 0.0).unwrap();
        let mut quantization_result = self.histogram.quantize(&self.quantizer).unwrap();

        let palette_set: std::collections::HashSet<[u8; 3]> = quantization_result
            .palette()
            .iter()
            .map(|x| [posterize(x.r, 3), posterize(x.g, 2), posterize(x.b, 3)])
            .collect();
        let palette = {
            let mut palette: Vec<[u8; 3]> = palette_set.into_iter().collect();

            let mut histogram_colors = histogram_colors.clone();
            histogram_colors.sort_by(|a, b| a.count.cmp(&b.count).reverse());
            while palette.len() < std::cmp::min(usize::from(palt::PALT_COLOR_ENTRY_COUNT) - 1, histogram_colors.len()) {
                for entry in &histogram_colors {
                    if !palette.iter().any(|x| x[0] == entry.color.r && x[1] == entry.color.g && x[2] == entry.color.b)
                    {
                        palette.push([entry.color.r, entry.color.g, entry.color.b]);
                        break;
                    }
                }
            }
            while palette.len() < usize::from(palt::PALT_COLOR_ENTRY_COUNT) - 1 {
                palette.push([0, 0, 0]);
            }
            palette
        };

        let mut histogram = imagequant::Histogram::new(&self.quantizer);
        histogram.add_fixed_color(QUANTIZER_TRANSPARENT_COLOR, 0.0).unwrap();
        for color in &palette {
            histogram
                .add_fixed_color(
                    imagequant::RGBA {
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: 255,
                    },
                    0.0,
                )
                .unwrap();
        }
        let quantization_result = histogram.quantize(&self.quantizer).unwrap();
        let mut final_palette = vec![[255, 255, 0]];
        final_palette.extend(&palette);

        assert!(final_palette.len() == 256);
        for color in &final_palette {
            assert!(color[0] == posterize(color[0], 3));
            assert!(color[1] == posterize(color[1], 2));
            assert!(color[2] == posterize(color[2], 3));
        }

        Ok(Quantizer {
            quantizer: self.quantizer,
            quantization_result,
            palette: final_palette,
            transparent_color_index: TRANSPARENT_COLOR_INDEX,
        })
    }
}

pub struct Quantizer {
    quantizer: imagequant::Attributes,
    quantization_result: imagequant::QuantizationResult,
    pub palette: Vec<[u8; 3]>,
    pub transparent_color_index: u8,
}

impl Quantizer {
    pub fn quantize(&mut self, color: &R5g6b5Image, alpha: &image::Rgb32FImage) -> image::GrayImage {
        let quantizer_pixels: Vec<_> = color
            .0
            .pixels()
            .zip(alpha.pixels())
            .map(|(rgb, a)| {
                if a[0] > 0.0 {
                    imagequant::RGBA::new(rgb[0], rgb[1], rgb[2], 255)
                } else {
                    QUANTIZER_TRANSPARENT_COLOR
                }
            })
            .collect();
        let mut quantizer_image = self
            .quantizer
            .new_image(
                quantizer_pixels.as_slice(),
                usize::try_from(color.0.width()).unwrap(),
                usize::try_from(color.0.height()).unwrap(),
                0.0,
            )
            .unwrap();

        let (_, quantized_pixels) = self.quantization_result.remapped(&mut quantizer_image).unwrap();

        image::GrayImage::from_raw(color.0.width(), color.0.height(), quantized_pixels).unwrap()
    }
}
