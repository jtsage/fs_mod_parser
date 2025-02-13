mod common;

/// Populate test file results
/// 
/// NOTE: this is destructive, and is a time saving measure - the
/// result files *must* be hand-checked for correctness if/when used
#[test]
#[ignore]
fn populate_failures() {
    common::populate_result_file("0FAILURE_Starts_With_Digit.zip");
    common::populate_result_file("FAIL_Contains_EXE.zip");
    common::populate_result_file("FAILURE_Bad_ModDesc_CRC.zip");
    common::populate_result_file("FAILURE_Broken_Zip_File.zip");
    common::populate_result_file("FAILURE_Really_Malformed_ModDesc.zip");
    common::populate_result_file("FAILURE_Copied_Mod (2).zip");
    common::populate_result_file("FAILURE_Garbage_File.txt");
    common::populate_result_file("FAILURE_Missing_ModDesc.zip");
    common::populate_result_file("FAILURE_No_DescVersion.zip");
    common::populate_result_file("FAILURE_Invalid_Folder");
    common::populate_result_file("FAILURE_Invalid_ModDesc");
    common::populate_result_file("VARIANT_Mod_Pack.zip");
}

#[test]
fn starts_with_digit() {
    common::record_from_file("0FAILURE_Starts_With_Digit.zip", false);
}

#[test]
fn contains_exe() {
    common::record_from_file("FAIL_Contains_EXE.zip", false);
}

#[test]
fn bad_crc() {
    common::record_from_file("FAILURE_Bad_ModDesc_CRC.zip", false);
}

#[test]
fn broken_zip() {
    common::record_from_file("FAILURE_Broken_Zip_File.zip", false);
}

#[test]
fn malformed_mod_desc() {
    common::record_from_file("FAILURE_Really_Malformed_ModDesc.zip", false);
}

#[test]
fn invalid_copy() {
    common::record_from_file("FAILURE_Copied_Mod (2).zip", false);
}

#[test]
fn missing_mod_desc() {
    common::record_from_file("FAILURE_Missing_ModDesc.zip", false);
}

#[test]
fn missing_desc_version() {
    common::record_from_file("FAILURE_No_DescVersion.zip", false);
}

#[test]
fn garbage_file() {
    common::record_from_file("FAILURE_Garbage_File.txt", false);
}

#[test]
fn invalid_mod_desc() {
    common::record_from_file("FAILURE_Invalid_ModDesc", false);
}

#[test]
fn invalid_folder() {
    common::record_from_file("FAILURE_Invalid_Folder", false);
}

#[test]
fn mod_pack() {
    common::record_from_file("VARIANT_Mod_Pack.zip", false);
}
