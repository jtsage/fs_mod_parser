//! Parser functions for basic mod reading
use crate::shared::errors::ModError;
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile, FileDefinition};
use crate::shared::structs::ModRecord;
use crate::shared::convert_mod_icon;

use std::{time::SystemTime, path::Path};
use chrono::{DateTime, SecondsFormat, Utc};
use regex::{Regex, RegexBuilder};

/// Known false positives for the malware check
pub const NOT_MALWARE: [&str; 11] = [
    "FS22_001_NoDelete",
    "FS22_AutoDrive",
    "FS22_Courseplay",
    "FS22_FSG_Companion",
    "FS22_VehicleControlAddon",
    "MultiOverlayV3", // Happylooser
    "MultiOverlayV4", // Happylooser
    "VehicleInspector", // Happylooser
    "FS19_AutoDrive",
    "FS19_Courseplay",
    "FS19_GlobalCompany",
];

/* cSpell: disable */
/// Get the JSON representation of a mod in one shot
/// 
/// # Sample Output
/// 
/// ```json
/// {
///  "badgeArray": [ "problem" ],
///  "canNotUse": false,
///  "currentCollection": "",
///  "fileDetail": {
///    "copyName": null,
///    "extraFiles": [],
///    "fileDate": "2024-10-17T02:57:15Z",
///    "fileSize": 461383317,
///    "fullPath": "C:\\...\\FS22_Test.zip",
///    "i3dFiles": [],
///    "imageDDS": [
///      "icon_eldoradoMap.dds",
///    ],
///    "imageNonDDS": [
///      "maps/data/map_dem.png"
///    ],
///    "isFolder": false,
///    "isSaveGame": false,
///    "isModPack": false,
///    "pngTexture": [
///      "maps/data/map_dem.png"
///    ],
///    "shortName": "FS22_EldoradoMap",
///    "spaceFiles": [],
///    "tooBigFiles": []
///  },
///  "issues": [ "PERF_GRLE_TOO_MANY" ],
///  "l10n": {
///    "title": {
///      "en": "Eldorado"
///    },
///    "description": {
///      "en": "The Eldorado map is a Brazilian...",
///      "de": "Die Eldorado-Karte ist eine ...",
///      "br": "O mapa Eldorado é um mapa ..."
///    }
///  },
///  "md5Sum": null,
///  "modDesc": {
///    "actions": {},
///    "binds": {},
///    "author": "Case IH Brasil, Connect Modding",
///    "scriptFiles": 0,
///    "storeItems": 41,
///    "cropInfo": [
///      {
///        "name": "wheat",
///        "growthTime": 8,
///        "harvestPeriods": [],
///        "plantPeriods": [ 7, 8 ]
///      },
///    ],
///    "cropWeather": {
///      "spring": { "max": 32, "min": 16 },
///      "autumn": { "min": 14, "max": 30 },
///      "winter": { "max": 30, "min": 13
///      },
///      "summer": { "max": 31, "min": 21 }
///    },
///    "depend": [
///      "FS22_Cerca_BR"
///    ],
///    "descVersion": 79,
///    "iconFileName": "icon_eldoradoMap.dds",
///    "iconImage": "data:image/webp;base64, ...",
///    "mapConfigFile": "xml/map.xml",
///    "mapIsSouth": true,
///    "multiPlayer": true,
///    "version": "1.0.0.0"
///  },
///  "uuid": "e4d48eaebd40e7f8d160081dad9c8802"
///}
/// ```
/* cSpell: enable */
pub fn parse_to_json(full_path :&Path, is_folder: bool) -> String {
    parser(full_path, is_folder).to_string()
}


/// Test a mod file against known game limitations
/// 
/// Returns a [ModRecord]
/// 
/// captured information includes version, l10n title and description,
/// key bindings, multiplayer status, if it's a map,
/// icon, abd some simple piracy detection - see [ModRecord] for more details
/// 
/// # File Name Checks
/// 
/// - Filename must start with a letter (not a number)
/// - Filename must not contain spaces
/// - Filename can only contain A-Z, a-z, 0-9, and underscore.
/// - Non-Folders must end with a .zip extension
/// 
/// # Valid file types
/// ```
/// vec!["png", "dds", "i3d", "shapes", "lua", "gdm", "cache", "xml", "grle", "pdf", "txt", "gls", "anim", "ogg"];
/// ```
/// 
/// # Quantity Limits
/// ```
/// let mut max_grle = 10;
/// let mut max_pdf = 1;
/// let mut max_png = 128;
/// let mut max_txt = 2;
/// ```
/// 
/// # Size Limits (in bytes)
/// ```
/// let size_cache :u64  = 10_485_760;
/// let size_dds :u64    = 12_582_912;
/// let size_gdm :u64    = 18_874_368;
/// let size_shapes :u64 = 268_435_456;
/// let size_xml :u64    = 262_144;
/// ```
pub fn parser(full_path :&Path, is_folder: bool) -> ModRecord {
    let mut mod_record = ModRecord::new(full_path, is_folder);

    mod_record.can_not_use = !test_file_name(&mut mod_record);

    if mod_record.can_not_use {
        mod_record.add_issue(ModError::FileErrorNameInvalid);
        mod_record.update_badges();
        return mod_record;
    }

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder 
        {
            mod_record.add_issue(ModError::InfoNoMultiplayerUnzipped);
            match AbstractFolder::new(full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.add_issue(e);
                    mod_record.can_not_use = true;
                    mod_record.update_badges();
                    return mod_record;
                }
            }
        } else {
            match AbstractZipFile::new(full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.add_issue(e);
                    mod_record.can_not_use = true;
                    mod_record.update_badges();
                    return mod_record
                } 
            }
        };

    let abstract_file_list = abstract_file.list();

    if let Ok(meta) = std::fs::metadata(full_path) {
        if let Ok(time) = meta.created() {
            mod_record.file_detail.file_date = sys_time_to_string(time)
        }
        if ! abstract_file.is_folder() {
            mod_record.file_detail.file_size = meta.len();
        } else {
            let mut full_size:u64 = 0;
            for entry in &abstract_file_list {
                full_size += entry.size;
            }
            mod_record.file_detail.file_size = full_size;
        }
    }

    if abstract_file.exists("careerSavegame.xml") {
        mod_record.file_detail.is_save_game = true;
        mod_record.can_not_use = true;
        mod_record.add_issue(ModError::FileErrorLikelySaveGame);
        mod_record.update_badges();
        return mod_record;
    }

    if ! abstract_file.is_folder() && abstract_file_list.iter().all(|x| x.name.ends_with(".zip")) {
        mod_record.file_detail.is_mod_pack = true;
        mod_record.can_not_use = true;
        mod_record.add_issue(ModError::FileErrorLikelyZipPack);
        mod_record.update_badges();
        return mod_record;
    }

    let mod_desc_content = match abstract_file.as_text("modDesc.xml") {
        Ok(content) => content,
        Err(..) => {
            mod_record.add_issue(ModError::ModDescMissing);
            mod_record.can_not_use = true;
            mod_record.update_badges();
            return mod_record;
        },
    };

    let mod_desc_doc = match roxmltree::Document::parse(&mod_desc_content) {
        Ok(tree) => tree,
        Err(..) => {
            mod_record.add_issue(ModError::ModDescParseError);
            mod_record.can_not_use = true;
            mod_record.update_badges();
            return mod_record;
        }
    };

    do_file_counts(&mut mod_record, &abstract_file_list);
    mod_desc_basics(&mut mod_record, &mod_desc_doc);

    mod_record.mod_desc.icon_image = match &mod_record.mod_desc.icon_file_name {
        Some(filename) => convert_mod_icon(abstract_file.as_bin(filename).unwrap()),
        None => None,
    };

    if ! NOT_MALWARE.contains(&mod_record.file_detail.short_name.clone().as_str()) {
        let re_1 = RegexBuilder::new(r"\.deleteFolder")
            .multi_line(true)
            .build()
            .unwrap();
        let re_2 = RegexBuilder::new(r"\.deleteFile")
            .multi_line(true)
            .build()
            .unwrap();

        for lua_file in abstract_file_list.iter().filter(|n|n.name.ends_with(".lua")) {
            if let Ok(content) = abstract_file.as_text(&lua_file.name) {
                if re_1.is_match(content.as_str()) || re_2.is_match(content.as_str()) {
                    mod_record.add_issue(ModError::InfoMaliciousCode);
                }
            }
        }
    }

    crate::maps::read_map_basics(&mut mod_record, &mut abstract_file);

    mod_record.update_badges();
    mod_record
}

/// Test a mod file name against known game limitations
fn test_file_name(mod_record : &mut ModRecord) -> bool {
    if !mod_record.file_detail.is_folder && ! mod_record.file_detail.full_path.ends_with(".zip") {
        if mod_record.file_detail.full_path.ends_with(".rar") || mod_record.file_detail.full_path.ends_with(".7z") {
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

/// Count contained files in the mod
fn do_file_counts(mod_record : &mut ModRecord, file_list : &Vec<FileDefinition>) {
    let mut max_grle = 10;
    let mut max_pdf = 1;
    let mut max_png = 128;
    let mut max_txt = 2;

    let size_cache :u64  = 10_485_760;
    let size_dds :u64    = 12_582_912;
    let size_gdm :u64    = 18_874_368;
    let size_shapes :u64 = 268_435_456;
    let size_xml :u64    = 262_144;

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

/// Convert a system time to a ISO JSON string
fn sys_time_to_string(now: SystemTime) -> String {
    let now: DateTime<Utc> = now.into();
    now.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Load basic details from the modDesc.xml file
fn mod_desc_basics(mod_record : &mut ModRecord, mod_desc : &roxmltree::Document) {
    match mod_desc.root_element().attribute("descVersion") {
        Some(val) => mod_record.mod_desc.desc_version = val.parse().unwrap_or(0_u32),
        None => { mod_record.add_issue(ModError::ModDescVersionOldOrMissing); },
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("version")) {
        Some(node) => mod_record.mod_desc.version = node.text().unwrap_or("1.0.0.0").to_owned(),
        None => { mod_record.add_issue(ModError::ModDescNoModVersion); }
    }

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("author") && n.is_text() ) {
        mod_record.mod_desc.author = node.text().unwrap_or("--").to_owned();
    }

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("multiplayer")) {
        if let Some(val) = node.attribute("supported") {
            mod_record.mod_desc.multi_player = val.parse().unwrap_or(false)
        }
    }

    mod_record.mod_desc.store_items = mod_desc.descendants().filter(|n| n.has_tag_name("storeItem")).count() as u32;

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("map")) {
        if let Some(val) = node.attribute("configFilename") {
            mod_record.mod_desc.map_config_file = Some(val.to_owned())
        }
    }

    for depend in mod_desc.descendants().filter(|n| n.has_tag_name("dependency") && n.is_text()) {
        mod_record.mod_desc.depend.push(depend.text().unwrap_or("--").to_owned())
    }

    if mod_desc.descendants().any(|n| n.has_tag_name("productId")) {
        mod_record.add_issue(ModError::InfoLikelyPiracy); 
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("iconFilename")) {
        Some(node) => {
            match node.text() {
                Some(val) => {
                    let mut value_string = val.to_string();
                    if let Some(index) = value_string.find(".png") {
                        value_string.replace_range(index..value_string.len(), ".dds");
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
        if let Some(name) = action.attribute("name") {
            mod_record.mod_desc.actions.insert(
                name.to_owned(),
                match action.attribute("category") {
                    Some(cat) => cat.to_owned(),
                    None => "ALL".to_string(),
                }
            );
        }
    }

    for action in mod_desc.descendants().filter(|n| n.has_tag_name("actionBinding")) {
        if let Some(name) = action.attribute("action") {
            mod_record.mod_desc.binds.insert(
                name.to_owned(),
                action
                    .children()
                    .filter(|n|n.has_tag_name("binding") && n.attribute("device") == Some("KB_MOUSE_DEFAULT") && n.has_attribute("input"))
                    .map(|x| x.attribute("input").unwrap().to_string())
                    .collect()
            );
        }
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("title")) {
        Some(titles) => {
            if titles.is_text() {
                mod_record.l10n.title.insert(
                    "en".to_string(),
                    titles.text().unwrap_or("--").to_string()
                );
                mod_record.add_issue(ModError::PerformanceMissingL10N);
            } else {
                for title in titles.children().filter(|n|n.is_element()) {
                    mod_record.l10n.title.insert(
                        title.tag_name().name().to_string(),
                        title.text().unwrap_or("--").to_string()
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
                    descriptions.text().unwrap_or("").to_string()
                );
                mod_record.add_issue(ModError::PerformanceMissingL10N);
            } else {
                for description in descriptions.children().filter(|n|n.is_element()) {
                    mod_record.l10n.description.insert(
                        description.tag_name().name().to_string(),
                        description.text().unwrap_or("").to_string()
                    );
                }
            }
        },
        None => { mod_record.add_issue(ModError::PerformanceMissingL10N); },
    }
}
