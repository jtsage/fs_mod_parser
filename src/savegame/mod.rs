//! Parse save game files.
use serde::ser::{Serialize, Serializer};
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile};
use std::{collections::{HashSet, HashMap}, path::Path};

/// Possible parse problems with a savegame
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

/// Data structure for a savegame mod
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveGameMod {
    pub version : String,
    pub title : String,
    #[serde(serialize_with = "ordered_set")]
    pub farms : HashSet<usize>,
}
impl SaveGameMod {
    fn new() -> Self {
        SaveGameMod {
            version : String::from("0"),
            title   : String::from("--"),
            farms   : HashSet::new()
        }
    }
}
fn ordered_set<S, K: Ord + Serialize>(
    value: &HashSet<K>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ordered: Vec<_> = value.iter().collect();
    ordered.sort();
    ordered.serialize(serializer)
}

/// Data structure for a savegame farm
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

// Data structure for a savegame
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGameRecord {
    pub error_list  : HashSet<SaveError>,
    pub farms       : HashMap<usize,  SaveGameFarm>,
    pub is_valid    : bool,
    pub map_mod     : Option<String>,
    pub map_title   : Option<String>,
    pub mod_count   : usize,
    pub mods        : HashMap<String, SaveGameMod>,
    pub name        : Option<String>,
    pub play_time   : String,
    pub save_date   : String,
    pub single_farm : bool
}

impl SaveGameRecord {
    /// raise an error on the savegame
    fn add_issue(&mut self, issue : SaveError) {
        self.is_valid = false;
        self.error_list.insert(issue);
    }

    fn add_mod_with_farm(&mut self, mod_key : &str, farm_id: usize) -> &mut Self {
        let this_mod = self.mods.entry(mod_key.to_string()).or_insert_with(SaveGameMod::new);
        this_mod.farms.insert(farm_id);
        self
    }

    fn add_mod_with_detail(&mut self, mod_key : &str, title : Option<&str>, version : Option<&str>) -> &mut Self {
        let this_mod = self.mods.entry(mod_key.to_string()).or_insert_with(SaveGameMod::new);

        if let Some(title)   = title   { this_mod.title = title.to_string() }
        if let Some(version) = version { this_mod.version = version.to_string(); }

        self
    }

    fn new() -> Self {
        SaveGameRecord {
            error_list  : HashSet::new(),
            farms       : HashMap::from([
                (0_usize, SaveGameFarm::new(String::from("--unowned--")))
            ]),
            is_valid    : true,
            map_mod     : None,
            map_title   : None,
            mod_count   : 0,
            mods        : HashMap::new(),
            name        : None,
            play_time   : String::from("0:00"),
            save_date   : String::from("1970-01-01"),
            single_farm : true,
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


/// Parse a savegame into a pretty-print json representation
/// 
/// See also [`parse_to_json`]
pub fn parse_to_json_pretty<P: AsRef<Path>>(full_path :P) -> String {
    parser(full_path).pretty_print()
}

/// Parse a savegame into a json representation
/// 
/// # Sample Output
/// ```json
/// {
///   "errorList": [],
///   "farms": {
///     "4": { "name": "BELLWETHER RANCH", "cash": 110758, "loan": 0, "color": 1 },
///     "0": { "name": "--unowned--", "cash": 0, "loan": 0, "color": 1 }
///   },
///   "isValid": true,
///   "mapMod": "FS22_BackRoadsCounty",
///   "mapTitle": "Back Roads County",
///   "modCount": 38,
///   "mods": {
///     "FS22_BackRoadsCounty": {
///       "version": "1.0.0.2",
///       "title": "Back Roads County",
///       "farms": [ 0, 1, 4, 5, 15 ]
///     }
///   },
///   "name": "BRC",
///   "playTime": "306:40",
///   "saveDate": "2022-10-14",
///   "singleFarm": false
/// }
/// ```
pub fn parse_to_json<P: AsRef<Path>>(full_path :P) -> String {
    parser(full_path).to_string()
}

/// Parse a savegame
/// 
/// Returned information includes:
/// - Mods loaded and used in the save with total count
/// - Playtime, Save Date, Save Name
/// - Map mod name and title
/// - Errors, if any, and boolean valid flag
/// - Farm list, boolean if it's a multiplayer save or not
pub fn parser<P: AsRef<Path>>(full_path :P) -> SaveGameRecord {
    let mut save_record = SaveGameRecord::new();
    let is_folder = full_path.as_ref().is_dir();

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

    do_farms(&mut save_record, &mut abstract_file);
    do_placeables(&mut save_record, &mut abstract_file);
    do_vehicles(&mut save_record, &mut abstract_file);
    do_career(&mut save_record, &mut abstract_file);

    save_record.mod_count = save_record.mods.len();

    save_record
}

fn do_farms(save_record: &mut SaveGameRecord, abstract_file : &mut Box<dyn AbstractFileHandle>) {
    let Ok(farms_content) = abstract_file.as_text("farms.xml") else {
        save_record.add_issue(SaveError::FarmsMissing);
        return;
    };

    let Ok(farms_document) = roxmltree::Document::parse(&farms_content) else {
        save_record.add_issue(SaveError::FarmsParseError);
        return;
    };

    let mut ran_more_than_once = false;
    #[allow(clippy::cast_possible_truncation)]
    for farm_entry in farms_document.descendants().filter(|n|n.has_tag_name("farm")) {
        let Some(farm_id_str) = farm_entry.attribute("farmId") else { continue; };
        let Ok(farm_id) = farm_id_str.parse::<usize>() else { continue; };
        let Some(farm_name) = farm_entry.attribute("name") else { continue; };

        if ran_more_than_once {
            save_record.single_farm = false;
        } else {
            ran_more_than_once = true;
        }

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
}

fn do_placeables(save_record: &mut SaveGameRecord, abstract_file : &mut Box<dyn AbstractFileHandle> ) {
    let Ok(placeable_content) = abstract_file.as_text("placeables.xml") else {
        save_record.add_issue(SaveError::PlaceableMissing);
        return;
    };

    let Ok(placeable_document) = roxmltree::Document::parse(&placeable_content) else {
        save_record.add_issue(SaveError::PlaceableParseError);
        return;
    };

    for item in placeable_document.descendants().filter(|n| n.has_tag_name("placeable") && n.has_attribute("farmId") && n.has_attribute("modName")) {
        let farm_id = item.attribute("farmId")
            .unwrap_or("0").parse::<usize>()
            .unwrap_or(0);

        if let Some(mod_key) = item.attribute("modName") {
            save_record.add_mod_with_farm(mod_key, farm_id);
        }
    }
}

fn do_vehicles(save_record: &mut SaveGameRecord, abstract_file : &mut Box<dyn AbstractFileHandle> ) {
    let Ok(vehicles_content) = abstract_file.as_text("vehicles.xml") else {
        save_record.add_issue(SaveError::VehicleMissing);
        return;
    };

    let Ok(vehicles_document) = roxmltree::Document::parse(&vehicles_content) else {
        save_record.add_issue(SaveError::VehicleParseError);
        return;
    };

    for item in vehicles_document.descendants().filter(|n| n.has_tag_name("vehicle") && n.has_attribute("farmId") && n.has_attribute("modName")) {
        let farm_id = item.attribute("farmId")
            .unwrap_or("0").parse::<usize>()
            .unwrap_or(0);

        if let Some(mod_key) = item.attribute("modName") {
            save_record.add_mod_with_farm(mod_key, farm_id);
        }
    }
}

fn do_career(save_record: &mut SaveGameRecord, abstract_file : &mut Box<dyn AbstractFileHandle> ) {
    let Ok(career_content) = abstract_file.as_text("careerSavegame.xml") else {
        save_record.add_issue(SaveError::CareerMissing);
        return;
    };

    let Ok(career_document) = roxmltree::Document::parse(&career_content) else {
        save_record.add_issue(SaveError::CareerParseError);
        return;
    };


    if let Some(node) = career_document.descendants().find(|n| n.has_tag_name("mapTitle")) {
        if let Some(value) = node.text() {save_record.map_title = Some(value.to_string()) }
    }

    if let Some(node) = career_document.descendants().find(|n| n.has_tag_name("savegameName")) {
        if let Some(value) = node.text() { save_record.name = Some(value.to_string()) }
    }

    if let Some(node) = career_document.descendants().find(|n| n.has_tag_name("saveDate")) {
        if let Some(value) = node.text() { save_record.save_date = value.to_string() }
    }

    if let Some(node) = career_document.descendants().find(|n| n.has_tag_name("playTime")) {
        if let Some(value) = node.text() { 
            if let Ok(value_f) = value.parse::<f64>() {
                let hours = (value_f / 60_f64).floor();
                let minutes = (value_f % 60_f64).floor();
                save_record.play_time = format!("{hours:.0}:{minutes:02.0}");
            }
        }
    }

    if let Some(node) = career_document.descendants().find(|n| n.has_tag_name("mapId")) {
        if let Some(map_pattern) = node.text() {
            let map_split : Vec<&str> = map_pattern.split('.').collect();
            save_record.map_mod = Some(map_split[0].to_string());
        }
    }
 
    for item in career_document.descendants().filter(|n| n.has_tag_name("mod") && n.has_attribute("modName")) {
        if let Some(mod_key) = item.attribute("modName") {
            save_record.add_mod_with_detail(
                mod_key,
                item.attribute("title"),
                item.attribute("version")
            );
        }
    }
}