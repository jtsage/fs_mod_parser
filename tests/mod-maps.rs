mod common;

/// Populate test file results
/// 
/// NOTE: this is destructive, and is a time saving measure - the
/// result files *must* be hand-checked for correctness if/when used
#[test]
#[ignore]
fn populate_maps() {
    common::populate_result_file("MAP_AddedCrops.zip");
    common::populate_result_file("MAP_CustomGrowth.zip");
    common::populate_result_file("MAP_CustomGrowthAndEnvironment.zip");
    common::populate_icon_result_file("MAP_NoCustoms.zip");
}

#[test]
fn all_base_game() {
    common::icon_record_from_file("MAP_NoCustoms.zip", false);
}

#[test]
fn growth_and_weather() {
    common::record_from_file("MAP_CustomGrowthAndEnvironment.zip", false);
}

#[test]
fn just_growth() {
    common::record_from_file("MAP_CustomGrowth.zip", false);
}

#[test]
fn added_crops() {
    common::record_from_file("MAP_AddedCrops.zip", false);
}

