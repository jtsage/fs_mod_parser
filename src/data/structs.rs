use std::{collections::HashSet, path::Path};
// use serde::Serialize;
use std::collections::HashMap;

use super::flags::*;
use super::maps::CropOutput;
use serde::ser::{Serialize, Serializer};


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


#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDescL10N {
    pub title : HashMap<String, String>,
    pub description : HashMap<String, String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModRecord {
    pub badge_array        : ModBadges,
    pub can_not_use        : bool,
    pub current_collection : String,
    pub file_detail        : ModFile,
    pub issues             : HashSet<super::flags::ModError>,
    pub l10n               : ModDescL10N,
    pub md5_sum            : Option<String>,
    pub mod_desc           : ModDesc,
    pub uuid               : String,
}

impl ModRecord {
    pub fn add_issue(&mut self, issue : ModError) {
        self.issues.insert(issue);
    }
    pub fn update_badges(&mut self) -> &mut ModRecord{
        self.badge_array.notmod = BADGE_NOT_MOD.iter().any(|x| self.issues.contains(x));
        self.badge_array.pconly = self.mod_desc.script_files > 0;

        if self.file_detail.is_save_game {
            self.badge_array.savegame = true;
            self.badge_array.broken = false;
            self.badge_array.problem = false;
        } else {
            self.badge_array.savegame = false;
            self.badge_array.folder = self.file_detail.is_folder;
            self.badge_array.no_mp  = !self.badge_array.notmod && (self.file_detail.is_folder || ! self.mod_desc.multi_player);
            self.badge_array.malware = self.issues.contains(&ModError::InfoMaliciousCode);
            self.badge_array.broken = BADGE_BROKEN.iter().any(|x| self.issues.contains(x));
            self.badge_array.problem = BADGE_ISSUE.iter().any(|x| self.issues.contains(x));
        }
        self
    }
}
impl std::fmt::Display for ModRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string_pretty(&self).unwrap())
    }
}

pub type CropWeatherType = HashMap<String, HashMap<String, i8>>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDesc {
    pub actions         : HashMap<String, String>,
    pub binds           : HashMap<String, Vec<String>>,
    pub author          : String,
    pub script_files    : u32,
    pub store_items     : u32,
    pub crop_info       : Option<Vec<CropOutput>>,
    pub crop_weather    : Option<CropWeatherType>,
    pub depend          : Vec<String>,
    pub desc_version    : u32,
    pub icon_file_name  : Option<String>,
    pub icon_image      : Option<String>,
    pub map_config_file : Option<String>,
    pub map_is_south    : bool,
    pub multi_player    : bool,
    pub version         : String,
}

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
}

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

fn badge_new_record() -> ModBadges {
    ModBadges {
        broken   : false,
        folder   : false,
        malware  : false,
        no_mp    : false,
        notmod   : false,
        pconly   : false,
        problem  : false,
        savegame : false,
    }
}


pub fn new_record(full_path: &Path, is_folder : bool) -> ModRecord {
    ModRecord {
        badge_array        : badge_new_record(),
        can_not_use        : true,
        current_collection : "".to_owned(),
        file_detail        : file_detail_new_record(full_path, is_folder),
        issues             : HashSet::new(),
        l10n               : ModDescL10N{
            title       : HashMap::from([("en".to_string(), "--".to_string())]),
            description : HashMap::from([("en".to_string(), "--".to_string())])
        },
        md5_sum            : None,
        mod_desc           : mod_desc_new_record(),
        uuid               : format!("{:?}", md5::compute(full_path.to_str().unwrap()))
    }
}

fn file_detail_new_record(file : &Path, is_folder : bool) -> ModFile {
    ModFile {
        copy_name     : None,
        extra_files   : vec![],
        file_date     : "".to_owned(),
        file_size     : 0,
        full_path     : format!("{}", file.to_str().unwrap()),
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
    }
}
fn mod_desc_new_record() -> ModDesc {
    ModDesc {
        actions         : HashMap::new(),
        author          : "--".to_owned(),
        binds           : HashMap::new(),
        crop_info       : None,
        crop_weather    : None,
        depend          : vec![],
        desc_version    : 0,
        icon_file_name  : None,
        icon_image      : None,
        map_config_file : None,
        map_is_south    : false,
        multi_player    : false,
        script_files    : 0,
        store_items     : 0,
        version         : "--".to_owned(),
    }
}
