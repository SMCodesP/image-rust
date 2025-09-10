use image::{DynamicImage, EncodableLayout, GenericImageView, ImageBuffer, ImageError, ImageFormat, Rgb, Rgba};
use std::{collections::HashMap, time::Instant};
use std::io::Cursor;
use webp::{Encoder as WebPEncoder, PixelLayout};
use fast_image_resize::{self as fir, IntoImageView};
use fast_image_resize::images::Image;
use ravif::{self, Img};

pub async fn process_image(
    image_data: &[u8],
    content_type: &str,
    operations: &str,
) -> Result<Vec<u8>, ImageError> {
    println!("Current content type: {}", content_type);
    let current_format = match content_type {
        "image/png" => ImageFormat::Png,
        "image/webp" => ImageFormat::WebP,
        "image/avif" => ImageFormat::Avif,
        _ => ImageFormat::Jpeg,
    };

    let start_total = Instant::now();
    let start = Instant::now();
    let mut img = image::load_from_memory(image_data)?;
    println!("Time to load image: {} ms", start.elapsed().as_millis());

    // Parse das operações
    let operations_map: HashMap<_, _> = operations
        .split(',')
        .filter_map(|op| {
            let mut parts = op.split('=');
            Some((parts.next()?, parts.next()?))
        })
        .collect();

    let quality: u8 = operations_map
        .get("quality")
        .and_then(|q| q.parse::<u8>().ok())
        .unwrap_or(75);

    let start_resize = Instant::now();
    if let Some(width_str) = operations_map.get("width") {
        if let Ok(target_width) = width_str.parse::<u32>() {
            let (orig_width, orig_height) = img.dimensions();
            let target_height = ((orig_height as f32) * (target_width as f32 / orig_width as f32))
                .round() as u32;

            let pixel_type = img.pixel_type().expect("Failed to get pixel type");

            let mut dst_image = Image::new(target_width, target_height, pixel_type);

            let start_resize = Instant::now();
            let mut resizer = fir::Resizer::new();
            resizer.resize(&img, &mut dst_image, None).unwrap();
            println!("Time to resize image: {} ms", start_resize.elapsed().as_millis());

            let dst_buf = dst_image.into_vec();

            img = match pixel_type {
                fir::PixelType::U8x3 => {
                    let buf = ImageBuffer::<Rgb<u8>, _>::from_raw(target_width, target_height, dst_buf)
                        .expect("Failed to create RGB image");
                    DynamicImage::ImageRgb8(buf)
                }
                fir::PixelType::U8x4 => {
                    let buf = ImageBuffer::<Rgba<u8>, _>::from_raw(target_width, target_height, dst_buf)
                        .expect("Failed to create RGBA image");
                    DynamicImage::ImageRgba8(buf)
                }
                _ => panic!("Unsupported pixel type for dynamic reconstruction"),
            };
        }
    }
    println!("Time to resize image: {} ms", start_resize.elapsed().as_millis());

    let start_encode = Instant::now();
    let format = match operations_map.get("format") {
        Some(&"png") => ImageFormat::Png,
        Some(&"webp") => ImageFormat::WebP,
        Some(&"avif") => ImageFormat::Avif,
        _ => ImageFormat::Jpeg,
    };

    let mut buf = Vec::new();

    match format {
        ImageFormat::WebP if current_format != ImageFormat::WebP => {
            let rgba = img.to_rgba8();
            let encoder = WebPEncoder::new(&rgba, PixelLayout::Rgba, img.width(), img.height());
            let quality = quality as f32;
            let webp_data = encoder.encode(quality);
            buf = webp_data.as_bytes().to_vec();
        }
        ImageFormat::Avif => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();

            let pixels: Vec<rgb::RGBA8> = rgba.pixels()
                .map(|p| rgb::RGBA8 {
                    r: p.0[0],
                    g: p.0[1],
                    b: p.0[2],
                    a: p.0[3],
                })
                .collect();
            
            let img_ref = Img::new(&pixels[..], width.try_into().unwrap(), height.try_into().unwrap());

            let encoder = ravif::Encoder::new()
                .with_quality(quality as f32)
                .with_speed(10);
            let encoded = encoder
                .encode_rgba(img_ref).unwrap();
            buf = encoded.avif_file;
        }
        _ => {
            img.write_to(&mut Cursor::new(&mut buf), format)?;
        }
    }
    println!("Time to encode image: {} ms", start_encode.elapsed().as_millis());

    println!("Total function time: {} ms", start_total.elapsed().as_millis());
    Ok(buf)
}