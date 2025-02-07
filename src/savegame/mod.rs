//! Parse save game files.

use std::{ collections::HashSet, path::Path };
use crate::errors::{AbstractFileError, SaveError};
use crate::files::{AbstractFile, XMLReader};
use crate::files::savegame::career::Career as SaveCareer;
use crate::files::savegame::farms::Farms as SaveFarms;
use crate::files::savegame::items::Mods as SaveMods;

/// Parse a savegame from a filename
///
/// Returned information includes:
/// - Mods loaded and used in the save with total count
/// - Playtime, Save Date, Save Name
/// - Map mod name and title
/// - Errors, if any, and boolean valid flag
/// - Farm list, boolean if it's a multiplayer save or not
///
/// # Sample Output
/// 
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
///       "version": "1.0.0.2", "title": "Back Roads County", "farms": [ 0, 1, 4, 5, 15 ]
///     }
///   },
///   "name": "BRC",
///   "playTime": "306:40",
///   "saveDate": "2022-10-14",
///   "singleFarm": false
/// }
/// ```
/// 
/// # Open file handle:
/// 
/// If you already have an open file handle, see [`SaveGame::from_abstract`]
/// 
pub fn parser<P: AsRef<Path>>(filename : P) -> SaveGame {
    let mut file_handle = AbstractFile::new(filename);
    SaveGame::from_abstract(&mut file_handle)
}

/// Data structure for a savegame
#[derive(serde::Serialize, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SaveGame {
    /// List of found errors
    pub error_list: HashSet<SaveError>,
    /// List of farms
    pub farms: SaveFarms,
    /// Save passed all checks
    pub is_valid: bool,
    /// Map mod name (shortname)
    pub map_mod: Option<String>,
    /// Map title
    pub map_title: Option<String>,
    /// Number of mods loaded
    pub mod_count: usize,
    /// List of mods
    pub mods: SaveMods,
    /// Name of the save
    pub name: Option<String>,
    /// Playtime in hours:minutes, hours is unbound
    pub play_time: Option<String>,
    /// Save date, in rfc3339
    pub save_date: Option<String>,
    /// Single player save
    pub single_farm: bool,
}

impl SaveGame {
    #[inline]
    /// Raise and issue with the save
    fn raise_issue(&mut self, v : SaveError) -> &mut Self {
        self.error_list.insert(v);
        self
    }

    /// Load from an [`AbstractFile`]
    fn from_abstract(mod_file : &mut AbstractFile) -> Self {
        let mut save_record = Self::default();

        let Ok(career) = SaveCareer::from_abstract(mod_file).map_err(|e| {
            match e {
                AbstractFileError::XmlParseError => {
                    save_record.raise_issue(SaveError::CareerParseError)
                },
                _ => { save_record.raise_issue(SaveError::CareerMissing) },
            };
        }) else { return save_record };

        let Ok(farms) = SaveFarms::from_abstract(mod_file).map_err(|e| {
            match e {
                AbstractFileError::XmlParseError => {
                    save_record.raise_issue(SaveError::FarmsParseError)
                },
                _ => { save_record.raise_issue(SaveError::FarmsMissing) },
            };
        }) else { return save_record };
        
        let Ok(vehicles) = SaveMods::from_abstract_file(mod_file, "vehicles.xml").map_err(|e| {
            match e {
                AbstractFileError::XmlParseError => {
                    save_record.raise_issue(SaveError::VehicleParseError)
                },
                _ => { save_record.raise_issue(SaveError::VehicleMissing) },
            };
        }) else { return save_record };

        let Ok(placeables) = SaveMods::from_abstract_file(mod_file, "placeables.xml").map_err(|e| {
            match e {
                AbstractFileError::XmlParseError => {
                    save_record.raise_issue(SaveError::PlaceableParseError)
                },
                _ => { save_record.raise_issue(SaveError::PlaceableMissing) },
            };
        }) else { return save_record };
        

        save_record.mods        = career.mods;
        save_record.mods.deep_merge(placeables);
        save_record.mods.deep_merge(vehicles);
        save_record.mod_count   = save_record.mods.0.len();

        save_record.single_farm = farms.0.len() <= 2;
        save_record.farms       = farms;

        save_record.map_mod   = career.map_mod;
        save_record.map_title = career.map_title;
        save_record.name      = career.name;
        save_record.play_time = career.play_time;
        save_record.save_date = career.save_date;
        save_record.is_valid  = true;

        save_record
    }
}


// MARK: TESTING
#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_include;

    #[test]
    fn good_file() {
        let actual = parser("tests/test_mods/SAVEGAME_Good.zip");

        // cSpell: disable
        let expected = serde_json::json!({
            "errorList": [],
            "farms": [
                { "name": "--unowned--", "cash": 0, "loan": 0, "color": 0 },
                { "name": "HENNESSEY ACRES", "cash": 46198, "loan": 230000, "color": 7 },
                { "name": "joinFSG.gg", "cash": 100000, "loan": 0, "color": 1 },
                { "name": "PUBLIC", "cash": 878837, "loan": 0, "color": 8 },
                { "name": "BELLWETHER RANCH", "cash": 110758,"loan": 0,"color": 2 },
                { "name": "THE CROFT", "cash": 42937, "loan": 0, "color": 6 }
            ],
            "isValid": true,
            "mapMod": "FS22_BackRoadsCounty",
            "mapTitle": "Back Roads County",
            "modCount": 38,
            "mods": {
                "FS22_BackRoadsCounty": {
                    "version": "1.0.0.2",
                    "title": "Back Roads County",
                    "farms": [ 0, 1, 4, 5, 15]
                }
            },
            "name": "BRC",
            "playTime": "306:40",
            "saveDate": "2022-10-14",
            "singleFarm": false
        });
        // cSpell:enable

        assert_json_include!(actual: serde_json::json!(actual), expected: expected);
    }

    #[test]
    fn bad_career_file() {
        let actual = parser("tests/test_mods/SAVEGAME_No_Career.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::CareerMissing));

        let actual = parser("tests/test_mods/SAVEGAME_Broken_Career.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::CareerParseError));
    }

    #[test]
    fn bad_farms_file() {
        let actual = parser("tests/test_mods/SAVEGAME_No_Farms.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::FarmsMissing));

        let actual = parser("tests/test_mods/SAVEGAME_Broken_Farms.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::FarmsParseError));
    }

    #[test]
    fn bad_placeable_file() {
        let actual = parser("tests/test_mods/SAVEGAME_No_Placeable.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::PlaceableMissing));

        let actual = parser("tests/test_mods/SAVEGAME_Broken_Placeable.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::PlaceableParseError));
    }

    #[test]
    fn bad_vehicle_file() {
        let actual = parser("tests/test_mods/SAVEGAME_No_Vehicles.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::VehicleMissing));

        let actual = parser("tests/test_mods/SAVEGAME_Broken_Vehicles.zip");
        assert_eq!(actual.error_list.len(), 1);
        assert!(actual.error_list.contains(&SaveError::VehicleParseError));
    }

    #[test]
    fn single_farm() {
        let actual = parser("tests/test_mods/SAVEGAME_Single_Farm.zip");
        assert_eq!(actual.error_list.len(), 0);
        assert_eq!(actual.single_farm, true);
    }

}