//! Main mod parser
use std::collections::{HashMap, HashSet};
use std::path::{self, Path, PathBuf};
use md5::{Md5, Digest};
use base64ct::{Base64UrlUnpadded, Encoding};
use std::io::{Read, Seek, SeekFrom};
use std::time::SystemTime;

use crate::files::{AbstractFile, XMLReader};
use crate::files::mod_desc::DescXML;
use crate::files::store_item::StoreItem;
use crate::files::l10n::L10n;
use crate::files::map22::Map22;
use crate::savegame::SaveGame;


use crate::{ParseOption, ParseOptions};
use crate::errors::{AbstractFileError, ModError, ModDescWarnings};
use crate::errors::{BADGE_BROKEN, BADGE_NOT_MOD, BADGE_ISSUE, BADGE_PERF};

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


/// Parse a mod file
///
/// Returns a [`Record`]
///
/// captured information includes version, l10n title and description,
/// key bindings, multiplayer status, if it's a map,
/// icon, abd some simple piracy detection.
///
pub fn parse<P: AsRef<Path>>(filename: P) -> Record {
    parse_with_options(filename, &ParseOptions::default())
}

/// Parse a mod file with defined options
pub fn parse_with_options<P: AsRef<Path>>(filename: P, options : &ParseOptions) -> Record {
    let mut record = Record::from_filename(filename, options);
    record.update_badges();
    record
}

/// Parse a mod file
/// 
/// # Errors
/// will return an error if either the mod or the indicated store item does not exist
pub fn parse_detail<P: AsRef<Path>, S: AsRef<str>>(filename: P, needle : S) -> Result<StoreItem, AbstractFileError> {
    parse_detail_with_options(filename, needle, &ParseOptions::default())
}

/// Parse a mod file with defined options
/// 
/// # Errors
/// will return an error if either the mod or the indicated store item does not exist
pub fn parse_detail_with_options<P: AsRef<Path>, S: AsRef<str>>(filename: P, needle: S, options : &ParseOptions) -> Result<StoreItem, AbstractFileError> {
    let mut file = AbstractFile::new(filename);
    let mut item = StoreItem::from_abstract_file(&mut file, needle)?;

    if options.contains(&ParseOption::ImageDetail) {
        if let Some(vehicle) = &mut item.vehicle {
            if let Some(filename) = &vehicle.icon_file {
                vehicle.icon_data = file.mod_icon(filename);
            }
        } else if let Some(placable) = &mut item.placeable {
            if let Some(filename) = &placable.icon_file {
                placable.icon_data = file.mod_icon(filename);
            }
        }
    }

    Ok(item)
}


/// Mod file record
/// 
/// Note on equality - this will be equal if all of the details match,
/// ignoring the path to the file, the age hash, file date, and the ident (generated)
/// from the path
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// Full path to file
    pub file: FileInfo,
    /// Mod ident from full path and filename (MD5)
    pub ident: String,
    /// Mod badges
    pub badge_array: Vec<Badges>,
    /// Mod not usable flag
    pub can_not_use: bool,
    /// Current collection for mod (not set)
    pub current_collection: String,
    /// Errors or issues found
    pub issues: HashSet<ModError>,
    /// storeItems found (if processed)
    pub include_detail: HashMap<String, StoreItem>,
    /// storeItem icons loaded
    pub detail_icons_loaded : bool,
    /// save game record (if processed)
    pub include_save_game: Option<SaveGame>,
    /// L10N data
    pub l10n: L10n,
    /// modDesc.xml fields
    pub mod_desc: DescXML,
    /// Map22 fields
    pub map22 : Option<Map22>,
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file && 
        self.badge_array == other.badge_array && 
        self.can_not_use == other.can_not_use && 
        self.current_collection == other.current_collection && 
        self.issues == other.issues && 
        self.include_detail == other.include_detail && 
        self.detail_icons_loaded == other.detail_icons_loaded && 
        self.include_save_game == other.include_save_game && 
        self.l10n == other.l10n && 
        self.mod_desc == other.mod_desc
    }
}

/// mod file identity
pub struct ModIdent {
    /// full path
    full_path : PathBuf,
    /// identity hash (full path -> MD5)
    ident : String,
    /// age hash (name, size, and last 2k of file -> MD5)
    hash : String,
}

impl Record {
    /// make a new record
    fn new(mod_ident: ModIdent) -> Self {
        Self {
            ident : mod_ident.ident,
            file : FileInfo::new(mod_ident.full_path, mod_ident.hash),
            ..Default::default()
        }
    }

    /// Get a mod ident
    pub fn get_ident<P: AsRef<Path>>(filename: P) -> ModIdent {
        let full_path = path::absolute(&filename).unwrap_or_else(|_| filename.as_ref().to_path_buf());

        let mut ident = Md5::new();
        ident.update(full_path.to_string_lossy().as_ref());

        ModIdent {
            ident : Base64UrlUnpadded::encode_string(&ident.finalize()),
            hash : FileInfo::make_hash(&full_path),
            full_path,
        }
    }

    /// Create record from filename
    #[expect(clippy::too_many_lines)]
    pub fn from_filename<P: AsRef<Path>>(filename: P, options : &ParseOptions) -> Self {
        let mod_ident = Self::get_ident(filename);

        // TODO: allow cache checking here? or allow passing mod_ident?
        // expectation hash == other.hash and ident == other.ident but
        // hash != DEFAULT_HASH

        let mut record = Self::new(mod_ident);
    
        record.check_name();
    
        let mut file = AbstractFile::new(&record.file.full_path);
    
        match file {
            AbstractFile::Null(AbstractFileError::ZipReadError) => {
                record.fail(ModError::FileErrorUnreadableZip);
                return record
            },
            AbstractFile::Null(_) => {
                record.fail(ModError::FileErrorUnreadable);
                return record
            },
            AbstractFile::Folder(_, _) => {
                record.warn(ModError::InfoNoMultiplayerUnzipped);
            }
            AbstractFile::Zip(_, _) => (),
        }
    
        if let Ok(meta) = std::fs::metadata(&record.file.full_path) {
            if let Ok(date) = meta.created() {
                record.file.file_date = date.duration_since(SystemTime::UNIX_EPOCH).map(|v|v.as_secs()).unwrap_or_default();
            }
        }
    
        record.file.file_size = file.size();
    
        if file.is_in_list("careerSavegame.xml") {
            record.file.is_save_game = true;
            record.fail(ModError::FileErrorLikelySaveGame);

            if options.contains(&ParseOption::IncludeSaveGame) {
                record.include_save_game = Some(SaveGame::from_abstract(&mut file));
            }
            return record
        }
    
        if !record.file.is_folder && record.check_mod_pack(&file) {
            return record
        }
    
        match DescXML::from_abstract(&mut file) {
            Ok(mod_desc) => {
                record.mod_desc = mod_desc;
            },
            Err(AbstractFileError::XmlParseError | AbstractFileError::XmlUndeclared | AbstractFileError::XmlWrongFileType)  => {
                record.fail(ModError::ModDescParseError);
                return record
            },
            Err(_) => {
                record.fail(ModError::ModDescMissing);
                return record
            }
        }
    
        record.do_file_counts(&file);
    
        if record.mod_desc.desc_version == 0 { record.warn(ModError::ModDescVersionOldOrMissing) }
        if record.mod_desc.version.is_none() { record.warn(ModError::ModDescNoModVersion) }
        if record.mod_desc.icon_file.is_none() { record.warn(ModError::ModDescNoModIcon) }

        for item in record.mod_desc.warnings.clone() {
            match item {
                ModDescWarnings::L10nMalformed() | ModDescWarnings::L10nInvalidLanguage(_, _) | ModDescWarnings::ShouldBeL10n(_) => 
                    record.warn(ModError::PerformanceMissingL10n),
                ModDescWarnings::MaybePiracy() => record.warn(ModError::InfoLikelyPiracy),
                _ => ()
            }
        }

        record.check_lua(&mut file);

        if options.contains(&ParseOption::IncludeMap) && record.mod_desc.game_version == 22 {
            if let Some(map_config) = &record.mod_desc.map_config_filename {
                record.map22 = Map22::from_abstract_file(&mut file, map_config);
                if options.contains(&ParseOption::ImageMap) {
                    if let Some(map_record) = &mut record.map22 {
                        if let Some(map_image) = &map_record.config.image_file {
                            map_record.config.image_data = file.map_image(map_image);
                        }
                    }
                }
            }
        }

        if options.contains(&ParseOption::ImageMod) {
            if let Some(filename) = &record.mod_desc.icon_file {
                record.mod_desc.icon_data = file.mod_icon(filename);

                if record.mod_desc.icon_data.is_none() {
                    record.warn(ModError::ModDescNoModIcon);
                }
            }
        }

        if options.contains(&ParseOption::ImageDetail) {
            for item in &mut record.mod_desc.brands {
                if let Some(filename) = &item.icon_file {
                    item.icon_data = file.mod_icon(filename);
                }
            }
        }

        if options.contains(&ParseOption::IncludeDetail) {
            if let Some(folder) = record.mod_desc.l10n_file_prefix.clone() {
                record.l10n = L10n::from_abstract_folder(&mut file, folder);
            }
            for item in record.mod_desc.store_items.clone() {
                if let Ok(mut item_record) = StoreItem::from_abstract_file(&mut file, item.clone()) {
                    if options.contains(&ParseOption::ImageDetail) {
                        // TODO: also load brand icons!
                        record.detail_icons_loaded = true;
                        if let Some(vehicle) = &mut item_record.vehicle {
                            if let Some(filename) = &vehicle.icon_file {
                                vehicle.icon_data = file.mod_icon(filename);
                            }
                        } else if let Some(placable) = &mut item_record.placeable {
                            if let Some(filename) = &placable.icon_file {
                                placable.icon_data = file.mod_icon(filename);
                            }
                        }

                        
                    }
                    record.include_detail.insert(item, item_record);
                }
            }
        }

        for (key,lang_map) in record.mod_desc.l10n_local.clone() {
            for ( k, v ) in lang_map {
                let lang_entry = record.l10n.0.entry(k).or_default();
                lang_entry.insert(key.clone(), v);
            }
        }
        record.mod_desc.l10n_local.clear();

        record
    }

    /// add a warning issue
    fn warn(&mut self, e : ModError) { self.issues.insert(e); }
    /// set mod has failed
    fn fail(&mut self, e : ModError) { self.issues.insert(e); self.can_not_use = true; }

    /// Check file name
    fn check_name(&mut self) {
        if !self.file.is_folder {
            if let Some(ext) = Path::new(&self.file.full_path).extension() {
                match ext.to_ascii_lowercase().as_encoded_bytes() {
                    b"zip" => (),
                    b"rar" | b"7z" => {
                        self.fail(ModError::FileErrorUnsupportedArchive);
                        self.fail(ModError::FileErrorNameInvalid);
                    },
                    _ => {
                        self.fail(ModError::FileErrorGarbageFile);
                        self.fail(ModError::FileErrorNameInvalid);
                    }
                }
            }
        }

        if self.file.short_name.to_ascii_lowercase().contains("unzip") {
            self.warn(ModError::FileErrorLikelyZipPack);
        }

        if self.file.short_name.starts_with(|c: char| c.is_ascii_digit()) {
            self.fail(ModError::FileErrorNameStartsDigit);
            self.fail(ModError::FileErrorNameInvalid);
        }

        if !self.file.short_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' ) {
            let copy_name: Vec<&str> = self.file.short_name
                .split_inclusive(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .map(str::trim)
                .collect();

            if copy_name[0].chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
            {
                self.issues.insert(ModError::FileErrorLikelyCopy);
                self.file.copy_name = Some(copy_name[0].to_owned());
            }
            
            self.fail(ModError::FileErrorNameInvalid);
        }
    }

    /// Check if this a pack of mods, not a single mod
    fn check_mod_pack(&mut self, file : &AbstractFile) -> bool {
        let mut zip_list: Vec<ZipPackFile> = vec![];
        let mut max_non_zip_files = 2;
        let mut zip_files = false;
        
        for file in file.list() {
            if file.is_dir { return false }
        
            match file.extension.as_str() {
                "xml" => return false,
                "zip" => {
                    zip_files = true;
                    zip_list.push(ZipPackFile {
                        name: file.path,
                        size: file.size,
                    });
                }
                _ if max_non_zip_files < 1 => return false,
                _ => max_non_zip_files -= 1,
            }
        }
        
        if max_non_zip_files > 0 && zip_files {
            self.file.is_mod_pack = true;
            self.file.zip_files   = zip_list;
            self.fail(ModError::FileErrorLikelyZipPack);
            return true;
        }
        false
    }

    /// Count mod files
    fn do_file_counts(&mut self, file : &AbstractFile) {
        let mut found_grle: u32 = 0;
        let mut found_pdf: u32 = 0;
        let mut found_png: u32 = 0;
        let mut found_txt: u32 = 0;
        
        let known_good = vec![
            "png", "dds", "i3d", "shapes", "lua",
            "gdm", "cache", "xml", "grle", "pdf",
            "txt", "gls", "anim", "ogg",
        ];
        
        for file in file.list() {
            if file.is_dir { continue }
        
            if known_good.contains(&file.extension.as_str()) {
                if file.path.contains(' ') {
                    self.warn(ModError::PerformanceFileSpaces);
                    self.file.space_files.push(file.path.clone());
                }
                match file.extension.as_str() {
                    "lua" => self.file.lua_count += 1,
                    "png" => {
                        if !file.path.ends_with("_weight.png") {
                            self.file.image_non_dds.push(file.path.clone());
                            self.file.png_texture.push(file.path);
                        }
                        found_png += 1;
                    }
                    "pdf" => found_pdf += 1,
                    "grle" => found_grle += 1,
                    "txt" => found_txt += 1,
                    "cache" => {
                        if file.size > SIZE_CACHE {
                            self.warn(ModError::PerformanceOversizeI3D);
                            self.file.too_big_files.push(file.path);
                        }
                    }
                    "dds" => {
                        self.file.image_dds.push(file.path.clone());
                        if file.size > SIZE_DDS {
                            self.warn(ModError::PerformanceOversizeDDS);
                            self.file.too_big_files.push(file.path);
                        }
                    }
                    "gdm" => {
                        if file.size > SIZE_GDM {
                            self.warn(ModError::PerformanceOversizeGDM);
                            self.file.too_big_files.push(file.path);
                        }
                    }
                    "shapes" => {
                        if file.size > SIZE_SHAPES {
                            self.warn(ModError::PerformanceOversizeSHAPES);
                            self.file.too_big_files.push(file.path);
                        }
                    }
                    "xml" => {
                        if file.size > SIZE_XML {
                            self.warn(ModError::PerformanceOversizeXML);
                            self.file.too_big_files.push(file.path);
                        }
                    }
                    _ => {}
                }
        
                if found_grle > MAX_GRLE {
                    self.warn(ModError::PerformanceQuantityGRLE);
                }
                if found_pdf > MAX_PDF {
                    self.warn(ModError::PerformanceQuantityPDF);
                }
                if found_png > MAX_PNG {
                    self.warn(ModError::PerformanceQuantityPNG);
                }
                if found_txt > MAX_TXT {
                    self.warn(ModError::PerformanceQuantityTXT);
                }
            } else {
                if file.extension == "dat" || file.extension == "l64" {
                    self.warn(ModError::InfoLikelyPiracy);
                }
                if file.extension == "exe" || file.extension == "bat" || file.extension == "ps1" {
                    self.fail(ModError::InfoDangerousFile);
                }
                self.warn(ModError::PerformanceQuantityExtra);
                self.file.extra_files.push(file.path);
            }
        }
    }

    /// Check for malicious LUA files
    fn check_lua(&mut self, file : &mut AbstractFile) {
        if crate::NOT_MALWARE.contains(&self.file.short_name.as_str()) { return }
        
        for lua_file in file.list().into_iter().filter(|n| n.extension == "lua") {
            if let Ok(content) = file.text(&lua_file.path) {
                if content.contains(".deleteFolder") || content.contains(".deleteFile") {
                    self.warn(ModError::InfoDangerousFile);
                    return
                }
            }
        }
    }

    /// Update the badge list
    fn update_badges(&mut self) {
        if BADGE_NOT_MOD.iter().any(|x| self.issues.contains(x)) {
            self.badge_array.push(Badges::NotMod);

            if self.issues.contains(&ModError::FileErrorLikelySaveGame) {
                self.badge_array.push(Badges::SaveGame);
            }

        } else if BADGE_BROKEN.iter().any(|x| self.issues.contains(x)) {
            self.badge_array.push(Badges::Broken);
        } else {

            if BADGE_ISSUE.iter().any(|x| self.issues.contains(x)) {
                self.badge_array.push(Badges::Problem);
            }
            if BADGE_PERF.iter().any(|x| self.issues.contains(x)) {
                self.badge_array.push(Badges::Performance);
            }
            if self.file.is_folder || !self.mod_desc.multiplayer {
                self.badge_array.push(Badges::NoMp);
            }
            if self.mod_desc.script_files {
                self.badge_array.push(Badges::PcOnly);
            }
            if self.mod_desc.map_config_filename.is_some() {
                self.badge_array.push(Badges::Map);
            }

            if !self.mod_desc.action_binding.is_empty() {
                self.badge_array.push(Badges::Keys);
            }

            if !self.mod_desc.dependencies.is_empty() {
                self.badge_array.push(Badges::Depend);
            }
        }

        if self.issues.contains(&ModError::InfoMaliciousCode) || self.issues.contains(&ModError::InfoDangerousFile) {
            self.badge_array.push(Badges::Malware);
        }

        match self.mod_desc.game_version {
            11 => self.badge_array.push(Badges::Fs11),
            13 => self.badge_array.push(Badges::Fs13),
            15 => self.badge_array.push(Badges::Fs15),
            17 => self.badge_array.push(Badges::Fs17),
            19 => self.badge_array.push(Badges::Fs19),
            22 => self.badge_array.push(Badges::Fs22),
            25 => self.badge_array.push(Badges::Fs25),
            _  => self.badge_array.push(Badges::FsUnknown),
        }
    }
}


/// File related metadata for a mod
/// 
/// Note on equality - this will be equal if all of the details match,
/// ignoring the path to the file, file date, and the age hash.
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    /// Age hash - MD5 of {shortname}-{size}-{date}
    pub age_hash: String,
    /// suggested name if this appears to be a copy of a mod
    pub copy_name: Option<String>,
    /// list of extra files in mod
    pub extra_files: Vec<String>,
    /// mod file date
    pub file_date: u64,
    /// mod size (packed zip or folder contents)
    pub file_size: u64,
    /// full path to file
    pub full_path: String,
    /// list of I3D files
    pub i3d_files: Vec<String>,
    /// list of DDS files
    #[serde(rename = "imageDDS")]
    pub image_dds: Vec<String>,
    /// list of non DDS images
    #[serde(rename = "imageNonDDS")]
    pub image_non_dds: Vec<String>,
    /// folder flag (is this a folder?)
    pub is_folder: bool,
    /// save game flag (is this a save game?)
    pub is_save_game: bool,
    /// mod pack flag (is this a pack of mods?)
    pub is_mod_pack: bool,
    /// list of PNG textures (false positives possible)
    pub png_texture: Vec<String>,
    /// short name of mod (the bit before the .zip extension, or the folder name)
    pub short_name: String,
    /// list of files with spaces in them
    pub space_files: Vec<String>,
    /// list of oversized files
    pub too_big_files: Vec<String>,
    /// list of zip files
    pub zip_files: Vec<ZipPackFile>,
    /// has lua files
    pub lua_count: u32,
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.copy_name == other.copy_name &&
        self.extra_files == other.extra_files &&
        self.file_size == other.file_size &&
        self.i3d_files == other.i3d_files &&
        self.image_dds == other.image_dds &&
        self.image_non_dds == other.image_non_dds &&
        self.is_folder == other.is_folder &&
        self.is_save_game == other.is_save_game &&
        self.is_mod_pack == other.is_mod_pack &&
        self.png_texture == other.png_texture &&
        self.short_name == other.short_name &&
        self.space_files == other.space_files &&
        self.too_big_files == other.too_big_files &&
        self.zip_files == other.zip_files &&
        self.lua_count == other.lua_count
    }
}


/// Entry for zip files inside a "mod" file.
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct ZipPackFile {
    /// name of file (includes relative path)
    pub name: String,
    /// size of file (unpacked)
    pub size: u64,
}

/// Default hash value (for invalid caching items)
pub const DEFAULT_HASH:&str = "ERR-HASH-NOT-COMPUTED--";

impl FileInfo {
    /// Create a new fileinfo (from abs path)
    fn new<P: AsRef<Path>>(filename : P, age_hash : String) -> Self {
        Self {
            full_path  : filename.as_ref().to_string_lossy().to_string(),
            short_name : filename.as_ref().file_stem().map(|v| v.to_string_lossy().to_string() ).unwrap_or_default(),
            is_folder  : filename.as_ref().is_dir(),
            age_hash,
            ..Default::default()
        }
    }

    /// Get a hash from a filename
    pub fn make_hash<P: AsRef<Path>>(filename : P) -> String {
        Self::hash_from_file(filename).unwrap_or_else(|_| DEFAULT_HASH.to_owned())
    }

    /// compute hash from filename
    fn hash_from_file<P: AsRef<Path>>(filename : P) -> Result<String, std::io::Error>{
        let Some(name) = filename.as_ref().file_name() else { return Ok(DEFAULT_HASH.to_owned()) };
        let stats = std::fs::metadata(&filename)?;
        let mut file = std::fs::File::open(&filename)?;
        let mut buf = vec![0; 2048];
    
        file.seek(SeekFrom::End(-2048))?;
        file.read_exact(&mut buf)?;
    
        let mut hasher = Md5::new();
        hasher.update(name.as_encoded_bytes());
        hasher.update(stats.len().to_be_bytes());
        hasher.update(buf);
        let hash = hasher.finalize();
        Ok(Base64UrlUnpadded::encode_string(&hash))
    }
}


/// Badges
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Badges {
    /// Mod has performance issues (maybe)
    Performance,
    /// mod is totally broken
    Broken,
    /// mod is a folder
    Folder,
    /// mod might be malware
    Malware,
    /// can't be used in multiplayer
    NoMp,
    /// not actually a mod
    NotMod,
    /// pc only (scripts)
    PcOnly,
    /// mod has issues
    Problem,
    /// actually a savegame
    SaveGame,
    /// depends on something else
    Depend,
    /// version unknown
    FsUnknown,
    /// FS11
    Fs11,
    /// FS13
    Fs13,
    /// FS15
    Fs15,
    /// FS17
    Fs17,
    /// FS19
    Fs19,
    /// FS22
    Fs22,
    /// FS25
    Fs25,
    /// Has keyboard bindings
    Keys,
    /// is a map
    Map,
}

