//! Map file parsing
//! 
//! Reads crop data, weather data, and the map overview image
use std::collections::{HashMap, HashSet};
use crate::shared::{normalize_image_file, convert_map_image};
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
    use crate::shared::files::AbstractNull;

    #[test]
    fn base_game_weather_invalid_id() {
        let weather = weather_from_base_game("foo");

        assert_eq!(weather.0, false);
        assert!(weather.1.is_none());
    }

    #[test]
    fn test_array_convert() {
        let input:[bool;12] = [true, false, false, true, true, false, false, true, true, false, false, true];
        let output = bool_array_to_vector(input);
        let expected: Vec<u8> = vec![1,4,5,8,9,12];
        assert_eq!(output, expected);
    }

    #[test]
    fn test_game_entry_key_valid_us() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="$data/maps/mapUS/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, Some("mapUS".to_string()));
    }

    #[test]
    fn test_game_entry_key_valid_fr() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="$data/maps/mapFR/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, Some("mapFR".to_string()));
    }

    #[test]
    fn test_game_entry_key_valid_alpine() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="$data/maps/mapAlpine/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, Some("mapAlpine".to_string()));
    }

    #[test]
    fn test_game_entry_key_valid_unknown() {
        let document = roxmltree::Document::parse(r#"<map><environment filename="$data/maps/mapBullshit/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, Some("mapUS".to_string()));
    }

    #[test]
    fn test_game_entry_key_missing_filename() {
        let document = roxmltree::Document::parse(r#"<map><environment name="$data/maps/mapBullshit/environment.xml" /></map>"#).unwrap();
        let result = get_base_game_entry_key(&document);
        assert_eq!(result, Some("mapUS".to_string()));
    }

    #[test]
    fn test_game_null_entry_key_missing_filename() {
        let document = roxmltree::Document::parse(r#"<map><environment name="$data/maps/mapBullshit/environment.xml" /></map>"#).unwrap();
        let result = nullify_base_game_entry(&document, "environment");
        assert_eq!(result, None);
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

    #[test]
    fn test_range() {
        // Invalid options
        assert_eq!(decode_max_range(Some("1-4-8")), 8_u8);
        assert_eq!(decode_max_range(Some("1-")), 0_u8);
        assert_eq!(decode_max_range(Some("-6")), 6_u8);
        // Valid options
        assert_eq!(decode_max_range(Some("1-4")), 4_u8);
        assert_eq!(decode_max_range(Some("3")), 3_u8);
        assert_eq!(decode_max_range(None), 0_u8);
    }

    #[test]
    fn missing_overview() {
        let minimum_xml = r#"<map></map>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut file_handle:Box<dyn AbstractFileHandle> = Box::new(AbstractNull::new().unwrap());
        let result = process_overview(&minimum_doc, &mut file_handle);
        assert_eq!(result, None);
    }
}

/// Convert array of booleans to vector of indexes
#[expect(clippy::cast_possible_truncation)]
fn bool_array_to_vector(input_array:[bool;12]) -> Vec<u8> {
    input_array.iter().enumerate().map(|(i,v)| if *v { i as u8 + 1_u8 } else {0_u8}).filter(|n| *n!=0_u8 ).collect()
}

/// Convert base game crop data to usable version
fn crops_from_base_game() -> CropList {
    let mut crop_list = CropList::new();

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

    if let Some(found_weather) = BG_CROP_WEATHER.iter().find(|n|n.0 == base_game_key) {
        for season in &found_weather.1 {
            weather_map.insert(
                season.name.to_owned(),
                HashMap::from([
                    (String::from("min"), season.min),
                    (String::from("max"), season.max)
                ])
            );
        }
    }

    if weather_map.is_empty() { 
        MapEnvironment( false, None )
    } else {
        MapEnvironment( false, Some(weather_map.clone()) )
    }
}


/// Map file information
struct MapFiles {
    /// fruitTypes file
    pub fruits : Option<String>,
    /// growth file
    pub growth : Option<String>,
    /// included environment
    pub env_in : Option<String>,
    /// base game environment key
    pub env_base : Option<String>
}

impl MapFiles {
    /// Make new map files struct
    #[must_use]
    #[inline]
    fn new() -> Self {
        MapFiles {
            fruits : None,
            growth : None,
            env_in : None,
            env_base : None,
        }
    }
}
/// Read basic details about the map
/// 
/// Includes weather, crops, if it's southern, and the map image
pub fn read_map_basics(mod_record : &mut ModRecord, file_handle: &mut Box<dyn AbstractFileHandle> ) {
    let Some(map_config_file_name) = &mod_record.mod_desc.map_config_file else {
        return;
    };

    let mut map_config = MapFiles::new();

    if let Ok(contents) = file_handle.as_text(map_config_file_name) {
        if let Ok(map_config_tree) = roxmltree::Document::parse(&contents) {
            mod_record.mod_desc.map_image = process_overview(&map_config_tree, file_handle);

            map_config.fruits = nullify_base_game_entry(&map_config_tree, "fruitTypes");
            map_config.growth = nullify_base_game_entry(&map_config_tree, "growth");
            map_config.env_in = nullify_base_game_entry(&map_config_tree, "environment");
            map_config.env_base = get_base_game_entry_key(&map_config_tree);
        }
    }

    mod_record.mod_desc.map_custom_crop = map_config.fruits.is_some();
    mod_record.mod_desc.map_custom_env  = map_config.env_in.is_some();
    mod_record.mod_desc.map_custom_grow = map_config.growth.is_some();

    let this_map_environment = populate_weather(file_handle, map_config.env_base, map_config.env_in);
    mod_record.mod_desc.map_is_south = this_map_environment.0;
    mod_record.mod_desc.crop_weather = this_map_environment.1;

    if map_config.growth.is_none() {
        mod_record.mod_desc.crop_info = crops_from_base_game();
        return;
    }

    let crop_builder = populate_crop_builder(file_handle, map_config.fruits);

    match populate_crop_growth(file_handle, map_config.growth, &crop_builder) {
        Some(value) => mod_record.mod_desc.crop_info = value,
        None => mod_record.mod_desc.crop_info = crops_from_base_game()
    }

}

/// Decode a range argument and get the maximum from it
#[inline]
fn decode_max_range(range:Option<&str>) -> u8 {
    if let Some(value) = range {
        if value.contains('-') {
            if let Some(split_value) = value.split('-').last() {
                return split_value.parse::<u8>().unwrap_or(0_u8);
            }
        }
        return value.parse::<u8>().unwrap_or(0_u8);
    }
    0
}

/// Load and convert the overview image
/// 
/// Automatically crops to the center 1/4 of the image that contains the map
/// and constrains the size to 512x512px
#[inline]
fn process_overview(xml_tree: &roxmltree::Document, file_handle: &mut Box<dyn AbstractFileHandle>) -> Option<String> {
    let image_file = normalize_image_file(xml_tree.root_element().attribute("imageFilename"));

    if let Some(filename) = image_file.local_file {
        if let Ok(content) = file_handle.as_bin(&filename) {
            return convert_map_image(content)
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
                        max_harvest : get_crop_attribute(&item, "harvest", "maxHarvestingGrowthState", 20_u8),
                        min_harvest : get_crop_attribute(&item, "harvest", "minHarvestingGrowthState", 20_u8),
                        states      : get_crop_attribute(&item, "growth", "numGrowthStates", 20_u8),
                    };

                    item_struct.min_harvest = get_crop_attribute(&item, "preparing", "minGrowthState", item_struct.min_harvest);
                    item_struct.max_harvest = get_crop_attribute(&item, "preparing", "maxGrowthState", item_struct.max_harvest);

                    new_build.push(item_struct);
                }
                return new_build
            }
        }
    }
    fruits_from_base_game()
}

#[inline]
/// Get a crop attribute from a tag
fn get_crop_attribute(xml_node: &roxmltree::Node, tag_name: &str, attr_name : &str, default : u8) -> u8 {
    if let Some(node) = xml_node.children().find(|n|n.has_tag_name(tag_name)) {
        if let Some(value) = node.attribute(attr_name) {
            return value.parse::<u8>().unwrap_or(default);
        }
    }
    default
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

                if let Some(node) = tree.descendants().find(|n|n.has_tag_name("latitude")) {
                    if node.text().unwrap_or("0.1").parse::<f32>().unwrap_or(0.1) < 0.0 {
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
                                .unwrap_or("127")
                                .parse::<i8>()
                                .unwrap_or(127_i8) );
                        max_temp = std::cmp::max(
                            max_temp,
                            variant.attribute("maxTemperature")
                                .unwrap_or("-127")
                                .parse::<i8>()
                                .unwrap_or(-127_i8) );
                    }

                    weather_map.insert(
                        season.attribute("name").unwrap_or("invalid").to_owned(),
                        HashMap::from([
                            (String::from("min"), min_temp),
                            (String::from("max"), max_temp)
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

    let mut crop_list = CropList::new();
    for fruit in seasonal_tree.descendants().filter(|n|n.has_tag_name("fruit")) {
        let fruit_name = fruit.attribute("name").unwrap_or("unknown").to_owned().to_lowercase();

        if SKIP_CROP_TYPES.contains(&fruit_name.as_str()) { continue }

        let builder = crop_builder.iter().find(|n|n.name == fruit_name);

        let Some(builder_unwrapped) = builder else { continue; };

        let mut crop_def = CropOutput::new(builder_unwrapped.states);

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
#[inline]
fn nullify_base_game_entry(xml_tree: &roxmltree::Document, tag : &str) -> Option<String> {
    match xml_tree.descendants().find(|n| n.has_tag_name(tag)) {
        Some(node) => match node.attribute("filename") {
            Some(val) => if val.starts_with("$data") { None } else { Some(val.to_owned()) },
            None => None
        },
        None => None
    }
}

/// Get a map base game entry key
#[inline]
fn get_base_game_entry_key(xml_tree: &roxmltree::Document) -> Option<String> {
    if let Some(node) = xml_tree.descendants().find(|n| n.has_tag_name("environment")) {
        if let Some(filename) = node.attribute("filename") {
            return match filename {
                x if ! x.starts_with("$data") => None,
                x if x.contains("mapUS") => Some(String::from("mapUS")),
                x if x.contains("mapFR") => Some(String::from("mapFR")),
                x if x.contains("mapAlpine") => Some(String::from("mapAlpine")),
                // starts with data, but unrecognized.  default to US map.
                _ => Some(String::from("mapUS"))
            }
        }
    }
    // xml element exists, but no filename field
    // this is invalid for a mod, but let's fallback to mapUS anyway
    Some(String::from("mapUS"))
}