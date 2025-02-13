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
    common::populate_icon_result_file("FS25_Good_2.zip");
    common::populate_icon_result_file("PASS_Good_Simple_Mod.zip");
    common::populate_icon_result_file("PASS_Good_Simple_Mod");
    common::populate_icon_result_file("DETAIL_Samples.zip");
    common::populate_icon_result_file("DETAIL_Internal_Failures.zip");
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
fn good_22_folder() {
    common::icon_record_from_file("PASS_Good_Simple_Mod", false);
}


#[test]
fn good_25() {
    common::icon_record_from_file("FS25_Good.zip", false);
}

#[test]
fn good_25_2() {
    common::icon_record_from_file("FS25_Good_2.zip", false);
}

#[test]
fn good_22_detail() {
    common::icon_record_from_file("DETAIL_Samples.zip", false);
}

#[test]
fn internal_fail_22_detail() {
    common::icon_record_from_file("DETAIL_Internal_Failures.zip", false);
}


