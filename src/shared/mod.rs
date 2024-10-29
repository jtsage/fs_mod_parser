//! Shared data
use base64::{engine::general_purpose, Engine as _};
use image::{imageops::FilterType, DynamicImage};
use image_dds::ddsfile;
use std::io::Cursor;
use webp::{Encoder, WebPMemory};

pub mod errors;
pub mod files;
pub mod structs;

/// Image tag information
#[cfg_attr(test, derive(Debug, PartialEq, Eq, PartialOrd, Ord))]
pub struct ImageFile {
    /// is a base game reference
    pub base_game: Option<String>,
    /// is a local file
    pub local_file: Option<String>,
    /// original string
    pub original: Option<String>
}
impl ImageFile {
    /// did not find tag
    fn fail() -> Self {
        ImageFile {
            base_game: None,
            local_file: None,
            original: None,
        }
    }
    /// found a base game reference
    fn base(name: String) -> Self {
        ImageFile {
            original: Some(name.clone()),
            base_game: Some(name),
            local_file: None,
            
        }
    }
    /// found a local reference
    fn local(name: String) -> Self {
        ImageFile {
            original: Some(name.clone()),
            base_game: None,
            local_file: Some(name),
        }
    }
}
/// Extract the text from an image file tag and normalize the name
///
/// - test if a base game reference
/// - .png -> .dds
/// - Fix slashes
#[must_use]
pub fn extract_and_normalize_image(xml_tree: &roxmltree::Document, tag_name: &str) -> ImageFile {
    normalize_image_file(
        xml_tree
            .descendants()
            .find(|n| n.has_tag_name(tag_name))
            .and_then(|n| n.text()),
    )
}

/// Extract the text from an image file option string and normalize
///
/// - test if a base game reference
/// - .png -> .dds
/// - Fix slashes
#[must_use]
#[inline]
pub fn normalize_image_file(file_node: Option<&str>) -> ImageFile {
    if let Some(entry_text) = file_node {
        if entry_text.starts_with("$data") {
            return ImageFile::base(entry_text.to_owned());
        }

        let mut entry_text = entry_text.to_owned().replace('\\', "/");

        if std::path::Path::new(&entry_text)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("png"))
        {
            entry_text.replace_range(entry_text.len() - 4.., ".dds");
        }

        return ImageFile::local(entry_text);
    }
    ImageFile::fail()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_image_file_base() {
        let response = normalize_image_file(Some("$data/something.dds"));

        let expected = ImageFile {
            original: Some(String::from("$data/something.dds")),
            base_game: Some(String::from("$data/something.dds")),
            local_file: None,
        };

        assert_eq!(response, expected);
    }

    #[test]
    fn test_image_file_local_dds() {
        let response = normalize_image_file(Some("./data/something.dds"));

        let expected = ImageFile {
            original: Some(String::from("./data/something.dds")),
            base_game: None,
            local_file: Some(String::from("./data/something.dds")),
        };

        assert_eq!(response, expected);
    }

    #[test]
    fn test_image_file_local_png() {
        let response = normalize_image_file(Some("./data/something.PNG"));

        let expected = ImageFile {
            original: Some(String::from("./data/something.dds")),
            base_game: None,
            local_file: Some(String::from("./data/something.dds")),
        };

        assert_eq!(response, expected);
    }

    #[test]
    fn test_image_file_fail() {
        let response = normalize_image_file(None);

        let expected = ImageFile {
            base_game: None,
            local_file: None,
            original: None,
        };

        assert_eq!(response, expected);
    }
}

/// Load the mod icon, and convert to webp
///
/// Returns the webp as a base64 string suitable for use
/// with an `<image src="...">` tag.
///
/// Supports DDS BC1-BC7 in one pass, in-memory
#[must_use]
pub fn convert_mod_icon(bin_file: Vec<u8>) -> Option<String> {
    let input_vector: Cursor<Vec<u8>> = Cursor::new(bin_file);
    let dds = ddsfile::Dds::read(input_vector).ok()?;
    let original_image = image_dds::image_from_dds(&dds, 0).ok()?;
    let unscaled_image = DynamicImage::ImageRgba8(original_image);
    let encoder: Encoder = Encoder::from_image(&unscaled_image).ok()?;
    let webp: WebPMemory = encoder.encode(75_f32);
    let b64 = general_purpose::STANDARD.encode(webp.as_ref());

    Some(format!("data:image/webp;base64, {b64}"))
}

/// Load the map image resize, crop, and convert to webp
///
/// Returns the webp as a base64 string suitable for use
/// with an `<image src="...">` tag.
///
/// Supports DDS BC1-BC7 in one pass, in-memory
#[must_use]
pub fn convert_map_image(bin_file: Vec<u8>) -> Option<String> {
    let input_vector = Cursor::new(bin_file);
    let dds = ddsfile::Dds::read(input_vector).ok()?;
    let original_image = image_dds::image_from_dds(&dds, 0).ok()?;
    let unscaled_image = DynamicImage::ImageRgba8(original_image);
    let cropped_image = unscaled_image
        .resize(1024, 1024, FilterType::Nearest)
        .crop(256, 256, 512, 512);
    let encoder: Encoder = Encoder::from_image(&cropped_image).ok()?;
    let webp: WebPMemory = encoder.encode(75_f32);
    let b64 = general_purpose::STANDARD.encode(webp.as_ref());

    Some(format!("data:image/webp;base64, {b64}"))
}
