//! Parser functions for basic mod reading
use crate::maps::read_map_basics;
use crate::mod_detail::parse_open_file as detail_parse;
use crate::savegame::parse_open_file as savegame_parse;
use crate::shared::errors::ModError;
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile, FileDefinition};
use crate::shared::structs::{ModRecord, ZipPackFile};
use crate::shared::{convert_mod_icon, extract_and_normalize_image, ImageFile};
use crate::ModParserOptions;

use chrono::{DateTime, SecondsFormat, Utc};
use std::{path::Path, time::SystemTime};

/// Known false positives for the malware check
pub const NOT_MALWARE: [&str; 11] = [
    "FS22_001_NoDelete",
    "FS22_AutoDrive",
    "FS22_Courseplay",
    "FS22_FSG_Companion",
    "FS22_VehicleControlAddon",
    "MultiOverlayV3",   // Happylooser
    "MultiOverlayV4",   // Happylooser
    "VehicleInspector", // Happylooser
    "FS19_AutoDrive",
    "FS19_Courseplay",
    "FS19_GlobalCompany",
];

/// one megabyte
const MB: u64 = 0x0010_0000;
/// max size allowed for I3D Cache files, 10MB
const SIZE_CACHE: u64 = 10 * MB;
/// max size allowed for DDS files, 12MB
const SIZE_DDS: u64 = 12 * MB;
/// max size allowed for GDM files, 18 MB
const SIZE_GDM: u64 = 18 * MB;
/// max size allowed for SHAPES files, 256MB
const SIZE_SHAPES: u64 = 256 * MB;
/// max size allowed for XML files, 256KB / 0.25MB
const SIZE_XML: u64 = MB / 4;

/// max allowed GRLE files
const MAX_GRLE: u32 = 10;
/// max allowed PDF files
const MAX_PDF: u32 = 1;
/// max allowed PNG files
const MAX_PNG: u32 = 128;
/// max allowed TXT files
const MAX_TXT: u32 = 2;

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
/// /// one megabyte
/// const MB:u64          = 0x0010_0000;
/// /// max size allowed for I3D Cache files, 10MB
/// const SIZE_CACHE:u64  = 10 * MB;
/// /// max size allowed for DDS files, 12MB
/// const SIZE_DDS: u64   = 12 * MB;
/// /// max size allowed for GDM files, 18 MB
/// const SIZE_GDM:u64    = 18 * MB;
/// /// max size allowed for SHAPES files, 256MB
/// const SIZE_SHAPES:u64 = 256 * MB;
/// /// max size allowed for XML files, 256KB / 0.25MB
/// const SIZE_XML:u64    = MB / 4;
/// ```
///
/// # Size Limits (in bytes)
/// ```
/// /// max allowed GRLE files
/// const MAX_GRLE:u32 = 10;
/// /// max allowed PDF files
/// const MAX_PDF:u32  = 1;
/// /// max allowed PNG files
/// const MAX_PNG:u32  = 128;
/// /// max allowed TXT files
/// const MAX_TXT:u32  = 2;
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
pub fn parser<P: AsRef<Path>>(full_path: P) -> ModRecord {
    parser_with_options(full_path, &ModParserOptions::default())
}

/// [`crate::mod_basic::parser`] with options
pub fn parser_with_options<P: AsRef<Path>>(full_path: P, options: &ModParserOptions) -> ModRecord {
    let is_folder = full_path.as_ref().is_dir();
    let mut mod_record = ModRecord::new(&full_path, is_folder);

    if !check_file_name(&mut mod_record) {
        mod_record.can_not_use = true;
        mod_record.add_issue(ModError::FileErrorNameInvalid);
    }

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder {
        mod_record.add_issue(ModError::InfoNoMultiplayerUnzipped);
        match AbstractFolder::new(&full_path) {
            Ok(archive) => Box::new(archive),
            Err(e) => {
                mod_record.add_fatal(e).update_badges();
                return mod_record;
            }
        }
    } else {
        match AbstractZipFile::new(&full_path) {
            Ok(archive) => Box::new(archive),
            Err(e) => {
                mod_record.add_fatal(e).update_badges();
                return mod_record;
            }
        }
    };

    let abstract_file_list = abstract_file.list();

    if let Ok(meta) = std::fs::metadata(full_path) {
        mod_record.file_detail.file_date = sys_time_to_string(meta.created().ok());

        if abstract_file.is_folder() {
            mod_record.file_detail.file_size =
                abstract_file_list.clone().iter().map(|n| n.size).sum();
        } else {
            mod_record.file_detail.file_size = meta.len();
        }
    }

    if abstract_file.exists("careerSavegame.xml") {
        mod_record.file_detail.is_save_game = true;
        mod_record
            .add_fatal(ModError::FileErrorLikelySaveGame)
            .update_badges();
        if options.include_save_game {
            mod_record.include_save_game = Some(savegame_parse(abstract_file));
        }
        return mod_record;
    }

    if !abstract_file.is_folder() {
        if let Some(list) = check_mod_pack(&abstract_file_list) {
            mod_record.file_detail.zip_files = list;
            mod_record.file_detail.is_mod_pack = true;
            mod_record
                .add_fatal(ModError::FileErrorLikelyZipPack)
                .update_badges();
            return mod_record;
        }
    }

    let Ok(mod_desc_content) = abstract_file.as_text("modDesc.xml") else {
        mod_record
            .add_fatal(ModError::ModDescMissing)
            .update_badges();
        return mod_record;
    };

    let Ok(mod_desc_doc) = roxmltree::Document::parse(&mod_desc_content) else {
        mod_record
            .add_fatal(ModError::ModDescParseError)
            .update_badges();
        return mod_record;
    };

    do_file_counts(&mut mod_record, &abstract_file_list);
    mod_desc_basics(&mut mod_record, &mod_desc_doc);

    if !options.skip_mod_icons {
        if let Some(filename) = &mod_record.mod_desc.icon_file_name {
            if let Ok(binary_file) = abstract_file.as_bin(filename) {
                mod_record.mod_desc.icon_image = convert_mod_icon(binary_file);
            } else {
                mod_record.add_issue(ModError::ModDescNoModIcon);
            }
        }
    }

    if check_lua(
        &mod_record.file_detail.short_name,
        &mut abstract_file,
        &abstract_file_list,
    ) {
        mod_record.add_issue(ModError::InfoMaliciousCode);
    }

    // Map Parsing not implemented for <FS22
    read_map_basics(
        mod_record.mod_desc.desc_version,
        &mut mod_record,
        &mut abstract_file,
    );

    mod_record.update_badges();

    if options.include_mod_detail {
        mod_record.detail_icon_loaded = !options.skip_detail_icons;
        mod_record.include_detail = Some(detail_parse(
            abstract_file,
            &mod_desc_doc,
            &abstract_file_list,
            options,
        ));
    }

    mod_record
}

/// Check LUA files for malware
fn check_lua(
    short_name: &String,
    file_handle: &mut Box<dyn AbstractFileHandle>,
    file_list: &[FileDefinition],
) -> bool {
    if NOT_MALWARE.iter().any(|&s| s == short_name) {
        return false;
    }

    for lua_file in file_list.iter().filter(|n| n.extension == "lua") {
        if let Ok(content) = file_handle.as_text(&lua_file.name) {
            if content.contains(".deleteFolder") || content.contains(".deleteFile") {
                return true;
            }
        }
    }
    false
}
/// Check if mod is actually a mod pack
fn check_mod_pack(file_list: &Vec<FileDefinition>) -> Option<Vec<ZipPackFile>> {
    let mut zip_list: Vec<ZipPackFile> = vec![];
    let mut max_non_zip_files = 2;
    let mut zip_files = false;

    for file in file_list {
        if file.is_folder {
            return None;
        }

        match file.extension.as_str() {
            "xml" => return None,
            "zip" => {
                zip_files = true;
                zip_list.push(ZipPackFile {
                    name: file.name.clone(),
                    size: file.size,
                });
            }
            _ => max_non_zip_files -= 1,
        }
    }

    if max_non_zip_files <= 0 || !zip_files {
        return None;
    }

    Some(zip_list)
}

#[test]
fn test_file_name_assumptions() {
    assert!(check_file_name(&mut ModRecord::new("Example.zip", false)));
    assert!(check_file_name(&mut ModRecord::new(
        "ExampleUNZIP.zip",
        false
    )));

    // digit start
    assert!(!check_file_name(&mut ModRecord::new("1Example.zip", false)));
    // space
    assert!(!check_file_name(&mut ModRecord::new(
        "Hello There.zip",
        false
    )));
    // dash
    assert!(!check_file_name(&mut ModRecord::new("Howdy-Ho.zip", false)));
    // invalid extensions
    assert!(!check_file_name(&mut ModRecord::new("GoodName.7z", false)));
    assert!(!check_file_name(&mut ModRecord::new("GoodName.rar", false)));
    assert!(!check_file_name(&mut ModRecord::new("GoodName.txt", false)));
}
/// Test a mod file name against known game limitations
fn check_file_name(mod_record: &mut ModRecord) -> bool {
    if !mod_record.file_detail.is_folder {
        let file_path = Path::new(&mod_record.file_detail.full_path);
        let extension = match file_path.extension() {
            Some(ext) => ext.to_str().unwrap_or("").to_owned().to_ascii_lowercase(),
            None => String::new(),
        };

        if !extension.eq_ignore_ascii_case("zip") {
            if extension.eq_ignore_ascii_case("rar") || extension.eq_ignore_ascii_case("7z") {
                mod_record.add_issue(ModError::FileErrorUnsupportedArchive);
            } else {
                mod_record.add_issue(ModError::FileErrorGarbageFile);
            }
            return false;
        }
    }

    if mod_record
        .file_detail
        .short_name
        .to_ascii_lowercase()
        .contains("unzip")
    {
        mod_record.add_issue(ModError::FileErrorLikelyZipPack);
    }

    if mod_record
        .file_detail
        .short_name
        .chars()
        .next()
        .map_or(true, |c| c.is_ascii_digit())
    {
        mod_record.add_issue(ModError::FileErrorNameStartsDigit);
        return false;
    }

    if !&mod_record
        .file_detail
        .short_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        let copy_name: Vec<&str> = mod_record
            .file_detail
            .short_name
            .split_inclusive(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .map(str::trim)
            .collect();
        if copy_name[0]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            mod_record.issues.insert(ModError::FileErrorLikelyCopy);
            mod_record.file_detail.copy_name = Some(copy_name[0].to_owned());
        }
        return false;
    }

    true
}

/// Count contained files in the mod
fn do_file_counts(mod_record: &mut ModRecord, file_list: &Vec<FileDefinition>) {
    let mut found_grle: u32 = 0;
    let mut found_pdf: u32 = 0;
    let mut found_png: u32 = 0;
    let mut found_txt: u32 = 0;

    let known_good = vec![
        "png", "dds", "i3d", "shapes", "lua", "gdm", "cache", "xml", "grle", "pdf", "txt", "gls",
        "anim", "ogg",
    ];

    for file in file_list {
        if file.is_folder {
            continue;
        }

        if known_good.contains(&file.extension.as_str()) {
            if file.name.contains(' ') {
                mod_record.add_issue(ModError::PerformanceFileSpaces);
                mod_record.file_detail.space_files.push(file.name.clone());
            }
            match file.extension.as_str() {
                "lua" => mod_record.mod_desc.script_files += 1,
                "png" => {
                    if !file.name.ends_with("_weight.png") {
                        mod_record.file_detail.image_non_dds.push(file.name.clone());
                        mod_record.file_detail.png_texture.push(file.name.clone());
                    }
                    found_png += 1;
                }
                "pdf" => found_pdf += 1,
                "grle" => found_grle += 1,
                "txt" => found_txt += 1,
                "cache" => {
                    if file.size > SIZE_CACHE {
                        mod_record.add_issue(ModError::PerformanceOversizeI3D);
                        mod_record.file_detail.too_big_files.push(file.name.clone());
                    }
                }
                "dds" => {
                    mod_record.file_detail.image_dds.push(file.name.clone());
                    if file.size > SIZE_DDS {
                        mod_record.add_issue(ModError::PerformanceOversizeDDS);
                        mod_record.file_detail.too_big_files.push(file.name.clone());
                    }
                }
                "gdm" => {
                    if file.size > SIZE_GDM {
                        mod_record.add_issue(ModError::PerformanceOversizeGDM);
                        mod_record.file_detail.too_big_files.push(file.name.clone());
                    }
                }
                "shapes" => {
                    if file.size > SIZE_SHAPES {
                        mod_record.add_issue(ModError::PerformanceOversizeSHAPES);
                        mod_record.file_detail.too_big_files.push(file.name.clone());
                    }
                }
                "xml" => {
                    if file.size > SIZE_XML {
                        mod_record.add_issue(ModError::PerformanceOversizeXML);
                        mod_record.file_detail.too_big_files.push(file.name.clone());
                    }
                }
                _ => {}
            }

            if found_grle > MAX_GRLE {
                mod_record.add_issue(ModError::PerformanceQuantityGRLE);
            }
            if found_pdf > MAX_PDF {
                mod_record.add_issue(ModError::PerformanceQuantityPDF);
            }
            if found_png > MAX_PNG {
                mod_record.add_issue(ModError::PerformanceQuantityPNG);
            }
            if found_txt > MAX_TXT {
                mod_record.add_issue(ModError::PerformanceQuantityTXT);
            }
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
fn sys_time_to_string(now: Option<SystemTime>) -> String {
    match now {
        Some(now) => {
            let now: DateTime<Utc> = now.into();
            now.to_rfc3339_opts(SecondsFormat::Secs, true)
        }
        None => String::from("1970-01-01T00:00:00Z"),
    }
}

/// Load basic details from the modDesc.xml file
fn mod_desc_basics(mod_record: &mut ModRecord, mod_desc: &roxmltree::Document) {
    match mod_desc.root_element().attribute("descVersion") {
        Some(val) => mod_record.mod_desc.desc_version = val.parse().unwrap_or(0_u32),
        None => {
            mod_record.add_issue(ModError::ModDescVersionOldOrMissing);
        }
    }

    match mod_desc.descendants().find(|n| n.has_tag_name("version")) {
        Some(node) => node
            .text()
            .unwrap_or("1.0.0.0")
            .clone_into(&mut mod_record.mod_desc.version),
        None => {
            mod_record.add_issue(ModError::ModDescNoModVersion);
        }
    }

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("author")) {
        node.text()
            .unwrap_or("--")
            .clone_into(&mut mod_record.mod_desc.author);
    }

    if let Some(node) = mod_desc
        .descendants()
        .find(|n| n.has_tag_name("multiplayer"))
    {
        if let Some(val) = node.attribute("supported") {
            mod_record.mod_desc.multi_player = val.parse().unwrap_or(false);
        }
    }

    mod_record.mod_desc.store_items = mod_desc
        .descendants()
        .filter(|n| n.has_tag_name("storeItem"))
        .count();

    if let Some(node) = mod_desc.descendants().find(|n| n.has_tag_name("map")) {
        if let Some(val) = node.attribute("configFilename") {
            mod_record.mod_desc.map_config_file = Some(val.to_owned());
        }
    }

    for depend in mod_desc
        .descendants()
        .filter(|n| n.has_tag_name("dependency") && n.is_text())
    {
        mod_record
            .mod_desc
            .depend
            .push(depend.text().unwrap_or("--").to_owned());
    }

    if mod_desc.descendants().any(|n| n.has_tag_name("productId")) {
        mod_record.add_issue(ModError::InfoLikelyPiracy);
    }

    match extract_and_normalize_image(mod_desc, "iconFilename") {
        ImageFile {
            local_file: Some(local_file),
            ..
        } => {
            mod_record.mod_desc.icon_file_name = Some(local_file);
        }
        ImageFile { .. } => {
            mod_record.add_issue(ModError::ModDescNoModIcon);
        }
    }

    mod_desc_actions(mod_record, mod_desc);
    mod_desc_l10n(mod_record, mod_desc);
}

/// Parse title and description entries
fn mod_desc_l10n(mod_record: &mut ModRecord, mod_desc: &roxmltree::Document) {
    match mod_desc.descendants().find(|n| n.has_tag_name("title")) {
        Some(titles) => {
            if titles.is_text() {
                mod_record
                    .l10n
                    .title
                    .insert(String::from("en"), titles.text().unwrap_or("--").to_owned());
                mod_record.add_issue(ModError::PerformanceMissingL10N);
            } else {
                for title in titles.children().filter(roxmltree::Node::is_element) {
                    mod_record.l10n.title.insert(
                        title.tag_name().name().to_owned(),
                        title.text().unwrap_or("--").to_owned(),
                    );
                }
            }
        }
        None => {
            mod_record.add_issue(ModError::PerformanceMissingL10N);
        }
    }

    match mod_desc
        .descendants()
        .find(|n| n.has_tag_name("description"))
    {
        Some(descriptions) => {
            if descriptions.is_text() {
                mod_record.l10n.description.insert(
                    String::from("en"),
                    descriptions.text().unwrap_or("").to_owned(),
                );
                mod_record.add_issue(ModError::PerformanceMissingL10N);
            } else {
                for description in descriptions.children().filter(roxmltree::Node::is_element) {
                    mod_record.l10n.description.insert(
                        description.tag_name().name().to_owned(),
                        description.text().unwrap_or("").to_owned(),
                    );
                }
            }
        }
        None => {
            mod_record.add_issue(ModError::PerformanceMissingL10N);
        }
    }
}

/// Parse actions and key binds in the mod
fn mod_desc_actions(mod_record: &mut ModRecord, mod_desc: &roxmltree::Document) {
    for action in mod_desc.descendants().filter(|n| n.has_tag_name("action")) {
        if let Some(name) = action.attribute("name") {
            mod_record.mod_desc.actions.insert(
                name.to_owned(),
                match action.attribute("category") {
                    Some(cat) => cat.to_owned(),
                    None => String::from("ALL"),
                },
            );
        }
    }

    for action in mod_desc
        .descendants()
        .filter(|n| n.has_tag_name("actionBinding"))
    {
        if let Some(name) = action.attribute("action") {
            mod_record.mod_desc.binds.insert(
                name.to_owned(),
                action
                    .children()
                    .filter(|n| {
                        n.has_tag_name("binding")
                            && n.attribute("device") == Some("KB_MOUSE_DEFAULT")
                            && n.has_attribute("input")
                    })
                    .filter_map(|x| x.attribute("input").map(std::borrow::ToOwned::to_owned))
                    .collect(),
            );
        }
    }
}
