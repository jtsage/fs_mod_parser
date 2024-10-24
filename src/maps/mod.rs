//! Map file parsing
//! 
//! Reads crop data, weather data, and the map overview image
use std::collections::{HashMap, HashSet};
use regex::Regex;
use crate::shared::convert_map_image;
use crate::shared::structs::ModRecord;
use crate::shared::files::AbstractFileHandle;
use crate::maps::structs::CropList;

pub mod structs;
mod data;

use structs::{CropOutput, CropTypeStateBuilder, CropWeatherType};
use data::{BG_CROPS, BG_CROP_TYPES, BG_CROP_WEATHER, SKIP_CROP_TYPES};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_convert() {
        let input:[bool;12] = [true, false, false, true, true, false, false, true, true, false, false, true];
        let output = bool_array_to_vector(input);
        let expected: Vec<u8> = vec![1,4,5,8,9,12];
        assert_eq!(output, expected);
    }

    #[test]
    fn test_game_entry_key_valid() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="$data/maps/mapUS/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, Some("mapUS".to_string()));
    }

    #[test]
    fn test_game_entry_key_invalid() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="maps/mapUS/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, None);
    }

    #[test]
    fn test_mod_entry_invalid() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="$data/maps/mapUS/environment.xml" /></map>"#).unwrap();
        let result = nullify_base_game_entry(&document, "environment");
        assert_eq!(result, None);
    }

    #[test]
    fn test_mod_entry_valid() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="maps/mapUS/environment.xml" /></map>"#).unwrap();
        let result = nullify_base_game_entry(&document, "environment");
        assert_eq!(result, Some("maps/mapUS/environment.xml".to_string()));
    }
}

/// Convert array of booleans to vector of indexes
#[allow(clippy::cast_possible_truncation)]
fn bool_array_to_vector(input_array:[bool;12]) -> Vec<u8> {
    input_array.iter().enumerate().map(|(i,v)| if *v { i as u8 + 1_u8 } else {0_u8}).filter(|n| *n!=0_u8 ).collect()
}

/// Convert base game crop data to usable version
fn crops_from_base_game() -> CropList {
    let mut crop_list:CropList = CropList::new();

    for crop in &BG_CROPS {
        crop_list.insert(crop.name.to_owned(), CropOutput {
            growth_time : crop.growth_time,
            harvest_periods : bool_array_to_vector(crop.harvest_periods),
            plant_periods : bool_array_to_vector(crop.plant_periods),
        });
    }
    crop_list
}

/// Convert base game fruit types to usable builders
fn fruits_from_base_game() -> Vec<CropTypeStateBuilder> {
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

/// Map environment - is souther hemisphere, weather struct
struct MapEnvironment (bool, Option<CropWeatherType>);

/// Return basegame weather by key
fn weather_from_base_game(base_game_key : &str) -> MapEnvironment {
    let mut weather_map:CropWeatherType = HashMap::new();

    for key in BG_CROP_WEATHER {
        if base_game_key == key.name {
            for season in key.seasons {
                weather_map.insert(
                    season.name.to_string(),
                    HashMap::from([
                        ("min".to_string(), season.min),
                        ("max".to_string(), season.max)
                    ])
                );
            }
        }
    }

    if weather_map.is_empty() { 
        MapEnvironment( false, Some(weather_map.clone()) )
    } else {
        MapEnvironment( false, None )
    }
}

/// Read basic details about the map
/// 
/// Includes weather, crops, if it's southern, and the map image
pub fn read_map_basics(mod_record : &mut ModRecord, file_handle: &mut Box<dyn AbstractFileHandle> ) {
    let Some(map_config_file_name) = &mod_record.mod_desc.map_config_file else {
        return;
    };

    let (
        fruits,
        growth,
        env_in,
        env_base
    ) = {
        if let Ok(contents) = file_handle.as_text(map_config_file_name) {
            if let Ok(map_config_tree) = roxmltree::Document::parse(&contents) {
                mod_record.mod_desc.map_image = process_overview(&map_config_tree, mod_record, file_handle);

                (
                    nullify_base_game_entry(&map_config_tree, "fruitTypes"),
                    nullify_base_game_entry(&map_config_tree, "growth"),
                    nullify_base_game_entry(&map_config_tree, "environment"),
                    get_base_game_entry_key(&map_config_tree)
                )
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        }
    };

    mod_record.mod_desc.map_custom_crop = fruits.is_some();
    mod_record.mod_desc.map_custom_env = env_in.is_some();
    mod_record.mod_desc.map_custom_grow = growth.is_some();

    let this_map_environment = populate_weather(file_handle, env_base, env_in);
    mod_record.mod_desc.map_is_south = this_map_environment.0;
    mod_record.mod_desc.crop_weather = this_map_environment.1;

    if growth.is_none() {
        mod_record.mod_desc.crop_info = crops_from_base_game();
        return;
    }

    let crop_builder = populate_crop_builder(file_handle, fruits);

    match populate_crop_growth(file_handle, growth, &crop_builder) {
        Some(value) => mod_record.mod_desc.crop_info = value,
        None => mod_record.mod_desc.crop_info = crops_from_base_game()
    }

}

/// Decode a range argument and get the maximum from it
fn decode_max_range(range:Option<&str>) -> u8 {
    match range {
        Some(value) => { 
            if value.contains('-') {
                let ( _, end ) = value.split_at(value.find('-').unwrap()+1);
                end.parse::<u8>().unwrap_or(0_u8)
            } else {
                value.parse::<u8>().unwrap_or(0_u8)
            }
        }
        None => 0
    }
}

/// Load and convert the overview image
/// 
/// Automatically crops to the center 1/4 of the image that contains the map
/// and constrains the size to 512x512px
fn process_overview(xml_tree: &roxmltree::Document, mod_record : &mut ModRecord, file_handle: &mut Box<dyn AbstractFileHandle>) -> Option<String> {
    if let Some(filename) = xml_tree.root_element().attribute("imageFilename") {
        let mut value_string = filename.to_string().replace('\\', "/");

        if let Some(index) = value_string.find(".png") {
            value_string.replace_range(index..value_string.len(), ".dds");
        }
        if mod_record.file_detail.image_dds.contains(&value_string) {
            if let Ok(content) = file_handle.as_bin(&value_string) {
                return convert_map_image(content)
            }
        }
    }
    None
}

/// Build the crop builder struct from crop constraints
fn populate_crop_builder(file_handle: &mut Box<dyn AbstractFileHandle>, fruits : Option<String>) -> Vec<CropTypeStateBuilder> {
    if let Some(file_name) = fruits {
        if let Ok(contents) = file_handle.as_text( &file_name) {
            if let Ok(tree) = roxmltree::Document::parse(&contents) {
                let mut new_build:Vec<CropTypeStateBuilder> = vec![];

                for item in tree.descendants().filter(|n|n.has_tag_name("fruitType")) {
                    let item_name = item.attribute("name").unwrap_or("unknown").to_owned().to_lowercase();

                    if SKIP_CROP_TYPES.contains(&item_name.as_str()) { continue }

                    let mut item_struct = CropTypeStateBuilder{
                        name        : item_name,
                        max_harvest : item.children().find(|n|n.has_tag_name("harvest") && n.has_attribute("maxHarvestingGrowthState")).unwrap().attribute("maxHarvestingGrowthState").unwrap_or("20").parse::<u8>().unwrap_or(20_u8),
                        min_harvest : item.children().find(|n|n.has_tag_name("harvest") && n.has_attribute("minHarvestingGrowthState")).unwrap().attribute("minHarvestingGrowthState").unwrap_or("20").parse::<u8>().unwrap_or(20_u8),
                        states      : item.children().find(|n|n.has_tag_name("growth") && n.has_attribute("numGrowthStates")).unwrap().attribute("numGrowthStates").unwrap_or("20").parse::<u8>().unwrap_or(20_u8),
                    };

                    if let Some(prep_node) = item.children().find(|n|n.has_tag_name("preparing") && ( n.has_attribute("minGrowthState") || n.has_attribute("maxGrowthState"))) {
                        if let Some(val) = prep_node.attribute("minGrowthState") {
                            item_struct.min_harvest = val.parse::<u8>().unwrap_or(item_struct.min_harvest);
                        }
                        if let Some(val) = prep_node.attribute("maxGrowthState") {
                            item_struct.max_harvest = val.parse::<u8>().unwrap_or(item_struct.max_harvest);
                        }
                    }
                    new_build.push(item_struct);
                }
                return new_build
            }
        }
    }
    fruits_from_base_game()
}

/// Build the weather from base game or included XML file
fn populate_weather(file_handle: &mut Box<dyn AbstractFileHandle>, env_base: Option<String>, env_in: Option<String>) -> MapEnvironment {
    if let Some(base_game_key) = env_base {
        return weather_from_base_game(&base_game_key)
    } else if let Some(file_name) = env_in {
        if let Ok(contents) = file_handle.as_text( file_name.as_str()) {
            if let Ok(tree) = roxmltree::Document::parse(&contents) {
                let mut weather_map:CropWeatherType = HashMap::new();
                let mut is_south = false;

                if let Some(node) = tree.descendants().find(|n|n.has_tag_name("latitude") && n.is_text()) {
                    if node.text().unwrap().parse::<f32>().unwrap_or(0.1) < 0.0 {
                        is_south = true;
                    }
                }

                for season in tree.descendants().filter(|n|n.has_tag_name("season") && n.has_attribute("name")) {
                    let mut min_temp:i8 = 127;
                    let mut max_temp:i8 = -127;

                    for variant in season.descendants().filter(|n|n.has_tag_name("variation") && n.has_attribute("minTemperature") && n.has_attribute("maxTemperature")) {
                        min_temp = std::cmp::min(
                            min_temp,
                            variant.attribute("minTemperature")
                                .unwrap().parse::<i8>()
                                .unwrap_or(127_i8) );
                        max_temp = std::cmp::max(
                            max_temp,
                            variant.attribute("maxTemperature")
                                .unwrap().parse::<i8>()
                                .unwrap_or(-127_i8) );
                    }

                    weather_map.insert(
                        season.attribute("name").unwrap().to_string(),
                        HashMap::from([
                            ("min".to_string(), min_temp),
                            ("max".to_string(), max_temp)
                        ])
                    );
                }
                
                return MapEnvironment( is_south, Some(weather_map.clone()) )
            }
        }
    }
    weather_from_base_game("mapUS")
}

/// Convert the read index into the real harvest index
/// 
/// This is +1 for all crops except olives (+2)
fn get_real_index(index : u8, name : &str) -> u8 {
    let test_index = if name == "olive" {
        index + 2
    } else {
        index + 1
    };
    ((test_index - 1) % 12) + 1
}

/// Populate crop growth from loaded XML file
/// 
/// This is only used when a map includes a growth file, the base game data is pre-calculated
fn populate_crop_growth(file_handle: &mut Box<dyn AbstractFileHandle>, growth : Option<String>, crop_builder: &[CropTypeStateBuilder]) -> Option<CropList> {
    let file_name = growth?;
    let contents = file_handle.as_text(&file_name).ok()?;
    let full_tree = roxmltree::Document::parse(&contents).ok()?;
    let seasonal_tree = full_tree.descendants().find(|n|n.has_tag_name("seasonal"))?;

    let mut crop_list:CropList = CropList::new();
    for fruit in seasonal_tree.descendants().filter(|n|n.has_tag_name("fruit")) {
        let fruit_name = fruit.attribute("name").unwrap_or("unknown").to_owned().to_lowercase();

        if SKIP_CROP_TYPES.contains(&fruit_name.as_str()) { continue }

        let builder = crop_builder.iter().find(|n|n.name == fruit_name);

        let Some(builder_unwrapped) = builder else { continue; };

        let mut crop_def = CropOutput {
            growth_time : builder_unwrapped.states,
            harvest_periods : vec![],
            plant_periods : vec![]
        };

        let mut possible_states:HashSet<u8> = HashSet::new();

        for period in fruit.children().filter(|n|n.has_tag_name("period") && n.has_attribute("index")) {
            let mut die_back_happened = false;
            let current_period_index = period.attribute("index").unwrap_or("0").parse::<u8>().unwrap_or(0_u8);

            if current_period_index == 0_u8 { continue; }

            if let Some(value) = period.attribute("plantingAllowed") {
                if value == "true" {
                    crop_def.plant_periods.push(current_period_index);
                }
            }

            let mut updates = period.children().filter(|n|n.has_tag_name("update")).peekable();

            if updates.peek().is_none() {
                // if we are already harvestable, we still are with no update
                for test_state in builder_unwrapped.min_harvest..=builder_unwrapped.max_harvest {
                    if possible_states.contains(&test_state) {
                        crop_def.harvest_periods.push(get_real_index(current_period_index, &fruit_name));
                    }
                }
            } else {
                // do the updates

                possible_states.clear();
                for update in updates {
                    if update.attribute("set").is_some() {
                        // if set range > growth_time, it's a regrow.
                        // if set range <= growth_time, it's die back
                        let range = decode_max_range(update.attribute("range"));
                        let new_value = decode_max_range(update.attribute("set"));
                        if range > new_value {
                            possible_states.insert(new_value);
                            die_back_happened  = true;
                        }
                    }
                    if ! die_back_happened {
                        if let Some(add_value) = update.attribute("add") {
                            let mut new_possible_max = decode_max_range(update.attribute("range"));
                            new_possible_max += add_value.parse::<u8>().unwrap_or(0_u8);
                            possible_states.insert(new_possible_max);
                        }
                    }
                }

                for test_state in builder_unwrapped.min_harvest..=builder_unwrapped.max_harvest {
                    if possible_states.contains(&test_state) {
                        crop_def.harvest_periods.push(get_real_index(current_period_index, &fruit_name));
                    }
                }
            }
        }
        if fruit_name == "poplar" { crop_def.harvest_periods = vec![1,2,3,4,5,6,7,8,9,10,11,12]; }
        crop_list.insert(fruit_name, crop_def);
    }
    Some(crop_list)
}

/// Get an included map support XML file
fn nullify_base_game_entry(xml_tree: &roxmltree::Document, tag : &str) -> Option<String> {
    match xml_tree.descendants().find(|n| n.has_tag_name(tag)) {
        Some(node) => match node.attribute("filename") {
            Some(val) => if val.starts_with("$data") { None } else { Some(val.to_string()) },
            None => None
        },
        None => None
    }
}

/// Get a map base game entry key
fn get_base_game_entry_key(xml_tree: &roxmltree::Document) -> Option<String> {
    match xml_tree.descendants().find(|n| n.has_tag_name("environment")) {
        Some(node) => match node.attribute("filename") {
            Some(val) => if val.starts_with("$data") {
                let re = Regex::new(r"(map[A-Z][A-Za-z]+)").unwrap();
                re.captures(val).map(|capture| capture.get(0).unwrap().as_str().to_owned())
            } else { None },
            None => None
        },
        None => None
    }
}