use std::path::Path;
use fs_mod_parser::mod_basic::parser;
use fs_mod_parser::shared::structs::{ModBadges, ModRecord};
use assert_json_diff::{assert_json_eq, assert_json_include};
use serde_json::json;

#[test]
fn check_json_mod_record() {
	let mut mod_record = ModRecord::new(Path::new("foo.txt"), false);

	mod_record.update_badges();

	let expected = json!({
		"badgeArray": [
			"noMP"
		],
		"canNotUse": true,
		"currentCollection": "",
		"detailIconLoaded" : false,
		"fileDetail": {
			"copyName": null,
			"extraFiles": [],
			"fileDate": "",
			"fileSize": 0,
			"fullPath": "foo.txt",
			"i3dFiles": [],
			"imageDDS": [],
			"imageNonDDS": [],
			"isFolder": false,
			"isModPack": false,
			"isSaveGame": false,
			"pngTexture": [],
			"shortName": "foo",
			"spaceFiles": [],
			"tooBigFiles": [],
			"zipFiles": [],
		},
		"issues": [],
		"includeDetail"   : null,
        "includeSaveGame" : null,
		"l10n": {
			"description": {
				"en": "--"
			},
			"title": {
				"en": "--"
			},
		},
		"md5Sum": null,
		"modDesc": {
			"actions": {},
			"binds": {},
			"author": "--",
			"scriptFiles": 0,
			"storeItems": 0,
			"cropInfo": null,
			"cropWeather": null,
			"depend": [],
			"descVersion": 0,
			"iconFileName": null,
			"iconImage": null,
			"mapConfigFile": null,
			"mapCustomEnv": false,
			"mapCustomCrop": false,
			"mapCustomGrow": false,
			"mapIsSouth": false,
			"mapImage": null,
			"multiPlayer": false,
			"version": "--"
		},
		"uuid": "4fd8cc85ca9eebd2fa3c550069ce2846"
	});

	assert_json_eq!(json!(mod_record), expected);
	
}

#[test]
fn check_json_badges() {
	let mod_badges = ModBadges {
		broken   : true,
		folder   : true,
		malware  : true,
		no_mp    : true,
		notmod   : true,
		pconly   : true,
		problem  : true,
		savegame : true,
	};

	let expected = json!([
		"broken",
		"folder",
		"malware",
		"noMP",
		"notmod",
		"pconly",
		"problem",
		"savegame"
	]);

	assert_json_eq!(json!(mod_badges), expected)
}

#[test]
fn simple_good_mod() {
	let test_file_path = Path::new("./tests/test_mods/PASS_Good_Simple_Mod.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);

	assert_eq!(mod_record.can_not_use, false);
	assert_eq!(mod_record.issues.len(), 0);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : false,
		savegame : false,
	});

	assert_ne!(mod_record.mod_desc.icon_image, None);

	let expected = json!({
		"badgeArray": [],
		"canNotUse": false,
		"currentCollection": "",
		"fileDetail": {
			"copyName": null,
			"extraFiles": [],
			"fileSize": 12530,
			"i3dFiles": [],
			"imageDDS": [
				"modIcon.dds"
			],
			"imageNonDDS": [],
			"isFolder": false,
			"isSaveGame": false,
			"isModPack": false,
			"pngTexture": [],
			"shortName": "PASS_Good_Simple_Mod",
			"spaceFiles": [],
			"tooBigFiles": [],
			"zipFiles": []
		},
		"issues": [],
		"l10n": {
			"title": {
				"en": "Totally valid FS22 Mod"
			},
			"description": {
				"en": "Demonstrates how FSModAssist handles a good mod file."
			}
		},
		"md5Sum": null,
		"modDesc": {
			"actions": {},
			"binds": {},
			"author": "FSModAssist Test",
			"scriptFiles": 0,
			"storeItems": 1,
			"cropInfo": null,
			"cropWeather": null,
			"depend": [],
			"descVersion": 69,
			"iconFileName": "modIcon.dds",
			"mapConfigFile": null,
			"mapCustomEnv": false,
			"mapCustomCrop": false,
			"mapCustomGrow": false,
			"mapIsSouth": false,
			"mapImage": null,
			"multiPlayer": true,
			"version": "1.0.0.0"
		},
		"uuid": "89b7c5117437014a47f7805bf24a0d43"
	});

	assert_json_include!(actual : json!(mod_record), expected : expected);
}