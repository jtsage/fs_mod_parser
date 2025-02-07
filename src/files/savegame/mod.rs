use std::collections::HashSet;
use serde::ser::{Serialize, Serializer};

/// farms.xml file
pub mod farms;
/// vehicles.xml file
pub mod items;
/// careerSavegame.xml
pub mod career;

/// Data structure for a savegame mod
#[derive(serde::Serialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Mod {
    /// Mod version from careerSavegame
    pub version: String,
    /// Mod title from careerSavegame
    pub title: String,
    /// List of farms mod is purchased on
    #[serde(serialize_with = "ordered_set")]
    pub farms: HashSet<usize>,
}

/// Order the farm set
fn ordered_set<S: Serializer, K: Ord + Serialize>(value: &HashSet<K>, serializer: S) -> Result<S::Ok, S::Error> {
    let mut ordered: Vec<_> = value.iter().collect();
    ordered.sort();
    ordered.serialize(serializer)
}


// MARK: TESTING
#[cfg(test)]
mod tests {
    use super::super::XMLReader;
    use super::items::Mods;
    use super::career::Career;
    // use assert_json_diff::assert_json_eq;

    #[test]
    fn good_vehicles() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/SAVEGAME_Good.zip");
        
        let vehicles = Mods::from_abstract_file(&mut file_handle, "vehicles.xml").expect("read fail");
        let placeables = Mods::from_abstract_file(&mut file_handle, "placeables.xml").expect("read fail");
        let mut career = Career::from_abstract_file(&mut file_handle, "careerSavegame.xml").expect("read fail");

        assert_eq!(placeables.0.len(), 9);
        assert_eq!(vehicles.0.len(), 8);
        assert_eq!(career.mods.0.len(), 38);

        career.mods.deep_merge(placeables);
        career.mods.deep_merge(vehicles);
        assert_eq!(career.mods.0.len(), 38);

        let mod_map = career.mods.0.get(&String::from("FS22_BackRoadsCounty")).unwrap();

        assert!(mod_map.farms.contains(&0_usize));
        assert!(mod_map.farms.contains(&1_usize));
        assert!(mod_map.farms.contains(&4_usize));
        assert!(mod_map.farms.contains(&5_usize));
        assert!(mod_map.farms.contains(&15_usize));
    }

}