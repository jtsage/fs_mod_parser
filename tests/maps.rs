use fs_mod_parser::parse_mod;
use fs_mod_parser::maps::structs::CropOutput;
use assert_json_diff::assert_json_include;
use serde_json::json;

#[test]
fn test_custom_environment_and_growth() {

    let result = parse_mod("./tests/test_mods/MAP_CustomGrowthAndEnvironment.zip");

    let expect_env = json!({"modDesc" : { "cropWeather": {
        "spring": { "min": 6, "max": 18 },
        "summer": { "min": 13, "max": 34 },
        "winter": { "min": -35, "max": 30 },
        "autumn": { "min": 5, "max": 25 }
    }}});

    assert_json_include!(actual : json!(result), expected : expect_env);

    assert_eq!(result.mod_desc.map_custom_crop, false);
    assert_eq!(result.mod_desc.map_custom_grow, true);
    assert_eq!(result.mod_desc.map_custom_env, true);
    assert_eq!(result.mod_desc.map_is_south, true);

    let mut crop_info_vec = result.mod_desc.crop_info;

    assert_eq!(crop_info_vec.get("wheat"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![5,6],
        plant_periods   : vec![8,9],
    }).as_ref(), "wheat");

    assert_eq!(crop_info_vec.get("barley"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![4,5],
        plant_periods   : vec![7,8],
    }).as_ref(), "barley");

    assert_eq!(crop_info_vec.get("canola"), Some(CropOutput{
        growth_time     : 9,
        harvest_periods : vec![5,6],
        plant_periods   : vec![6,7],
    }).as_ref(), "canola");

    assert_eq!(crop_info_vec.get("oat"), Some(CropOutput{
        growth_time     : 5,
        harvest_periods : vec![5,6],
        plant_periods   : vec![1,2],
    }).as_ref(), "oat");

    assert_eq!(crop_info_vec.get("maize"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8,9,10,11,12],
        plant_periods   : vec![2,3],
    }).as_ref(), "maize");

    assert_eq!(crop_info_vec.get("sunflower"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,2],
    }).as_ref(), "sunflower");

    assert_eq!(crop_info_vec.get("soybean"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8,9,10,11],
        plant_periods   : vec![2,3,4,5],
    }).as_ref(), "soybean");

    assert_eq!(crop_info_vec.get("potato"), Some(CropOutput{
        growth_time     : 6,
        harvest_periods : vec![6,7],
        plant_periods   : vec![1,2],
    }).as_ref(), "potato");

    assert_eq!(crop_info_vec.get("sugarbeet"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,2],
    }).as_ref(), "sugarbeet");

    assert_eq!(crop_info_vec.get("sugarcane"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,2],
    }).as_ref(), "sugarcane");

    assert_eq!(crop_info_vec.get("cotton"), Some(CropOutput{
        growth_time     : 9,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,12],
    }).as_ref(), "cotton");

    assert_eq!(crop_info_vec.get("sorghum"), Some(CropOutput{
        growth_time     : 5,
        harvest_periods : vec![6,7],
        plant_periods   : vec![2,3],
    }).as_ref(), "sorghum");

    assert_eq!(crop_info_vec.get("grape"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![7,8],
        plant_periods   : vec![1,2,3],
    }).as_ref(), "grape");

    assert_eq!(crop_info_vec.get("olive"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8],
        plant_periods   : vec![1,2,3,4],
    }).as_ref(), "olive");

    assert_eq!(crop_info_vec.get("poplar"), Some(CropOutput{
        growth_time     : 14,
        harvest_periods : vec![1,2,3,4,5,6,7,8,9,10,11,12],
        plant_periods   : vec![1,2,3,4,5,6],
    }).as_ref(), "poplar");

    assert_eq!(crop_info_vec.get("grass"), Some(CropOutput{
        growth_time     : 4,
        harvest_periods : vec![2,3,4,5,6,7,8,9,10,11,12,1],
        plant_periods   : vec![1,2,3,4,5,6,7,8,9],
    }).as_ref(), "grass");

    assert_eq!(crop_info_vec.get("oilseedradish"), Some(CropOutput{
        growth_time     : 2,
        harvest_periods : vec![2,3,4,5,6,7,8,9,10,11,12,1],
        plant_periods   : vec![1,2,3,4,5,6,7,8],
    }).as_ref(), "oilseedradish");

}

#[test]
fn test_custom_growth() {

    let result = parse_mod("./tests/test_mods/MAP_CustomGrowth.zip");

    let mut crop_info_vec = result.mod_desc.crop_info;

    assert_eq!(result.mod_desc.map_custom_crop, false);
    assert_eq!(result.mod_desc.map_custom_grow, true);
    assert_eq!(result.mod_desc.map_custom_env, false);
    assert_eq!(result.mod_desc.map_is_south, false);

    assert_eq!(crop_info_vec.get("wheat"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![5,6],
        plant_periods   : vec![2,3,7,8],
    }).as_ref(), "wheat");

    assert_eq!(crop_info_vec.get("barley"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![4],
        plant_periods   : vec![2,3,7,8],
    }).as_ref(), "barley");

    assert_eq!(crop_info_vec.get("canola"), Some(CropOutput{
        growth_time     : 9,
        harvest_periods : vec![5,6,7,8],
        plant_periods   : vec![3,6,7],
    }).as_ref(), "canola");

    assert_eq!(crop_info_vec.get("oat"), Some(CropOutput{
        growth_time     : 5,
        harvest_periods : vec![5,6],
        plant_periods   : vec![1,2],
    }).as_ref(), "oat");

    assert_eq!(crop_info_vec.get("maize"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8,9],
        plant_periods   : vec![2,3],
    }).as_ref(), "maize");

    assert_eq!(crop_info_vec.get("sunflower"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,2],
    }).as_ref(), "sunflower");

    assert_eq!(crop_info_vec.get("soybean"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8,9],
        plant_periods   : vec![3,4],
    }).as_ref(), "soybean");

    assert_eq!(crop_info_vec.get("potato"), Some(CropOutput{
        growth_time     : 6,
        harvest_periods : vec![7,8],
        plant_periods   : vec![2,3],
    }).as_ref(), "potato");

    assert_eq!(crop_info_vec.get("sugarbeet"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,2],
    }).as_ref(), "sugarbeet");

    assert_eq!(crop_info_vec.get("sugarcane"), Some(CropOutput{
        growth_time     : 8,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,2],
    }).as_ref(), "sugarcane");

    assert_eq!(crop_info_vec.get("cotton"), Some(CropOutput{
        growth_time     : 9,
        harvest_periods : vec![8,9],
        plant_periods   : vec![1,12],
    }).as_ref(), "cotton");

    assert_eq!(crop_info_vec.get("sorghum"), Some(CropOutput{
        growth_time     : 5,
        harvest_periods : vec![6,7],
        plant_periods   : vec![2,3],
    }).as_ref(), "sorghum");

    assert_eq!(crop_info_vec.get("grape"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![7,8],
        plant_periods   : vec![1,2,3],
    }).as_ref(), "grape");

    assert_eq!(crop_info_vec.get("olive"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8],
        plant_periods   : vec![1,2,3,4],
    }).as_ref(), "olive");

    assert_eq!(crop_info_vec.get("poplar"), Some(CropOutput{
        growth_time     : 14,
        harvest_periods : vec![1,2,3,4,5,6,7,8,9,10,11,12],
        plant_periods   : vec![1,2,3,4,5,6],
    }).as_ref(), "poplar");

    assert_eq!(crop_info_vec.get("grass"), Some(CropOutput{
        growth_time     : 4,
        harvest_periods : vec![2,3,4,5,6,7,8,9,10,11,12,1],
        plant_periods   : vec![1,2,3,4,5,6,7,8,9],
    }).as_ref(), "grass");

    assert_eq!(crop_info_vec.get("oilseedradish"), Some(CropOutput{
        growth_time     : 2,
        harvest_periods : vec![2,3,4,5,6,7,8,9,10,11,12,1],
        plant_periods   : vec![1,2,3,4,5,6,7,8],
    }).as_ref(), "oilseedradish");

}


#[test]
fn test_no_customs() {

    let result = parse_mod("./tests/test_mods/MAP_NoCustoms.zip");

    let expect_env = json!({"modDesc" : { "cropWeather": {
        "spring": { "min": 6, "max": 18 },
        "summer": { "min": 13, "max": 34 },
        "winter": { "min": -11, "max": 10 },
        "autumn": { "min": 5, "max": 25 }
    }}});

    assert_json_include!(actual : json!(result), expected : expect_env);

    assert_eq!(result.mod_desc.map_custom_crop, false);
    assert_eq!(result.mod_desc.map_custom_grow, false);
    assert_eq!(result.mod_desc.map_custom_env, false);
    assert_eq!(result.mod_desc.map_is_south, false);
    assert_eq!(result.mod_desc.crop_info.is_empty(), false);
    assert_eq!(result.mod_desc.crop_info.len(), 17);
}

#[test]
fn test_added_crops() {

    let result = parse_mod("./tests/test_mods/MAP_AddedCrops.zip");

    let expect_env = json!({"modDesc" : { "cropWeather": {
        "spring": { "min": 6, "max": 18 },
        "summer": { "min": 13, "max": 34 },
        "winter": { "min": -11, "max": 10 },
        "autumn": { "min": 5, "max": 25 }
    }}});

    assert_json_include!(actual : json!(result), expected : expect_env);

    assert_eq!(result.mod_desc.map_custom_crop, true);
    assert_eq!(result.mod_desc.map_custom_grow, true);
    assert_eq!(result.mod_desc.map_custom_env, true);
    assert_eq!(result.mod_desc.map_is_south, false);

    assert_eq!(result.mod_desc.crop_info.is_empty(), false);
    assert_eq!(result.mod_desc.crop_info.len(), 20);

    let mut crop_info_vec = result.mod_desc.crop_info;

    assert_eq!(crop_info_vec.get("alfalfa"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![2,3,4,5,6,7,8],
        plant_periods   : vec![1,2,3,4,5,6,7,8,9],
    }).as_ref(), "alfalfa");

    assert_eq!(crop_info_vec.get("clover"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![2,3,4,5,6,7,8],
        plant_periods   : vec![1,2,3,4,5,6,7,8,9],
    }).as_ref(), "clover");

    assert_eq!(crop_info_vec.get("silage_corn"), Some(CropOutput{
        growth_time     : 7,
        harvest_periods : vec![8,9,10,11],
        plant_periods   : vec![2,3],
    }).as_ref(), "silage_corn");
}