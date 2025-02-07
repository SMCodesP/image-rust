use image::{EncodableLayout, ImageError, ImageFormat};
use std::collections::HashMap;
use std::io::Cursor;
use webp::{Encoder as WebPEncoder, PixelLayout};

pub async fn process_image(
    image_data: &[u8],
    operations: &str,
) -> Result<Vec<u8>, ImageError> {
    let mut img = image::load_from_memory(image_data)?;

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

    // Aplicar redimensionamento (width)
    if let Some(width) = operations_map.get("width").and_then(|w| w.parse::<u32>().ok()) {
        img = img.resize(width, img.height(), image::imageops::FilterType::Triangle);
    }

    // Determinar formato e content_type
    let format = match operations_map.get("format") {
        Some(&"png") => ImageFormat::Png,
        Some(&"webp") => ImageFormat::WebP,
        Some(&"avif") => ImageFormat::Avif,
        _ => ImageFormat::Jpeg,
    };

    let mut buf = Vec::new();

    // Codificação específica para WebP
    if let ImageFormat::WebP = format {
        let rgba = img.to_rgba8();
        let encoder = WebPEncoder::new(&rgba, PixelLayout::Rgba, img.width(), img.height());
        let quality = quality as f32;
        let webp_data = encoder.encode(quality);
        buf = webp_data.as_bytes().to_vec();
    } else {
        img.write_to(&mut Cursor::new(&mut buf), format)?;
    }

    Ok(buf)
}