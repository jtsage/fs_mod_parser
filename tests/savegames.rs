mod common;

/// Populate test file results
/// 
/// NOTE: this is destructive, and is a time saving measure - the
/// result files *must* be hand-checked for correctness if/when used
#[test]
#[ignore]
fn populate_saves() {
    common::populate_result_file("SAVEGAME_Single_Farm.zip");
    common::populate_result_file("SAVEGAME_Broken_Career.zip");
    common::populate_result_file("SAVEGAME_Broken_Farms.zip");
    common::populate_result_file("SAVEGAME_Broken_Placeable.zip");
    common::populate_result_file("SAVEGAME_Broken_Vehicles.zip");
    common::populate_result_file("SAVEGAME_Good.zip");
    common::populate_result_file("SAVEGAME_No_Career.zip");
    common::populate_result_file("SAVEGAME_No_Farms.zip");
    common::populate_result_file("SAVEGAME_No_Placeable.zip");
    common::populate_result_file("SAVEGAME_No_Vehicles.zip");
}

#[test]
fn save_tests() {
    common::record_from_file("SAVEGAME_Single_Farm.zip", false);
    common::record_from_file("SAVEGAME_Broken_Career.zip", false);
    common::record_from_file("SAVEGAME_Broken_Farms.zip", false);
    common::record_from_file("SAVEGAME_Broken_Placeable.zip", false);
    common::record_from_file("SAVEGAME_Broken_Vehicles.zip", false);
    common::record_from_file("SAVEGAME_Good.zip", false);
    common::record_from_file("SAVEGAME_No_Career.zip", false);
    common::record_from_file("SAVEGAME_No_Farms.zip", false);
    common::record_from_file("SAVEGAME_No_Placeable.zip", false);
    common::record_from_file("SAVEGAME_No_Vehicles.zip", false);
}
