use fs_mod_parser::parse_detail;


#[test]
fn direct_detail() {
    let actual = parse_detail("./tests/test_mods/DETAIL_Samples.zip", "xml/example-fill-unit.xml").unwrap();
    let json = serde_json::to_string(&actual).unwrap();

    assert!(json.len() > 13000 && json.len() < 14000);

    let actual = parse_detail("./tests/test_mods/DETAIL_Samples.zip", "xml/place-husbandry.xml").unwrap();
    let json = serde_json::to_string(&actual).unwrap();

    assert!(json.len() > 13000 && json.len() < 14000);
}