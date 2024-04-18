use crate::palt;

fn posterize(color: u8, bits: u8) -> u8 {
    (color & !((1 << bits) - 1)) | (color >> (8 - bits))
}

fn posterize_normalized(color: f32, bits: u8) -> f32 {
    let color = (color * 255.0) as u8;
    posterize(color, bits) as f32 / 255.0
}

#[derive(Copy, Clone, Default, Debug)]
struct R5g6b5 {
    r: f32,
    g: f32,
    b: f32,
}

impl kmeans_colors::Calculate for R5g6b5 {
    fn get_closest_centroid(rgb: &[R5g6b5], centroids: &[R5g6b5], indices: &mut Vec<u8>) {
        for color in rgb.iter() {
            let mut index = 0;
            let mut diff;
            let mut min = core::f32::MAX;
            for (idx, cent) in centroids.iter().enumerate() {
                diff = Self::difference(color, cent);
                if diff < min {
                    min = diff;
                    index = idx;
                }
            }
            indices.push(index as u8);
        }
    }

    fn recalculate_centroids(mut rng: &mut impl rand::Rng, buf: &[R5g6b5], centroids: &mut [R5g6b5], indices: &[u8]) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut temp = R5g6b5::default();
            let mut counter: u64 = 0;
            for (&jdx, &color) in indices.iter().zip(buf) {
                if jdx == idx as u8 {
                    temp.r += color.r;
                    temp.g += color.g;
                    temp.b += color.b;
                    counter += 1;
                }
            }
            if counter != 0 {
                cent.r = posterize_normalized(temp.r / (counter as f32), 3);
                cent.g = posterize_normalized(temp.g / (counter as f32), 2);
                cent.b = posterize_normalized(temp.b / (counter as f32), 3);
            } else {
                *cent = Self::create_random(&mut rng);
            }
        }
    }

    fn check_loop(centroids: &[R5g6b5], old_centroids: &[R5g6b5]) -> f32 {
        let mut temp = R5g6b5::default();
        for (&c0, &c1) in centroids.iter().zip(old_centroids) {
            temp.r += c0.r - c1.r;
            temp.g += c0.g - c1.g;
            temp.b += c0.b - c1.b;
        }
        (temp.r).powi(2) + (temp.g).powi(2) + (temp.b).powi(2)
    }

    #[inline]
    fn create_random(rng: &mut impl rand::Rng) -> R5g6b5 {
        R5g6b5 {
            r: posterize_normalized(rng.gen_range(0.0..=1.0), 3),
            g: posterize_normalized(rng.gen_range(0.0..=1.0), 2),
            b: posterize_normalized(rng.gen_range(0.0..=1.0), 3),
        }
    }

    #[inline]
    fn difference(c1: &R5g6b5, c2: &R5g6b5) -> f32 {
        let mut temp = *c1;
        temp.r -= c2.r;
        temp.g -= c2.g;
        temp.b -= c2.b;
        (temp.r).powi(2) + (temp.g).powi(2) + (temp.b).powi(2)
    }
}

const QUANTIZER_TRANSPARENT_COLOR: imagequant::RGBA = imagequant::RGBA::new(255, 255, 0, 1);

pub fn create_color_palette(
    colour_set: &std::collections::HashSet<image::Rgb<u8>>,
    quantizer: &imagequant::Attributes,
) -> (imagequant::QuantizationResult, Vec<[u8; 3]>) {
    let colors: Vec<_> = colour_set
        .iter()
        .map(|x| R5g6b5 {
            r: posterize(x[0], 3) as f32 / 255.0,
            g: posterize(x[1], 2) as f32 / 255.0,
            b: posterize(x[2], 3) as f32 / 255.0,
        })
        .collect();

    let mut palette_kmeans_result = kmeans_colors::Kmeans::new();
    for i in 0..10 {
        let run_result =
            kmeans_colors::get_kmeans(palt::PALT_COLOR_ENTRY_COUNT as usize - 1, 20, 0.0025, false, &colors, i);
        if run_result.score < palette_kmeans_result.score {
            palette_kmeans_result = run_result;
        }
    }

    let histogram_palette: Vec<imagequant::RGBA> = palette_kmeans_result
        .centroids
        .iter()
        .map(|x| imagequant::RGBA::new((x.r * 255.0) as u8, (x.g * 255.0) as u8, (x.b * 255.0) as u8, 255))
        .collect();

    let mut histogram = imagequant::Histogram::new(quantizer);
    histogram.add_fixed_color(QUANTIZER_TRANSPARENT_COLOR, 0.0).unwrap();
    for entry in histogram_palette {
        histogram.add_fixed_color(entry, 0.0).unwrap();
    }

    let mut quantization_result = histogram.quantize(quantizer).unwrap();

    let palette = {
        let mut palette: Vec<_> = quantization_result.palette().iter().map(|x| [x.r, x.g, x.b]).collect();
        while palette.len() < palt::PALT_COLOR_ENTRY_COUNT as usize {
            palette.push([0u8, 0u8, 0u8]);
        }
        palette
    };

    for entry in &palette {
        assert!(entry[0] == posterize(entry[0], 3));
        assert!(entry[1] == posterize(entry[1], 2));
        assert!(entry[2] == posterize(entry[2], 3));
    }

    (quantization_result, palette)
}

pub fn dither_image(
    quantizer: &imagequant::Attributes,
    quantization_result: &mut imagequant::QuantizationResult,
    color: &image::RgbImage,
    alpha: &image::Rgb32FImage,
) -> image::GrayImage {
    let quantizer_pixels: Vec<_> = color
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
    let mut quantizer_image = quantizer
        .new_image(
            quantizer_pixels.as_slice(),
            color.width().try_into().unwrap(),
            color.height().try_into().unwrap(),
            0.0,
        )
        .unwrap();

    let (_, quantized_pixels) = quantization_result.remapped(&mut quantizer_image).unwrap();

    image::GrayImage::from_raw(color.width(), color.height(), quantized_pixels).unwrap()
}
