//! Parser functions for basic mod reading
use crate::ModParserOptions;
use crate::shared::errors::ModError;
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile, FileDefinition};
use crate::shared::structs::{ModRecord, ZipPackFile};
use crate::shared::convert_mod_icon;
use crate::savegame::parse_open_file as savegame_parse;
use crate::mod_detail::parse_open_file as detail_parse;

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
/// Test a mod file against known game limitations
/// 
/// Returns a [`ModRecord`]
/// 
/// captured information includes version, l10n title and description,
/// key bindings, multiplayer status, if it's a map,
/// icon, abd some simple piracy detection - see [`ModRecord`] for more details
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
///      "winter": { "max": 30, "min": 13 },
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
pub fn parser<P: AsRef<Path>>(full_path :P) -> ModRecord {
    parser_with_options(full_path, &ModParserOptions::default())
}

pub fn parser_with_options<P: AsRef<Path>>(full_path :P, options : &ModParserOptions) -> ModRecord {
    let is_folder = full_path.as_ref().is_dir();
    let mut mod_record = ModRecord::new(&full_path, is_folder);

    mod_record.can_not_use = !test_file_name(&mut mod_record);

    if mod_record.can_not_use {
        mod_record.add_issue(ModError::FileErrorNameInvalid);
    }

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder 
        {
            mod_record.add_issue(ModError::InfoNoMultiplayerUnzipped);
            match AbstractFolder::new(&full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.add_fatal(e);
                    mod_record.update_badges();
                    return mod_record;
                }
            }
        } else {
            match AbstractZipFile::new(&full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.add_fatal(e);
                    mod_record.update_badges();
                    return mod_record
                } 
            }
        };

    let abstract_file_list = abstract_file.list();

    if let Ok(meta) = std::fs::metadata(full_path) {
        if let Ok(time) = meta.created() {
            mod_record.file_detail.file_date = sys_time_to_string(time);
        }
        if abstract_file.is_folder() {
            let mut full_size:u64 = 0;
            for entry in &abstract_file_list {
                full_size += entry.size;
            }
            mod_record.file_detail.file_size = full_size;
        } else {
            mod_record.file_detail.file_size = meta.len();
        }
    }

    if abstract_file.exists("careerSavegame.xml") {
        mod_record.file_detail.is_save_game = true;
        mod_record.add_fatal(ModError::FileErrorLikelySaveGame);
        mod_record.update_badges();
        if options.include_save_game {
            mod_record.include_save_game = Some(savegame_parse(abstract_file));
        }
        return mod_record;
    }

    if ! abstract_file.is_folder() {
        if let Some(list) = test_mod_pack(&abstract_file_list) {
            mod_record.file_detail.zip_files   = list;
            mod_record.file_detail.is_mod_pack = true;
            mod_record.add_fatal(ModError::FileErrorLikelyZipPack);
            mod_record.update_badges();
            return mod_record;
        }
    }

    let Ok(mod_desc_content) = abstract_file.as_text("modDesc.xml") else {
        mod_record.add_fatal(ModError::ModDescMissing);
        mod_record.update_badges();
        return mod_record;
    };

    let Ok(mod_desc_doc) = roxmltree::Document::parse(&mod_desc_content) else {
        mod_record.add_fatal(ModError::ModDescParseError);
        mod_record.update_badges();
        return mod_record;
    };

    do_file_counts(&mut mod_record, &abstract_file_list);
    mod_desc_basics(&mut mod_record, &mod_desc_doc);

    if let Some(filename) = &mod_record.mod_desc.icon_file_name {
        if let Ok(binary_file) = abstract_file.as_bin(filename) {
            mod_record.mod_desc.icon_image = convert_mod_icon(binary_file);
        }
    }

    if ! NOT_MALWARE.contains(&mod_record.file_detail.short_name.clone().as_str()) {
        let re = RegexBuilder::new(r"\.delete(File|Folder)")
            .multi_line(true)
            .build();

        if let Ok(re) = re {
            for lua_file in abstract_file_list.iter().filter(|n|n.extension == "lua" ) {
                if let Ok(content) = abstract_file.as_text(&lua_file.name) {
                    if re.is_match(content.as_str()) {
                        mod_record.add_issue(ModError::InfoMaliciousCode);
                    }
                }
            }
        }
    }

    if mod_record.mod_desc.desc_version >= 60 {
        // Map Parsing not implemented for <FS22
        crate::maps::read_map_basics(&mut mod_record, &mut abstract_file);
    }

    mod_record.update_badges();

    if options.include_mod_detail {
        mod_record.include_detail = Some(detail_parse(abstract_file, &mod_desc_doc, &abstract_file_list));
    }

    mod_record
}


fn test_mod_pack(file_list : &Vec<FileDefinition>) -> Option<Vec<ZipPackFile>> {
    let mut zip_list:Vec<ZipPackFile> = vec![];
    let mut max_non_zip_files = 2;
    let mut zip_files = false;

    for file in file_list {
        if file.is_folder { return None }

        match file.extension.as_str() {
            "xml" => return None,
            "zip" => {
                zip_files = true;
                zip_list.push(ZipPackFile{
                    name : file.name.clone(),
                    size : file.size
                });
            },
            _ => max_non_zip_files -= 1
        }
    }

    if max_non_zip_files <= 0 || ! zip_files {
        return None
    }
    
    Some(zip_list)
}
/// Test a mod file name against known game limitations
fn test_file_name(mod_record : &mut ModRecord) -> bool {
    if !mod_record.file_detail.is_folder {
        let file_path = Path::new(&mod_record.file_detail.full_path);
        let extension = match file_path.extension() {
            Some(ext) => ext.to_str().unwrap().to_owned().to_ascii_lowercase(),
            None => String::new(),
        };

        if ! extension.eq_ignore_ascii_case("zip") {
            if extension.eq_ignore_ascii_case("rar") || extension.eq_ignore_ascii_case("7z") {
                mod_record.add_issue(ModError::FileErrorUnsupportedArchive);
            } else {
                mod_record.add_issue(ModError::FileErrorGarbageFile);
            }
            return false
        }
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
    let mut max_pdf  = 1;
    let mut max_png  = 128;
    let mut max_txt  = 2;

    let size_cache :u64  = 10_485_760;
    let size_dds :u64    = 12_582_912;
    let size_gdm :u64    = 18_874_368;
    let size_shapes :u64 = 268_435_456;
    let size_xml :u64    = 262_144;

    let known_good = vec!["png", "dds", "i3d", "shapes", "lua", "gdm", "cache", "xml", "grle", "pdf", "txt", "gls", "anim", "ogg"];

    for file in file_list {
        if file.is_folder { continue }

        if known_good.contains(&file.extension.as_str()) {
            if file.name.contains(' ') {
                mod_record.add_issue(ModError::PerformanceFileSpaces);
                mod_record.file_detail.space_files.push(file.name.clone());
            }
            match file.extension.as_str() {
                "lua"  => mod_record.mod_desc.script_files += 1,
                "png"  => {
                    if ! file.name.ends_with("_weight.png") {
                        mod_record.file_detail.image_non_dds.push(file.name.clone());
                        mod_record.file_detail.png_texture.push(file.name.clone());
                    }
                    max_png -= 1;
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

        } else {
            if file.extension == "dat" || file.extension == "l64" {
                mod_record.add_issue(ModError::InfoLikelyPiracy);
            }
            mod_record.add_issue(ModError::PerformanceQuantityExtra);
            mod_record.file_detail.extra_files.push(file.name.clone());
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
        Some(node) =>  node.text().unwrap_or("1.0.0.0").clone_into(&mut mod_record.mod_desc.version),
        None => { mod_record.add_issue(ModError::ModDescNoModVersion); }
    }

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("author")) {
        node.text().unwrap_or("--").clone_into(&mut mod_record.mod_desc.author);
    }

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("multiplayer")) {
        if let Some(val) = node.attribute("supported") {
            mod_record.mod_desc.multi_player = val.parse().unwrap_or(false);
        }
    }

    mod_record.mod_desc.store_items = mod_desc.descendants().filter(|n| n.has_tag_name("storeItem")).count();

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("map")) {
        if let Some(val) = node.attribute("configFilename") {
            mod_record.mod_desc.map_config_file = Some(val.to_owned());
        }
    }

    for depend in mod_desc.descendants().filter(|n| n.has_tag_name("dependency") && n.is_text()) {
        mod_record.mod_desc.depend.push(depend.text().unwrap_or("--").to_owned());
    }

    if mod_desc.descendants().any(|n| n.has_tag_name("productId")) {
        mod_record.add_issue(ModError::InfoLikelyPiracy); 
    }

    mod_record.mod_desc.icon_file_name = read_mod_filename(mod_desc.descendants().find(|n| n.has_tag_name("iconFilename")), mod_record);

    if mod_record.mod_desc.icon_file_name.is_none() {
        mod_record.add_issue(ModError::ModDescNoModIcon);
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
                for title in titles.children().filter(roxmltree::Node::is_element) {
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
                for description in descriptions.children().filter(roxmltree::Node::is_element) {
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

fn read_mod_filename(node : Option<roxmltree::Node>, mod_record : &mut ModRecord) -> Option<String> {
    match node {
        Some(node) => {
            match node.text() {
                Some(val) => {
                    let mut value_string = val.to_string();
                    if let Some(index) = value_string.find(".png") {
                        value_string.replace_range(index..value_string.len(), ".dds");
                    }
                    if mod_record.file_detail.image_dds.contains(&value_string) {
                        Some(value_string)
                    } else {
                        None
                    }
                },
                None => None
            }
        },
        None => None
    }
}