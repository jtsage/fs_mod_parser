//! Parse save game files.
use serde::ser::{Serialize, Serializer};
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile};
use std::{collections::{HashSet, HashMap}, path::Path};

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum SaveError {
    FileUnreadable,
    FarmsMissing,
    FarmsParseError,
    PlaceableMissing,
    PlaceableParseError,
    VehicleMissing,
    VehicleParseError,
    CareerMissing,
    CareerParseError,
}

impl Serialize for SaveError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            SaveError::FileUnreadable      => serializer.serialize_unit_variant("SaveError", 0, "SAVE_ERROR_UNREADABLE"),
            SaveError::FarmsMissing        => serializer.serialize_unit_variant("SaveError", 1, "SAVE_ERROR_MISSING_FARMS"),
            SaveError::FarmsParseError     => serializer.serialize_unit_variant("SaveError", 2, "SAVE_ERROR_PARSE_FARMS"),
            SaveError::PlaceableMissing    => serializer.serialize_unit_variant("SaveError", 3, "SAVE_ERROR_MISSING_PLACABLE"),
            SaveError::PlaceableParseError => serializer.serialize_unit_variant("SaveError", 4, "SAVE_ERROR_PARSE_PLACABLE"),
            SaveError::VehicleMissing      => serializer.serialize_unit_variant("SaveError", 5, "SAVE_ERROR_MISSING_VEHICLE"),
            SaveError::VehicleParseError   => serializer.serialize_unit_variant("SaveError", 6, "SAVE_ERROR_PARSE_VEHICLE"),
            SaveError::CareerMissing       => serializer.serialize_unit_variant("SaveError", 7, "SAVE_ERROR_MISSING_CAREER"),
            SaveError::CareerParseError    => serializer.serialize_unit_variant("SaveError", 8, "SAVE_ERROR_PARSE_CAREER"),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGameMod {
    pub version : String,
    pub title : String,
    pub farms : HashSet<usize>,
}
impl SaveGameMod {
    fn new() -> Self {
        SaveGameMod {
            version : String::from("0"),
            title : String::from("--"),
            farms : HashSet::new()
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGameFarm {
    pub name : String,
    pub cash : i64,
    pub loan : i64,
    pub color : usize,
}

impl SaveGameFarm {
    fn new(name : String) -> Self {
        SaveGameFarm {
            name,
            cash : 0_i64,
            loan : 0_i64,
            color : 1_usize,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGameRecord {
    pub error_list  : HashSet<SaveError>,
    pub farms       : HashMap<usize,  SaveGameFarm>,
    pub is_valid    : bool,
    pub map_mod     : Option<String>,
    pub mods        : HashMap<String, SaveGameMod>,
    pub single_farm : bool
}

impl SaveGameRecord {
    /// raise an error on the savegame
    fn add_issue(&mut self, issue : SaveError) {
        self.is_valid = false;
        self.error_list.insert(issue);
    }
    fn new() -> Self {
        SaveGameRecord {
            error_list : HashSet::new(),
            farms : HashMap::from([
                (0_usize, SaveGameFarm::new(String::from("--unowned--")))
            ]),
            is_valid : true,
            map_mod : None,
            mods : HashMap::new(),
            single_farm : true
        }
    }
    pub fn pretty_print(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }
}

impl std::fmt::Display for SaveGameRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

pub fn parse_to_json(full_path :&Path, is_folder: bool) -> String {
    parser(full_path, is_folder).to_string()
}

pub fn parser(full_path :&Path, is_folder: bool) -> SaveGameRecord {
    let mut save_record = SaveGameRecord::new();

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder 
        {
            if let Ok(archive) = AbstractFolder::new(full_path) {
                Box::new(archive)
            } else {
                save_record.add_issue(SaveError::FileUnreadable);
                return save_record;
            }
        } else if let Ok(archive) = AbstractZipFile::new(full_path) {
            Box::new(archive)
        } else {
            save_record.add_issue(SaveError::FileUnreadable);
            return save_record;
        };

    let Ok(farms_content) = abstract_file.as_text("farms.xml") else {
        save_record.add_issue(SaveError::FarmsMissing);
        return save_record;
    };

    let Ok(farms_document) = roxmltree::Document::parse(&farms_content) else {
        save_record.add_issue(SaveError::FarmsParseError);
        return save_record;
    };

    #[allow(clippy::cast_possible_truncation)]
    for farm_entry in farms_document.descendants().filter(|n|n.has_tag_name("farm")) {
        let Some(farm_id_str) = farm_entry.attribute("farmId") else { continue; };
        let Ok(farm_id) = farm_id_str.parse::<usize>() else { continue; };
        let Some(farm_name) = farm_entry.attribute("name") else { continue; };

        let mut farm_record = SaveGameFarm::new(farm_name.to_owned());

        farm_record.loan = match farm_entry.attribute("loan") {
            Some(value) => value.parse::<f64>().unwrap_or(0_f64) as i64,
            None => 0_i64
        };

        farm_record.cash = match farm_entry.attribute("money") {
            Some(value) => value.parse::<f64>().unwrap_or(0_f64) as i64,
            None => 0_i64
        };

        save_record.farms.insert(farm_id, farm_record);
    }
    save_record
}
