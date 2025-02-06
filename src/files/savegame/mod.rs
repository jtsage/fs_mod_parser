use std::collections::{HashMap, HashSet};
use serde::ser::{Serialize, Serializer};



/// farms.xml file
pub mod farms;
/// vehicles.xml file
// pub mod items;
/// careerSavegame.xml
// pub mod career;


/// Data structure for savegame mods
#[derive(serde::Serialize, Clone, PartialEq, Eq, Default, Debug)]
pub struct Mods(HashMap<String, Mod>);


/// Data structure for a savegame mod
#[derive(serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Mod {
    /// Mod version from careerSavegame
    pub version: String,
    /// Mod title from careerSavegame
    pub title: String,
    /// List of farms mod is purchased on
    #[serde(serialize_with = "ordered_set")]
    pub farms: HashSet<usize>,
}

impl Default for Mod {
    fn default() -> Self {
        Self { version: String::from("0"), title: String::from("--"), farms: HashSet::default() }
    }
}


/// Order the farm set
fn ordered_set<S: Serializer, K: Ord + Serialize>(value: &HashSet<K>, serializer: S) -> Result<S::Ok, S::Error> {
    let mut ordered: Vec<_> = value.iter().collect();
    ordered.sort();
    ordered.serialize(serializer)
}