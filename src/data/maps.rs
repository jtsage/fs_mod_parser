use std::collections::HashMap;
use serde::ser::{Serialize, SerializeMap, Serializer};
use super::structs::CropWeatherType;


pub struct CropTypeState {
    name        : &'static str,
    max_harvest : u8,
    min_harvest : u8,
    states      : u8,
}

pub struct CropTypeStateBuilder {
    pub max_harvest : u8,
    pub min_harvest : u8,
    pub name        : String,
    pub states      : u8,
}

pub fn crops_from_base_game() -> std::option::Option<Vec<CropOutput>> {
    let mut crop_list:Vec<CropOutput> = vec![];

    for crop in BG_CROPS.iter() {
        let mut harvest_periods: Vec<u8> = vec![];
        for (i, elem) in crop.harvest_periods.iter().enumerate() {
            if *elem { harvest_periods.push(i as u8 + 1) }
        }

        let mut plant_periods: Vec<u8> = vec![];
        for (i, elem) in crop.plant_periods.iter().enumerate() {
            if *elem { plant_periods.push(i as u8 + 1) }
        }

        crop_list.push(CropOutput {
            name : crop.name.to_owned(),
            growth_time : crop.growth_time,
            harvest_periods,
            plant_periods,
        });
    }
    Some(crop_list)
}

pub fn fruits_from_base_game() -> Vec<CropTypeStateBuilder> {
    let mut collector:Vec<CropTypeStateBuilder> = vec![];

    for item in BG_CROP_TYPES {
        collector.push( CropTypeStateBuilder{
            name        : item.name.to_owned(),
            max_harvest : item.max_harvest,
            min_harvest : item.min_harvest,
            states      : item.states
        });
    }
    collector
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

impl Serialize for CropWeather {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        let mut min_max: HashMap<&str, HashMap<&str, i8>> = HashMap::new();

        for item in &self.seasons {
            min_max.insert(item.name, HashMap::from([
                ("min", item.min),
                ("max", item.max)
            ]));
        }
        map.serialize_entry(&self.name, &min_max)?;
        map.end()
    }
}

pub const SKIP_CROP_TYPES: [&str; 1] = [
    "meadow"
];

pub const BG_CROP_TYPES: [CropTypeState; 17] = [
    CropTypeState { name : "wheat", max_harvest : 8, min_harvest : 8, states : 8},
    CropTypeState { name : "barley", max_harvest : 7,  min_harvest : 7,  states : 7 },
    CropTypeState { name : "canola", max_harvest : 9,  min_harvest : 9,  states : 9 },
    CropTypeState { name : "oat", max_harvest : 5,  min_harvest : 5,  states : 5 },
    CropTypeState { name : "maize", max_harvest : 7,  min_harvest : 7,  states : 7 },
    CropTypeState { name : "sunflower", max_harvest : 8,  min_harvest : 8,  states : 8 },
    CropTypeState { name : "soybean", max_harvest : 7,  min_harvest : 7,  states : 7 },
    CropTypeState { name : "potato", max_harvest : 6,  min_harvest : 6,  states : 6 },
    CropTypeState { name : "sugarbeet", max_harvest : 8,  min_harvest : 8,  states : 8 },
    CropTypeState { name : "sugarcane", max_harvest : 8,  min_harvest : 8,  states : 8 },
    CropTypeState { name : "cotton", max_harvest : 9,  min_harvest : 9,  states : 9 },
    CropTypeState { name : "sorghum", max_harvest : 5,  min_harvest : 5,  states : 5 },
    CropTypeState { name : "grape", max_harvest : 11, min_harvest : 10, states : 7 },
    CropTypeState { name : "olive", max_harvest : 10, min_harvest : 9,  states : 7 },
    CropTypeState { name : "poplar", max_harvest : 14, min_harvest : 14, states : 14 },
    CropTypeState { name : "grass", max_harvest : 4,  min_harvest : 3,  states : 4 },
    CropTypeState { name : "oilseedradish", max_harvest: 2, min_harvest : 2,  states : 2 },
];

pub const BG_CROP_WEATHER: [CropWeather; 3] = [
    CropWeather { name : "mapUS", seasons : [
        CropSeason { name : "spring", min : 6, max : 18 },
        CropSeason { name : "summer", min : 13, max : 34 },
        CropSeason { name : "autumn", min : 5, max : 25 },
        CropSeason { name : "winter", min : -11, max : 10 },
    ]},
    CropWeather { name : "mapFR", seasons : [
        CropSeason { name : "spring", min : 6, max : 18 },
        CropSeason { name : "summer", min : 13, max : 34 },
        CropSeason { name : "autumn", min : 5, max : 25 },
        CropSeason { name : "winter", min : -11, max : 10 },
    ]},
    CropWeather { name : "mapAlpine", seasons : [
        CropSeason { name : "spring", min : 5, max : 18 },
        CropSeason { name : "summer", min : 10, max : 30 },
        CropSeason { name : "autumn", min : 4, max : 22 },
        CropSeason { name : "winter", min : -12, max : 8 },
    ]},
];

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
    name : String,
    growth_time : u8,
    harvest_periods : Vec<u8>,
    plant_periods : Vec<u8>,
}

pub const BG_CROPS: [Crop; 17] = [
    Crop {
        name : "wheat",
        growth_time : 8,
        harvest_periods : [false,false,false,false,true,true,false,false,false,false,false,false],
        plant_periods : [false,false,false,false,false,false,true,true,false,false,false,false]
    },
    Crop {
        name : "barley",
        growth_time : 7,
        harvest_periods : [false,false,false,true,true,false,false,false,false,false,false,false],
        plant_periods : [false,false,false,false,false,false,true,true,false,false,false,false]
    },
    Crop {
        name : "canola",
        growth_time : 9,
        harvest_periods : [false,false,false,false,true,true,false,false,false,false,false,false],
        plant_periods : [false,false,false,false,false,true,true,false,false,false,false,false]
    },
    Crop {
        name : "oat",
        growth_time : 5,
        harvest_periods : [false,false,false,false,true,true,false,false,false,false,false,false],
        plant_periods : [true,true,false,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "maize",
        growth_time : 7,
        harvest_periods : [false,false,false,false,false,false,false,true,true,false,false,false],
        plant_periods : [false,true,true,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "sunflower",
        growth_time : 8,
        harvest_periods : [false,false,false,false,false,false,false,true,true,false,false,false],
        plant_periods : [true,true,false,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "soybean",
        growth_time : 7,
        harvest_periods : [false,false,false,false,false,false,false,true,true,false,false,false],
        plant_periods : [false,true,true,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "potato",
        growth_time : 6,
        harvest_periods : [false,false,false,false,false,true,true,false,false,false,false,false],
        plant_periods : [true,true,false,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "sugarbeet",
        growth_time : 8,
        harvest_periods : [false,false,false,false,false,false,false,true,true,false,false,false],
        plant_periods : [true,true,false,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "sugarcane",
        growth_time : 8,
        harvest_periods : [false,false,false,false,false,false,false,true,true,false,false,false],
        plant_periods : [true,true,false,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "cotton",
        growth_time : 9,
        harvest_periods : [false,false,false,false,false,false,false,true,true,false,false,false],
        plant_periods : [true,false,false,false,false,false,false,false,false,false,false,true]
    },
    Crop {
        name : "sorghum",
        growth_time : 5,
        harvest_periods : [false,false,false,false,false,true,true,false,false,false,false,false],
        plant_periods : [false,true,true,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "grape",
        growth_time : 7,
        harvest_periods : [false,false,false,false,false,false,true,true,false,false,false,false],
        plant_periods : [true,true,true,false,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "olive",
        growth_time : 7,
        harvest_periods : [false,false,false,false,false,false,false,true,false,false,false,false],
        plant_periods : [true,true,true,true,false,false,false,false,false,false,false,false]
    },
    Crop {
        name : "poplar",
        growth_time : 14,
        harvest_periods : [true,true,true,true,true,true,true,true,true,true,true,true],
        plant_periods : [true,true,true,true,true,true,false,false,false,false,false,false]
    },
    Crop {
        name : "grass",
        growth_time : 4,
        harvest_periods : [true,true,true,true,true,true,true,true,true,true,true,true],
        plant_periods : [true,true,true,true,true,true,true,true,true,false,false,false]
    },
    Crop {
        name : "oilseedradish",
        growth_time : 2,
        harvest_periods : [true,true,true,true,true,true,true,true,true,true,true,true],
        plant_periods : [true,true,true,true,true,true,true,true,false,false,false,false]
    }
];


impl Serialize for Crop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut harvest_periods: Vec<u8> = vec![];
        for (i, elem) in self.harvest_periods.iter().enumerate() {
            if *elem { harvest_periods.push(i as u8 + 1) }
        }
        let mut plant_periods: Vec<u8> = vec![];
        for (i, elem) in self.plant_periods.iter().enumerate() {
            if *elem { plant_periods.push(i as u8 + 1) }
        }

        let crop_json: CropOutput = CropOutput {
            name : self.name.to_owned(),
            growth_time : self.growth_time,
            harvest_periods,
            plant_periods,
        };
        crop_json.serialize(serializer)
    }
}


pub fn read_map_basics(mod_record : &mut super::structs::ModRecord, file_handle: &mut Box<dyn super::super::files::AbstractFileHandle> ) {
    if mod_record.mod_desc.map_config_file.is_none() { return (); }

    let (fruits, growth, env_in, env_base) = match file_handle.as_text( &mod_record.mod_desc.map_config_file.clone().unwrap().as_str()) {
        Ok(contents) => {
            match super::functions::parse_xml(&contents) {
                Ok(map_config_tree) => (
                    super::functions::nullify_base_game_entry(&map_config_tree, "fruitTypes"),
                    super::functions::nullify_base_game_entry(&map_config_tree, "growth"),
                    super::functions::nullify_base_game_entry(&map_config_tree, "environment"),
                    super::functions::get_base_game_entry_key(&map_config_tree)
                ),
                Err(..) => (None, None, None, None)
            }
        },
        Err(..) => (None, None, None, None)
    };

    if env_base.is_some() {
        let mut weather_map:CropWeatherType = HashMap::new();

        for key in BG_CROP_WEATHER {
            if env_base == Some(key.name.to_string()) {
                for season in key.seasons {
                    weather_map.insert(season.name.to_string(), HashMap::from([
                        ("min".to_string(), season.min),
                        ("max".to_string(), season.max)
                    ]));
                }
                mod_record.mod_desc.crop_weather = Some(weather_map.clone());
            }
        }
    } else {
        match &env_in {
            Some(file_name) => match file_handle.as_text( &file_name) {
                Ok(contents) => {
                    match super::functions::parse_xml(&contents) {
                        Ok(tree) => {
                            let mut weather_map:CropWeatherType = HashMap::new();

                            match tree.descendants().find(|n|n.has_tag_name("latitude")) {
                                Some(node) => {
                                    if node.text().unwrap().parse::<f32>().unwrap() < 0.0 {
                                        mod_record.mod_desc.map_is_south = true
                                    }
                                },
                                None => {}
                            }

                            for season in tree.descendants().filter(|n|n.has_tag_name("season") && n.has_attribute("name")) {
                                let mut min_temp:i8 = 127;
                                let mut max_temp:i8 = -127;

                                for variant in season.descendants().filter(|n|n.has_tag_name("variation") && n.has_attribute("minTemperature") && n.has_attribute("maxTemperature")) {
                                    min_temp = std::cmp::min(min_temp, variant.attribute("minTemperature").unwrap().parse::<i8>().unwrap() );
                                    max_temp = std::cmp::max(max_temp, variant.attribute("maxTemperature").unwrap().parse::<i8>().unwrap() );
                                }

                                weather_map.insert(season.attribute("name").unwrap().to_string(), HashMap::from([
                                    ("min".to_string(), min_temp),
                                    ("max".to_string(), max_temp)
                                ]));
                            }
                            mod_record.mod_desc.crop_weather = Some(weather_map.clone());
                        },
                        Err(..) => {}
                    }
                },
                Err(..) => {}
            },
            None => {}
        }
    }

    if growth.is_none() {
        mod_record.mod_desc.crop_info = crops_from_base_game();
        return ();
    }

    let crop_builder:Vec<CropTypeStateBuilder> = match &fruits {
        Some(file_name) => match file_handle.as_text( &file_name) {
            Ok(contents) => {
                match super::functions::parse_xml(&contents) {
                    Ok(tree) => {
                        let mut new_build:Vec<CropTypeStateBuilder> = vec![];

                        for item in tree.descendants().filter(|n|n.has_tag_name("fruitType")) {
                            let item_name = item.attribute("name").unwrap_or("unknown").to_owned();

                            if SKIP_CROP_TYPES.contains(&item_name.as_str()) { continue }

                            let mut item_struct = CropTypeStateBuilder{
                                name        : item_name,
                                max_harvest : item.children().find(|n|n.has_tag_name("harvest") && n.has_attribute("maxHarvestingGrowthState")).unwrap().attribute("maxHarvestingGrowthState").unwrap_or("20").parse::<u8>().unwrap().clone(),
                                min_harvest : item.children().find(|n|n.has_tag_name("harvest") && n.has_attribute("minHarvestingGrowthState")).unwrap().attribute("minHarvestingGrowthState").unwrap_or("20").parse::<u8>().unwrap().clone(),
                                states      : item.children().find(|n|n.has_tag_name("growth") && n.has_attribute("numGrowthStates")).unwrap().attribute("numGrowthStates").unwrap_or("20").parse::<u8>().unwrap().clone(),
                            };

                            match item.children().find(|n|n.has_tag_name("preparing")) {
                                Some(prep_node) => {
                                    match prep_node.attribute("minGrowthState") {
                                        Some(val) => item_struct.min_harvest = val.parse::<u8>().unwrap(),
                                        None => {}
                                    }
                                    match prep_node.attribute("maxGrowthState") {
                                        Some(val) => item_struct.max_harvest = val.parse::<u8>().unwrap(),
                                        None => {}
                                    }
                                },
                                None => {}
                            }
                            new_build.push(item_struct);
                        }
                        new_build
                    },
                    Err(..) => fruits_from_base_game()
                }
            },
            Err(..) => fruits_from_base_game()
        },
        None => fruits_from_base_game()
    };

    match &growth {
        Some(file_name) => match file_handle.as_text( &file_name) {
            Ok(contents) => {
                match super::functions::parse_xml(&contents) {
                    Ok(tree) => {
                        let mut crop_list:Vec<CropOutput> = vec![];
                        for fruit in tree.descendants().filter(|n|n.has_tag_name("fruit")) {
                            let fruit_name = fruit.attribute("name").unwrap_or("unknown").to_owned();

                            if SKIP_CROP_TYPES.contains(&fruit_name.as_str()) { continue }

                            let builder = crop_builder.iter().find(|n|n.name == fruit_name);

                            if builder.is_none() { continue }

                            let builder_unwrapped = builder.unwrap();

                            let mut crop_def = CropOutput {
                                name : fruit_name,
                                growth_time : builder_unwrapped.states,
                                harvest_periods : vec![],
                                plant_periods : vec![]
                            };

                            let mut last_maximum_state:u8 = 0;

                            for period in fruit.children().filter(|n|n.has_tag_name("period") && n.has_attribute("index")) {
                                let mut die_back_happened = false;
                                let current_period_index = period.attribute("index").unwrap().parse::<u8>().unwrap();

                                match period.attribute("plantingAllowed") {
                                    Some(value) => if value == "true" { crop_def.plant_periods.push(current_period_index)},
                                    None => {}
                                }

                                let updates_count = period.children().filter(|n|n.has_tag_name("update")).count();

                                if updates_count == 0 {
                                    // if we are already harvestable, we still are with no update
                                    if last_maximum_state >= builder_unwrapped.min_harvest && last_maximum_state <= builder_unwrapped.max_harvest {
                                        crop_def.harvest_periods.push(current_period_index);
                                    }
                                } else {
                                    // do the updates
                                    for update in period.children().filter(|n|n.has_tag_name("update")) {
                                        match update.attribute("set") {
                                            Some(_new_state) => {
                                                // if set range > growth_time, it's a regrow.
                                                // if set range <= growth_time, it's die back
                                                let range = decode_max_range(update.attribute("range"));
                                                if range <= builder_unwrapped.states {
                                                    last_maximum_state = range;
                                                    die_back_happened  = true;
                                                }
                                            }
                                            None => {}
                                        }
                                        if ! die_back_happened {
                                            match update.attribute("add") {
                                                Some(add_value) => {
                                                    let mut new_possible_max = decode_max_range(update.attribute("range"));
                                                    new_possible_max += add_value.parse::<u8>().unwrap();
                                                    last_maximum_state = std::cmp::max(last_maximum_state, new_possible_max)
                                                },
                                                None => {}
                                            }
                                        }
                                    }
                                    if last_maximum_state >= builder_unwrapped.min_harvest && last_maximum_state <= builder_unwrapped.max_harvest {
                                        crop_def.harvest_periods.push(current_period_index);
                                    }
                                }
                            }
                            crop_list.push(crop_def);
                        }
                        mod_record.mod_desc.crop_info = Some(crop_list);
                    },
                    Err(..) => {
                        mod_record.mod_desc.crop_info = crops_from_base_game();
                        return ();
                    }
                }
            },
            Err(..) => {
                mod_record.mod_desc.crop_info = crops_from_base_game();
                return ();
            }
        },
        None => {
            mod_record.mod_desc.crop_info = crops_from_base_game();
            return ();
        }
    }
}


fn decode_max_range(range:Option<&str>) -> u8 {
    match range {
        Some(value) => { 
            if value.contains("-") {
                let ( _, end ) = value.split_at(value.find("-").unwrap()+1);
                end.parse::<u8>().unwrap()
            } else {
                value.parse::<u8>().unwrap()
            }
        }
        None => 0
    }
}

