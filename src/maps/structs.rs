//! Map data structures
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::collections::HashMap;

/// Shared nested hashmap for map weather
pub type CropWeatherType = HashMap<String, HashMap<String, i8>>;

/// Static version of the crop types
pub struct CropTypeState {
    /// Crop name
    pub name: &'static str,
    /// Last valid harvest state
    pub max_harvest: u8,
    /// First valid harvest state
    pub min_harvest: u8,
    /// Number of growth states (note: states+1 is usually withered)
    pub states: u8,
}

/// Dynamic version of the crop types
pub struct CropTypeStateBuilder {
    /// Last valid harvest state
    pub max_harvest: u8,
    /// First valid harvest state
    pub min_harvest: u8,
    /// Name of crop
    pub name: String,
    /// Number of growth states (note: states+1 is usually withered)
    pub states: u8,
}

/// Static season definition
pub struct CropSeason {
    /// Name of season
    pub name: &'static str,
    /// Max temperature in celsius
    pub min: i8,
    /// Min temperature in celsius
    pub max: i8,
}

/// Static crop definition
#[derive(Clone)]
pub struct Crop {
    /// Name of crop
    pub name: &'static str,
    /// Periods for full growth
    pub growth_time: u8,
    /// Periods for valid harvest - 12 booleans
    pub harvest_periods: [bool; 12],
    /// Periods for valid sowing - 12 booleans
    pub plant_periods: [bool; 12],
}

/// Dynamic crop definition
#[derive(serde::Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct CropOutput {
    /// Periods for full growth
    pub growth_time: u8,
    /// Periods for valid harvest - vector of periods
    pub harvest_periods: Vec<u8>,
    /// Periods for valid sowing - vector of periods
    pub plant_periods: Vec<u8>,
}

impl CropOutput {
    /// create new crop output
    #[must_use]
    pub fn new(growth_time: u8) -> Self {
        CropOutput {
            growth_time,
            harvest_periods: vec![],
            plant_periods: vec![],
        }
    }
}

/// Temporary struct for serializing crop data properly
#[derive(serde::Serialize)]
struct CropSerializerOutput {
    /// Name of crop
    pub name: String,
    /// Periods for full growth
    pub growth_time: u8,
    /// Periods for valid harvest - vector of periods
    pub harvest_periods: Vec<u8>,
    /// Periods for valid sowing - vector of periods
    pub plant_periods: Vec<u8>,
}

/// Crop listing
pub struct CropList {
    /// Internal List
    list: HashMap<String, CropOutput>,
    /// Intended Order
    order: Vec<String>,
}

impl CropList {
    /// Number of crops in list
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }
    /// is crop list empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
    /// Add a crop to the list
    pub fn insert(&mut self, key: String, value: CropOutput) {
        self.list.insert(key.clone(), value);
        self.order.push(key);
    }
    /// Get a crop from the list by &str key
    pub fn get(&mut self, key: &str) -> Option<&CropOutput> {
        self.list.get(key)
    }
    #[must_use]
    /// Create new crop list
    pub fn new() -> Self {
        CropList {
            list: HashMap::new(),
            order: vec![],
        }
    }
}
impl Default for CropList {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for CropList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.list.is_empty() {
            return serializer.serialize_none();
        }
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for key in &self.order {
            let item = &self.list[key];
            let item_struct = CropSerializerOutput {
                name: key.to_string().to_lowercase(),
                growth_time: item.growth_time,
                harvest_periods: item.harvest_periods.clone(),
                plant_periods: item.plant_periods.clone(),
            };
            seq.serialize_element(&item_struct)?;
        }
        seq.end()
    }
}

#[test]
fn empty_crop_list() {
    let mine = CropList::default();

    assert_eq!(String::from("null"), serde_json::to_string(&mine).unwrap())
}
