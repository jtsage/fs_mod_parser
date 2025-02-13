mod common;

/// Populate test file results
/// 
/// NOTE: this is destructive, and is a time saving measure - the
/// result files *must* be hand-checked for correctness if/when used
#[test]
#[ignore]
fn populate_old() {
    common::populate_icon_result_file("OLD_FS11.zip");
    common::populate_icon_result_file("OLD_FS13.zip");
    common::populate_icon_result_file("OLD_FS15.zip");
    common::populate_icon_result_file("OLD_FS17.zip");
    common::populate_icon_result_file("OLD_FS19.zip");
}

#[test]
fn fs11() {
    common::icon_record_from_file("OLD_FS11.zip", false);
}

#[test]
fn fs13() {
    common::icon_record_from_file("OLD_FS13.zip", false);
}

#[test]
fn fs15() {
    common::icon_record_from_file("OLD_FS15.zip", false);
}

#[test]
fn fs17() {
    common::icon_record_from_file("OLD_FS17.zip", false);
}

#[test]
fn fs19() {
    common::icon_record_from_file("OLD_FS19.zip", false);
}
