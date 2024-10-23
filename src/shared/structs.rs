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
    pub title : HashMap<String, String>,
    pub description : HashMap<String, String>,
}

/// Master mod record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModRecord {
    pub badge_array        : ModBadges,
    pub can_not_use        : bool,
    pub current_collection : String,
    pub detail_icon_loaded : bool,
    pub file_detail        : ModFile,
    pub issues             : HashSet<ModError>,
    pub include_detail     : Option<ModDetail>,
    pub include_save_game  : Option<SaveGameRecord>,
    pub l10n               : ModDescL10N,
    pub md5_sum            : Option<String>,
    pub mod_desc           : ModDesc,
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
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }
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
    pub actions         : HashMap<String, String>,
    pub binds           : HashMap<String, Vec<String>>,
    pub author          : String,
    pub script_files    : u32,
    pub store_items     : usize,
    pub crop_info       : CropList,
    pub crop_weather    : Option<CropWeatherType>,
    pub depend          : Vec<String>,
    pub desc_version    : u32,
    pub icon_file_name  : Option<String>,
    pub icon_image      : Option<String>,
    pub map_config_file : Option<String>,
    pub map_custom_env  : bool,
    pub map_custom_crop : bool,
    pub map_custom_grow : bool,
    pub map_is_south    : bool,
    pub map_image       : Option<String>,
    pub multi_player    : bool,
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


// Entry for zip files inside a "mod" file.
#[derive(serde::Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct ZipPackFile {
    pub name : String,
    pub size : u64,
}

/// File related metadata for a mod
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
    pub copy_name     : Option<String>,
    pub extra_files   : Vec<String>,
    pub file_date     : String,
    pub file_size     : u64,
    pub full_path     : String,
    pub i3d_files     : Vec<String>,
    #[serde(rename = "imageDDS")]
    pub image_dds     : Vec<String>,
    #[serde(rename = "imageNonDDS")]
    pub image_non_dds : Vec<String>,
    pub is_folder     : bool,
    pub is_save_game  : bool,
    pub is_mod_pack   : bool,
    pub png_texture   : Vec<String>,
    pub short_name    : String,
    pub space_files   : Vec<String>,
    pub too_big_files : Vec<String>,
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
    pub broken   : bool,
    pub folder   : bool,
    pub malware  : bool,
    pub no_mp    : bool,
    pub notmod   : bool,
    pub pconly   : bool,
    pub problem  : bool,
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






