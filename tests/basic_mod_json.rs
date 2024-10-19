use std::path::Path;
use fs_mod_parser::shared::structs::{ModBadges, ModRecord};
use assert_json_diff::assert_json_eq;
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

	assert_eq!(json!(mod_badges), expected)
}