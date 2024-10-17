use super::flags::ModError;
use super::super::files::FileDefinition;

use std::time::SystemTime;
use std::io::Cursor;
use chrono::{DateTime, SecondsFormat, Utc};
use regex::{Regex, RegexBuilder};
use image_dds::ddsfile;
use webp::*;
use image::DynamicImage;
use base64::{Engine as _, engine::general_purpose};

pub fn parse_xml(content: &str) -> Result<roxmltree::Document<'_>, roxmltree::Error> {
    roxmltree::Document::parse(content)
}

pub fn test_file_name(mod_record : &mut super::structs::ModRecord) -> bool {
    if !mod_record.file_detail.is_folder && ! mod_record.file_detail.full_path.ends_with(".zip") {
        if mod_record.file_detail.full_path.ends_with(".rar") {
            mod_record.add_issue(ModError::FileErrorUnsupportedArchive);
        } else if mod_record.file_detail.full_path.ends_with(".7z") {
            mod_record.add_issue(ModError::FileErrorUnsupportedArchive);
        } else {
            mod_record.add_issue(ModError::FileErrorGarbageFile);
        }
        return false
    }

    let regex_zip_pack = RegexBuilder::new(r"unzip")
        .case_insensitive(true)
        .build()
        .unwrap();

    if regex_zip_pack.is_match(&mod_record.file_detail.short_name) {
        mod_record.add_issue(ModError::FileErrorLikelyZipPack);
    }

    let regex_digit = Regex::new(r"^\d").unwrap();

    if regex_digit.is_match(&mod_record.file_detail.short_name) {
        mod_record.add_issue(ModError::FileErrorNameStartsDigit);
    }

    let regex_good_file = Regex::new(r"^[A-Z_a-z]\w+$").unwrap();
    let regex_copy_capture = Regex::new(r"^(?<name>[A-Za-z]\w+)(?: - .+$| \(.+$)").unwrap();

    if ! regex_good_file.is_match(&mod_record.file_detail.short_name) {
        let Some(caps) = regex_copy_capture.captures(&mod_record.file_detail.short_name) else {
            return false
        };
        mod_record.issues.insert(ModError::FileErrorLikelyCopy);
        mod_record.file_detail.copy_name = Some(caps["name"].to_owned());
        return false
    }

    true
}

pub fn do_file_counts(mod_record : &mut super::structs::ModRecord, file_list : &Vec<FileDefinition>) {
    let mut max_grle = 10;
    let mut max_pdf = 1;
    let mut max_png = 128;
    let mut max_txt = 2;

    let size_cache :u64 = 10485760;
    let size_dds :u64 = 12582912;
    let size_gdm :u64 = 18874368;
    let size_shapes :u64 = 268435456;
    let size_xml :u64 = 262144;

    let known_good = vec!["png", "dds", "i3d", "shapes", "lua", "gdm", "cache", "xml", "grle", "pdf", "txt", "gls", "anim", "ogg"];

    for file in file_list {
        if file.is_folder { continue }

        let this_path = std::path::Path::new(&file.name);
        let this_ext = match this_path.extension() {
            Some(val) => val.to_str().unwrap(),
            None => { continue; }
        };

        if ! known_good.contains(&this_ext) {
            if this_ext == "dat" || this_ext == "l64" {
                mod_record.add_issue(ModError::InfoLikelyPiracy);
            }
            mod_record.add_issue(ModError::PerformanceQuantityExtra);
            mod_record.file_detail.extra_files.push(file.name.clone());
        } else {
            if file.name.contains(" ") {
                mod_record.add_issue(ModError::PerformanceFileSpaces);
                mod_record.file_detail.space_files.push(file.name.clone());
            }
            match this_ext {
                "png"  => {
                    if ! file.name.ends_with("_weight.png") {
                        mod_record.file_detail.image_non_dds.push(file.name.clone());
                        mod_record.file_detail.png_texture.push(file.name.clone());
                    }
                    max_png -= 1
                },
                "pdf"  => max_pdf -= 1,
                "grle" => max_grle -= 1,
                "txt"  => max_txt -= 1,
                "cache"  => if file.size > size_cache { mod_record.add_issue(ModError::PerformanceOversizeI3D); },
                "dds"    => {
                    mod_record.file_detail.image_dds.push(file.name.clone());
                    if file.size > size_dds { mod_record.add_issue(ModError::PerformanceOversizeDDS); }
                },
                "gdm"    => if file.size > size_gdm { mod_record.add_issue(ModError::PerformanceOversizeGDM); },
                "shapes" => if file.size > size_shapes { mod_record.add_issue(ModError::PerformanceOversizeSHAPES); },
                "xml"    => if file.size > size_xml { mod_record.add_issue(ModError::PerformanceOversizeXML); },
                _ => {},
            }

            if max_grle < 0 { mod_record.add_issue(ModError::PerformanceQuantityGRLE); }
            if max_pdf < 0 { mod_record.add_issue(ModError::PerformanceQuantityPDF); }
            if max_png < 0 { mod_record.add_issue(ModError::PerformanceQuantityPNG); }
            if max_txt < 0 { mod_record.add_issue(ModError::PerformanceQuantityTXT); }
        }
    }
}

pub fn sys_time_to_string(now: SystemTime) -> String {
    let now: DateTime<Utc> = now.into();
    now.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn mod_desc_basics(mod_record : &mut super::structs::ModRecord, mod_desc : &roxmltree::Document) {
    match mod_desc.root_element().attribute("descVersion") {
        Some(val) => mod_record.mod_desc.desc_version = val.parse().unwrap(),
        None => { mod_record.add_issue(ModError::ModDescVersionOldOrMissing); },
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("version")) {
        Some(node) => mod_record.mod_desc.version = node.text().unwrap().to_owned(),
        None => { mod_record.add_issue(ModError::ModDescNoModVersion); }
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("author")) {
        Some(node) => mod_record.mod_desc.author = node.text().unwrap_or_else(|| {"--"}).to_owned(),
        None => {}
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("multiplayer")) {
        Some(node) => match node.attribute("supported") {
            Some(val) => mod_record.mod_desc.multi_player = val.parse().unwrap(),
            None => {}
        },
        None => {}
    }

    mod_record.mod_desc.store_items = mod_desc.descendants().filter(|n| n.has_tag_name("storeItem")).count() as u32;

    match mod_desc.descendants().find(|n| n.has_tag_name("map")) {
        Some(node) => match node.attribute("configFilename") {
            Some(val) => mod_record.mod_desc.map_config_file = Some(val.to_owned()),
            None => {}
        },
        None => {}
    }

    for depend in mod_desc.descendants().filter(|n| n.has_tag_name("dependency")) {
        mod_record.mod_desc.depend.push(depend.text().unwrap_or_else(|| {"--"}).to_owned())
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("productId")) {
        Some(..) => { mod_record.add_issue(ModError::InfoLikelyPiracy); },
        None => {}
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("iconFilename")) {
        Some(node) => {
            match node.text() {
                Some(val) => {
                    let mut value_string = val.to_string();
                    match value_string.find(".png") {
                        Some(index) => { value_string.replace_range(index..value_string.len(), ".dds"); },
                        None => {}
                    }
                    if mod_record.file_detail.image_dds.contains(&value_string) {
                        mod_record.mod_desc.icon_file_name = Some(value_string);
                    } else {
                        mod_record.add_issue(ModError::ModDescNoModIcon);
                    }
                },
                None => { mod_record.add_issue(ModError::ModDescNoModIcon); }
            }
        },
        None => { mod_record.add_issue(ModError::ModDescNoModIcon); }
    }

    for action in mod_desc.descendants().filter(|n| n.has_tag_name("action")) {
        match action.attribute("name") {
            Some(name) => {
                mod_record.mod_desc.actions.insert(name.to_owned(), match action.attribute("category"){
                    Some(cat) => cat.to_owned(),
                    None => "ALL".to_string(),
                });
            },
            None => {}
        }
    }

    for action in mod_desc.descendants().filter(|n| n.has_tag_name("actionBinding")) {
        match action.attribute("action") {
            Some(name) => {
                mod_record.mod_desc.binds.insert(
                    name.to_owned(),
                    action
                        .children()
                        .filter(|n|n.has_tag_name("binding") && n.attribute("device") == Some("KB_MOUSE_DEFAULT") && n.has_attribute("input"))
                        .map(|x| x.attribute("input").unwrap().to_string())
                        .collect()
                );
            },
            None => {}
        }
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("title")) {
        Some(titles) => {
            if titles.is_text() {
                mod_record.l10n.title.insert(
                    "en".to_string(),
                    titles.text().unwrap_or_else(||"--").to_string()
                );
                mod_record.add_issue(ModError::PerformanceMissingL10N);
            } else {
                for title in titles.children().filter(|n|n.is_element()) {
                    mod_record.l10n.title.insert(
                        title.tag_name().name().to_string(),
                        title.text().unwrap_or_else(||"--").to_string()
                    );
                }
            }
        },
        None => { mod_record.add_issue(ModError::PerformanceMissingL10N); },
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("description")) {
        Some(descriptions) => {
            if descriptions.is_text() {
                mod_record.l10n.description.insert(
                    "en".to_string(),
                    descriptions.text().unwrap_or_else(||"").to_string()
                );
                mod_record.add_issue(ModError::PerformanceMissingL10N);
            } else {
                for description in descriptions.children().filter(|n|n.is_element()) {
                    mod_record.l10n.description.insert(
                        description.tag_name().name().to_string(),
                        description.text().unwrap_or_else(||"").to_string()
                    );
                }
            }
        },
        None => { mod_record.add_issue(ModError::PerformanceMissingL10N); },
    }
}

pub fn load_mod_icon(bin_file: Vec<u8>) -> Option<String> {
    let input_vector = Cursor::new(bin_file);
    let dds = ddsfile::Dds::read(input_vector).unwrap();
    let original_image = image_dds::image_from_dds(&dds, 0).unwrap();
    let unscaled_image = DynamicImage::ImageRgba8(original_image);
    let encoder: Encoder = Encoder::from_image(&unscaled_image).unwrap();
    let webp: WebPMemory = encoder.encode(75f32);
    let b64 = general_purpose::STANDARD.encode(webp.as_ref());

    Some(format!("data:image/webp;base64, {b64}"))
}

pub fn nullify_base_game_entry(xml_tree: &roxmltree::Document, tag : &str) -> Option<String> {
    match xml_tree.descendants().find(|n| n.has_tag_name(tag)) {
        Some(node) => match node.attribute("filename") {
            Some(val) => if val.starts_with("$data") { None } else { Some(val.to_string()) },
            None => None
        },
        None => None
    }
}
pub fn get_base_game_entry_key(xml_tree: &roxmltree::Document) -> Option<String> {
    match xml_tree.descendants().find(|n| n.has_tag_name("environment")) {
        Some(node) => match node.attribute("filename") {
            Some(val) => if ! val.starts_with("$data") { None } else {
                let re = Regex::new(r"(map[A-Z][A-Za-z]+)").unwrap();
                match re.captures(val) {
                    Some(capture) => Some(capture.get(0).unwrap().as_str().to_owned()),
                    None => None
                }
            },
            None => None
        },
        None => None
    }
}

