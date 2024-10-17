use std::collections::HashMap;

/// Shared nested hashmap for map weather
pub type CropWeatherType = HashMap<String, HashMap<String, i8>>;

// Static version of the crop types
pub struct CropTypeState {
    pub name        : &'static str,
    pub max_harvest : u8,
    pub min_harvest : u8,
    pub states      : u8,
}

// Dynamic version of the crop types
pub struct CropTypeStateBuilder {
    pub max_harvest : u8,
    pub min_harvest : u8,
    pub name        : String,
    pub states      : u8,
}

pub struct CropSeason {
    pub name : &'static str,
    pub min : i8,
    pub max : i8,
}

pub struct CropWeather {
    pub name : &'static str,
    pub seasons : [CropSeason; 4],
}

// impl Serialize for CropWeather {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut map = serializer.serialize_map(Some(1))?;
//         let mut min_max: HashMap<&str, HashMap<&str, i8>> = HashMap::new();

//         for item in &self.seasons {
//             min_max.insert(item.name, HashMap::from([
//                 ("min", item.min),
//                 ("max", item.max)
//             ]));
//         }
//         map.serialize_entry(&self.name, &min_max)?;
//         map.end()
//     }
// }

#[derive(Clone)]
pub struct Crop {
    pub name : &'static str,
    pub growth_time : u8,
    pub harvest_periods : [bool;12],
    pub plant_periods : [bool;12],
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CropOutput {
    pub name : String,
    pub growth_time : u8,
    pub harvest_periods : Vec<u8>,
    pub plant_periods : Vec<u8>,
}