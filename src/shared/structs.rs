//! Structs used to collect data for JSON export
use std::{collections::{HashMap, HashSet}, path::Path};

use crate::shared::errors::{ModError, BADGE_BROKEN, BADGE_ISSUE, BADGE_NOT_MOD};
use crate::maps::structs::{CropWeatherType, CropList};
use crate::savegame::SaveGameRecord;
use crate::mod_detail::structs::ModDetail;
use serde::ser::{Serialize, Serializer};

/// Translatable modDesc entries
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDescL10N {
    /// Translation strings for the mod title
    pub title : HashMap<String, String>,
    /// Translation strings for the mod description
    pub description : HashMap<String, String>,
}

/// Master mod record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModRecord {
    /// List of active badges
    pub badge_array        : ModBadges,
    /// Mod not usable flag
    pub can_not_use        : bool,
    /// Current collection for mod (not set)
    pub current_collection : String,
    /// Detail icons processed flag
    pub detail_icon_loaded : bool,
    /// File details
    pub file_detail        : ModFile,
    /// Errors or issues found
    pub issues             : HashSet<ModError>,
    /// storeItems found (if processed)
    pub include_detail     : Option<ModDetail>,
    /// save game record (if processed)
    pub include_save_game  : Option<SaveGameRecord>,
    /// L10N title and description
    pub l10n               : ModDescL10N,
    /// MD5 Sum (not yet implemented)
    pub md5_sum            : Option<String>,
    /// modDesc.xml fields
    pub mod_desc           : ModDesc,
    /// Mod UUID from full path and filename (MD5)
    pub uuid               : String,
}

impl ModRecord {
    /// Create a new mod record from a full path
    /// 
    /// You must define if it's a folder or not
    pub fn new<P: AsRef<Path>>(input_path :P, is_folder : bool) -> ModRecord {
        let full_path = input_path.as_ref();
        ModRecord {
            badge_array        : ModBadges::new(),
            can_not_use        : true,
            current_collection : String::new(),
            detail_icon_loaded : false,
            file_detail        : ModFile::new(full_path, is_folder),
            issues             : HashSet::new(),
            include_detail     : None,
            include_save_game  : None,
            l10n               : ModDescL10N{
                title       : HashMap::from([("en".to_string(), "--".to_string())]),
                description : HashMap::from([("en".to_string(), "--".to_string())])
            },
            md5_sum            : None,
            mod_desc           : ModDesc::new(),
            uuid               : format!("{:?}", md5::compute(full_path.to_str().unwrap_or("")))
        }
    }
    /// raise an fatal error on the mod
    pub fn add_fatal(&mut self, issue : ModError) -> &mut Self {
        self.can_not_use = true;
        self.issues.insert(issue);
        self
    }
    /// raise an error on the mod
    pub fn add_issue(&mut self, issue : ModError) -> &mut Self {
        self.issues.insert(issue);
        self
    }
    /// update the badge array from other data
    pub fn update_badges(&mut self) -> &mut Self {
        self.badge_array.notmod = BADGE_NOT_MOD.iter().any(|x| self.issues.contains(x));
        self.badge_array.pconly = self.mod_desc.script_files > 0;

        if self.file_detail.is_save_game {
            self.badge_array.savegame = true;
            self.badge_array.broken = false;
            self.badge_array.problem = false;
        } else {
            self.badge_array.savegame = false;
            self.badge_array.folder = self.file_detail.is_folder;
            self.badge_array.malware = self.issues.contains(&ModError::InfoMaliciousCode);
            self.badge_array.broken = BADGE_BROKEN.iter().any(|x| self.issues.contains(x));
            self.badge_array.problem = BADGE_ISSUE.iter().any(|x| self.issues.contains(x));
            self.badge_array.no_mp  = !self.badge_array.notmod && !self.badge_array.broken && (self.file_detail.is_folder || ! self.mod_desc.multi_player);
        }
        self
    }
    /// Output as pretty-print JSON
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }

    /// Output as JSON
    #[must_use]
    pub fn to_json(&self) -> String {
        self.to_string()
    }
}
impl std::fmt::Display for ModRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}


/// ModDesc.xml specific fields from a mod
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct ModDesc {
    /// Keyboard actions
    pub actions         : HashMap<String, String>,
    /// Keyboard bindings
    pub binds           : HashMap<String, Vec<String>>,
    /// Mod Author
    pub author          : String,
    /// Script file count
    pub script_files    : u32,
    /// Store Item count
    pub store_items     : usize,
    /// Crop details (for maps)
    pub crop_info       : CropList,
    /// Map Weather (for maps)
    pub crop_weather    : Option<CropWeatherType>,
    /// Mods this mod depends on (shortNames)
    pub depend          : Vec<String>,
    /// descVersion
    pub desc_version    : u32,
    /// icon file name
    pub icon_file_name  : Option<String>,
    /// icon image, if processed and loaded - base64 webp
    pub icon_image      : Option<String>,
    /// map config file (for maps)
    pub map_config_file : Option<String>,
    /// map has a custom environment
    pub map_custom_env  : bool,
    /// map has a custom fruit list
    pub map_custom_crop : bool,
    /// map has a custom growth file
    pub map_custom_grow : bool,
    /// map is in the southern hemisphere
    pub map_is_south    : bool,
    /// map image, if processed and loaded - base64 webp
    pub map_image       : Option<String>,
    /// multi-player capable
    pub multi_player    : bool,
    /// mod version
    pub version         : String,
}

impl ModDesc {
    /// Create an empty moddesc record
    fn new() -> ModDesc {
        ModDesc {
            actions         : HashMap::new(),
            author          : "--".to_owned(),
            binds           : HashMap::new(),
            crop_info       : CropList::new(),
            crop_weather    : None,
            depend          : vec![],
            desc_version    : 0,
            icon_file_name  : None,
            icon_image      : None,
            map_config_file : None,
            map_custom_env  : false,
            map_custom_crop : false,
            map_custom_grow : false,
            map_is_south    : false,
            map_image       : None,
            multi_player    : false,
            script_files    : 0,
            store_items     : 0,
            version         : "--".to_owned(),
        }
    }
}


/// Entry for zip files inside a "mod" file.
#[derive(serde::Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct ZipPackFile {
    /// name of file (includes relative path)
    pub name : String,
    /// size of file (unpacked)
    pub size : u64,
}

/// File related metadata for a mod
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
    /// suggested name if this appears to be a copy of a mod
    pub copy_name     : Option<String>,
    /// list of extra files in mod
    pub extra_files   : Vec<String>,
    /// mod file date
    pub file_date     : String,
    /// mod size (packed zip or folder contents)
    pub file_size     : u64,
    /// full path to file
    pub full_path     : String,
    /// list of I3D files
    pub i3d_files     : Vec<String>,
    /// list of DDS files
    #[serde(rename = "imageDDS")]
    pub image_dds     : Vec<String>,
    /// list of non DDS images
    #[serde(rename = "imageNonDDS")]
    pub image_non_dds : Vec<String>,
    /// folder flag (is this a folder?)
    pub is_folder     : bool,
    /// save game flag (is this a save game?)
    pub is_save_game  : bool,
    /// mod pack flag (is this a pack of mods?)
    pub is_mod_pack   : bool,
    /// list of PNG textures (false positives possible)
    pub png_texture   : Vec<String>,
    /// short name of mod (the bit before the .zip extension, or the folder name)
    pub short_name    : String,
    /// list of files with spaces in them
    pub space_files   : Vec<String>,
    /// list of oversized files
    pub too_big_files : Vec<String>,
    /// list of zip files
    pub zip_files     : Vec<ZipPackFile>,
}

impl ModFile {
    /// Create an empty file metadata record
    fn new(file : &Path, is_folder : bool) -> ModFile {
        ModFile {
            copy_name     : None,
            extra_files   : vec![],
            file_date     : String::new(),
            file_size     : 0,
            full_path     : file.to_str().unwrap().to_string(),
            i3d_files     : vec![],
            image_dds     : vec![],
            image_non_dds : vec![],
            is_folder     : is_folder.to_owned(),
            is_save_game  : false,
            is_mod_pack   : false,
            png_texture   : vec![],
            short_name    : file.file_stem().unwrap().to_str().unwrap().to_owned(),
            space_files   : vec![],
            too_big_files : vec![],
            zip_files     : vec![],
        }
    }
}

/// Badge information for a mod
#[allow(clippy::struct_excessive_bools)]
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct ModBadges {
    /// is broken (likely unusable)
    pub broken   : bool,
    /// is folder
    pub folder   : bool,
    /// contains malware
    pub malware  : bool,
    /// not valid for multiplayer
    pub no_mp    : bool,
    /// not a mod
    pub notmod   : bool,
    /// PC only (has LUA scripts)
    pub pconly   : bool,
    /// has a problem (likely still useable)
    pub problem  : bool,
    /// is a save game
    pub savegame : bool,
}

impl ModBadges {
    /// Create an empty badge record
    fn new() -> ModBadges{
        ModBadges {
            broken   : false, folder   : false, malware  : false, no_mp    : false,
            notmod   : false, pconly   : false, problem  : false, savegame : false,
        }
    }
}

impl Serialize for ModBadges {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut name_array:Vec<String> = vec![];
        if self.broken   { name_array.push("broken".to_string()) }
        if self.folder   { name_array.push("folder".to_string()) }
        if self.malware  { name_array.push("malware".to_string()) }
        if self.no_mp    { name_array.push("noMP".to_string()) }
        if self.notmod   { name_array.push("notmod".to_string()) }
        if self.pconly   { name_array.push("pconly".to_string()) }
        if self.problem  { name_array.push("problem".to_string()) }
        if self.savegame { name_array.push("savegame".to_string()) }
        name_array.serialize(serializer)
    }
}






