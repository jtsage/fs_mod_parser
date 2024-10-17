//! Shared data
use std::io::Cursor;
use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, imageops::FilterType};
use image_dds::ddsfile;
use webp::{Encoder, WebPMemory};

pub mod errors;
pub mod structs;
pub mod files;

/// Load the mod icon, and convert to webp
/// 
/// Returns the webp as a base64 string suitable for use
/// with an `<image src="...">` tag.
/// 
/// Supports DDS BC1-BC7 in one pass, in-memory
pub fn convert_mod_icon(bin_file: Vec<u8>) -> Option<String> {
    let input_vector = Cursor::new(bin_file);
    let dds = ddsfile::Dds::read(input_vector).unwrap();
    let original_image = image_dds::image_from_dds(&dds, 0).unwrap();
    let unscaled_image = DynamicImage::ImageRgba8(original_image);
    let encoder: Encoder = Encoder::from_image(&unscaled_image).unwrap();
    let webp: WebPMemory = encoder.encode(75f32);
    let b64 = general_purpose::STANDARD.encode(webp.as_ref());

    Some(format!("data:image/webp;base64, {b64}"))
}

/// Load the map image resize, crop, and convert to webp
/// 
/// Returns the webp as a base64 string suitable for use
/// with an `<image src="...">` tag.
/// 
/// Supports DDS BC1-BC7 in one pass, in-memory
pub fn convert_map_image(bin_file: Vec<u8>) -> Option<String> {
    let input_vector = Cursor::new(bin_file);
    let dds = ddsfile::Dds::read(input_vector).unwrap();
    let original_image = image_dds::image_from_dds(&dds, 0).unwrap();
    let unscaled_image = DynamicImage::ImageRgba8(original_image);
    let cropped_image = unscaled_image
        .resize(1024, 1024, FilterType::Nearest)
        .crop(256, 256, 512, 512);
    let encoder: Encoder = Encoder::from_image(&cropped_image).unwrap();
    let webp: WebPMemory = encoder.encode(75f32);
    let b64 = general_purpose::STANDARD.encode(webp.as_ref());

    Some(format!("data:image/webp;base64, {b64}"))
}