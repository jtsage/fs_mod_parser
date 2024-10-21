//! Map data structures
use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeSeq};

/// Shared nested hashmap for map weather
pub type CropWeatherType = HashMap<String, HashMap<String, i8>>;

/// Static version of the crop types
pub struct CropTypeState {
    pub name        : &'static str,
    pub max_harvest : u8,
    pub min_harvest : u8,
    pub states      : u8,
}

/// Dynamic version of the crop types
pub struct CropTypeStateBuilder {
    pub max_harvest : u8,
    pub min_harvest : u8,
    pub name        : String,
    pub states      : u8,
}

/// Static season definition
pub struct CropSeason {
    pub name : &'static str,
    pub min : i8,
    pub max : i8,
}

/// Static weather definition
pub struct CropWeather {
    pub name : &'static str,
    pub seasons : [CropSeason; 4],
}


/// Static crop definition
#[derive(Clone)]
pub struct Crop {
    pub name : &'static str,
    pub growth_time : u8,
    pub harvest_periods : [bool;12],
    pub plant_periods : [bool;12],
}

/// Dynamic crop definition
#[derive(serde::Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct CropOutput {
    pub growth_time : u8,
    pub harvest_periods : Vec<u8>,
    pub plant_periods : Vec<u8>,
}

#[derive(serde::Serialize)]
struct CropSerializerOutput {
    pub name : String,
    pub growth_time : u8,
    pub harvest_periods : Vec<u8>,
    pub plant_periods : Vec<u8>,
}


pub struct CropList {
    list : HashMap<String, CropOutput>,
    order : Vec<String>
}

impl CropList {
    pub fn insert(&mut self, key : String, value : CropOutput) {
        self.list.insert(key.clone(), value);
        self.order.push(key);
    }
    pub fn get(&mut self, key : &str) -> Option<&CropOutput> {
        self.list.get(key)
    }
    pub fn new() -> Self {
        CropList{
            list : HashMap::new(),
            order : vec![]
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
            return serializer.serialize_none()
        }
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for key in &self.order {
            let item = &self.list[key];
            let item_struct = CropSerializerOutput{
                name : key.to_string().to_lowercase(),
                growth_time : item.growth_time,
                harvest_periods : item.harvest_periods.clone(),
                plant_periods : item.plant_periods.clone(),
            };
            seq.serialize_element(&item_struct)?;
            
        }
        seq.end()
    }
}

#[test]
fn empty_crop_list() {
    let mine = CropList::new();

    assert_eq!(String::new(), serde_json::to_string(&mine).unwrap())
}