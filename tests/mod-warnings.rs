mod common;

/// Populate test file results
/// 
/// NOTE: this is destructive, and is a time saving measure - the
/// result files *must* be hand-checked for correctness if/when used
#[test]
#[ignore]
fn populate_warnings() {
    common::populate_result_file("WARNING_Size_Test_Mod.zip");
    common::populate_result_file("WARNING_Fake_Cracked_DLC.zip");
    common::populate_result_file("WARNING_Malicious_Code.zip");
    common::populate_result_file("WARNING_No_Version.zip");
    common::populate_result_file("PASS_Invalid_XML.zip");
    common::populate_result_file("WARNING_L10n.zip");
}

#[test]
fn size_test() {
    common::record_from_file("WARNING_Size_Test_Mod.zip", false);
}

#[test]
fn piracy() {
    common::record_from_file("WARNING_Fake_Cracked_DLC.zip", false);
}


#[test]
fn malicious_lua() {
    common::record_from_file("WARNING_Malicious_Code.zip", false);
}

#[test]
fn missing_version() {
    common::record_from_file("WARNING_No_Version.zip", false);
}

#[test]
fn missing_l10n() {
    common::record_from_file("WARNING_L10n.zip", false);
}

#[test]
fn recoverable_xml_issues() {
    common::record_from_file("PASS_Invalid_XML.zip", false);
}

