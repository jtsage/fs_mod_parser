//! Map file parsing
//! 
//! Reads crop data, weather data, and the map overview image
use std::collections::HashMap;
use regex::Regex;
use crate::shared::convert_map_image;
use crate::shared::structs::ModRecord;
use crate::shared::files::AbstractFileHandle;

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
fn bool_array_to_vector(input_array:[bool;12]) -> Vec<u8> {
    input_array.iter().enumerate().map(|(i,v)| if *v { i as u8 + 1_u8 } else {0_u8}).filter(|n| *n!=0_u8 ).collect()
}

// Convert base game crop data to usable version
fn crops_from_base_game() -> std::option::Option<Vec<CropOutput>> {
    let mut crop_list:Vec<CropOutput> = vec![];

    for crop in &BG_CROPS {
        crop_list.push(CropOutput {
            name : crop.name.to_owned(),
            growth_time : crop.growth_time,
            harvest_periods : bool_array_to_vector(crop.harvest_periods),
            plant_periods : bool_array_to_vector(crop.plant_periods),
        });
    }
    Some(crop_list)
}

// Convert base game fruit types to usable builders
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

// Return basegame weather by key
fn weather_from_base_game(mod_record : &mut ModRecord, base_game_key:Option<String>) {
    let mut weather_map:CropWeatherType = HashMap::new();

    for key in BG_CROP_WEATHER {
        if base_game_key == Some(key.name.to_string()) {
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
        mod_record.mod_desc.crop_weather = Some(weather_map.clone());
    } else {
        mod_record.mod_desc.crop_weather = None;
    }
}

/// Read basic details about the map
/// 
/// Includes weather, crops, if it's southern, and the map image
pub fn read_map_basics(mod_record : &mut ModRecord, file_handle: &mut Box<dyn AbstractFileHandle> ) {
    if mod_record.mod_desc.map_config_file.is_none() { return; }

    let (fruits, growth, env_in, env_base) = match file_handle.as_text(mod_record.mod_desc.map_config_file.clone().unwrap().as_str()) {
        Ok(contents) => {
            match roxmltree::Document::parse(&contents) {
                Ok(map_config_tree) => {
                    if let Some(filename) = map_config_tree.root_element().attribute("imageFilename") {
                        let mut value_string = filename.to_string();
                        if let Some(index) = value_string.find(".png") {
                            value_string.replace_range(index..value_string.len(), ".dds");
                        }
                        if mod_record.file_detail.image_dds.contains(&value_string) {
                            if let Ok(content) = file_handle.as_bin(&value_string) {
                                mod_record.mod_desc.map_image = convert_map_image(content);
                            }
                        }
                    }

                    (
                        nullify_base_game_entry(&map_config_tree, "fruitTypes"),
                        nullify_base_game_entry(&map_config_tree, "growth"),
                        nullify_base_game_entry(&map_config_tree, "environment"),
                        get_base_game_entry_key(&map_config_tree)
                    )
                },
                Err(..) => (None, None, None, None)
            }
        },
        Err(..) => (None, None, None, None)
    };

    if env_base.is_some() {
        weather_from_base_game(mod_record, env_base);
    } else {
        match &env_in {
            Some(file_name) => match file_handle.as_text( file_name) {
                Ok(contents) => {
                    match roxmltree::Document::parse(&contents) {
                        Ok(tree) => {
                            let mut weather_map:CropWeatherType = HashMap::new();

                            if let Some(node) = tree.descendants().find(|n|n.has_tag_name("latitude") && n.is_text()) {
                                if node.text().unwrap().parse::<f32>().unwrap_or(0.1) < 0.0 {
                                    mod_record.mod_desc.map_is_south = true;
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
                            mod_record.mod_desc.crop_weather = Some(weather_map.clone());
                        },
                        Err(..) => { weather_from_base_game(mod_record, Some("mapUS".to_string())); }
                    }
                },
                Err(..) => { weather_from_base_game(mod_record, Some("mapUS".to_string())); }
            },
            None => { weather_from_base_game(mod_record, Some("mapUS".to_string())); }
        }
    }

    if growth.is_none() {
        mod_record.mod_desc.crop_info = crops_from_base_game();
        return;
    }

    let crop_builder:Vec<CropTypeStateBuilder> = match &fruits {
        Some(file_name) => match file_handle.as_text( file_name) {
            Ok(contents) => {
                match roxmltree::Document::parse(&contents) {
                    Ok(tree) => {
                        let mut new_build:Vec<CropTypeStateBuilder> = vec![];

                        for item in tree.descendants().filter(|n|n.has_tag_name("fruitType")) {
                            let item_name = item.attribute("name").unwrap_or("unknown").to_owned();

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
        Some(file_name) => match file_handle.as_text( file_name) {
            Ok(contents) => {
                match roxmltree::Document::parse(&contents) {
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
                                let current_period_index = period.attribute("index").unwrap().parse::<u8>().unwrap_or(0_u8);

                                if current_period_index == 0_u8 { continue; }

                                if let Some(value) = period.attribute("plantingAllowed") {
                                    if value == "true" {
                                        crop_def.plant_periods.push(current_period_index);
                                    }
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
                                        if update.attribute("set").is_some() {
                                            // if set range > growth_time, it's a regrow.
                                            // if set range <= growth_time, it's die back
                                            let range = decode_max_range(update.attribute("range"));
                                            if range <= builder_unwrapped.states {
                                                last_maximum_state = range;
                                                die_back_happened  = true;
                                            }
                                        }
                                        if ! die_back_happened {
                                            if let Some(add_value) = update.attribute("add") {
                                                let mut new_possible_max = decode_max_range(update.attribute("range"));
                                                new_possible_max += add_value.parse::<u8>().unwrap_or(0_u8);
                                                last_maximum_state = std::cmp::max(last_maximum_state, new_possible_max);
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
                    Err(..) => { mod_record.mod_desc.crop_info = crops_from_base_game(); }
                }
            },
            Err(..) => { mod_record.mod_desc.crop_info = crops_from_base_game(); }
        },
        None => { mod_record.mod_desc.crop_info = crops_from_base_game(); }
    }
}


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

/// Get an included map support XML file
pub fn nullify_base_game_entry(xml_tree: &roxmltree::Document, tag : &str) -> Option<String> {
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