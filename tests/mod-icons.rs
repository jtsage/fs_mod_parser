mod common;

/// Populate test file results
/// 
/// NOTE: this is destructive, and is a time saving measure - the
/// result files *must* be hand-checked for correctness if/when used
#[test]
#[ignore]
fn populate_icons() {
    common::populate_icon_result_file("WARNING_Icon_Not_Found.zip");
    common::populate_icon_result_file("FS25_Good.zip");
    common::populate_icon_result_file("PASS_Good_Simple_Mod.zip");
}

#[test]
fn icon_missing() {
    common::icon_record_from_file("WARNING_Icon_Not_Found.zip", false);
}

#[test]
fn good_22() {
    common::icon_record_from_file("PASS_Good_Simple_Mod.zip", false);
}


#[test]
fn good_25() {
    common::icon_record_from_file("FS25_Good.zip", false);
}
