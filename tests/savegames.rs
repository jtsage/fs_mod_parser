use std::collections::HashSet;
use std::path::Path;
use fs_mod_parser::ModParserOptions;
use fs_mod_parser::{parse_mod, parse_mod_with_options};
use fs_mod_parser::shared::errors::ModError;
use fs_mod_parser::shared::structs::ModBadges;
use fs_mod_parser::parse_savegame;
use fs_mod_parser::savegame::SaveError;
use assert_json_diff::assert_json_include;
use serde_json::json;

#[test]
fn mod_parse_save_detection() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_Good.zip");
	assert!(test_file_path.exists());

	let mod_record = parse_mod(test_file_path);
	let _ = mod_record.to_json();

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorLikelySaveGame]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : true,
		pconly   : false,
		problem  : false,
		savegame : true,
	});
}


#[test]
fn missing_file() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_Fake_File.zip");
	assert!(!test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, false);

	let expected_errors:HashSet<SaveError> = HashSet::from([SaveError::FileUnreadable]);
	assert_eq!(save_record.error_list, expected_errors);

	assert!(save_record.to_json().len() > 10);
	assert!(save_record.to_json_pretty().len() > 10);
}

#[test]
fn missing_career() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_No_Career.zip");
	assert!(test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, false);

	let expected_errors:HashSet<SaveError> = HashSet::from([SaveError::CareerMissing]);
	assert_eq!(save_record.error_list, expected_errors);
}

#[test]
fn missing_farms() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_No_Farms.zip");
	assert!(test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, false);

	let expected_errors:HashSet<SaveError> = HashSet::from([SaveError::FarmsMissing]);
	assert_eq!(save_record.error_list, expected_errors);
}


#[test]
fn missing_vehicles() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_No_Vehicles.zip");
	assert!(test_file_path.exists());
	
	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, false);

	let expected_errors:HashSet<SaveError> = HashSet::from([SaveError::VehicleMissing]);
	assert_eq!(save_record.error_list, expected_errors);
}


#[test]
fn missing_placeable() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_No_Placeable.zip");
	assert!(test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, false);

	let expected_errors:HashSet<SaveError> = HashSet::from([SaveError::PlaceableMissing]);
	assert_eq!(save_record.error_list, expected_errors);
}

#[test]
fn all_malformed() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_Malformed.zip");
	assert!(test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, false);

	let expected_errors:HashSet<SaveError> = HashSet::from([
		SaveError::PlaceableParseError,
		SaveError::CareerParseError,
		SaveError::FarmsParseError,
		SaveError::VehicleParseError,
	]);
	assert_eq!(save_record.error_list, expected_errors);
}

#[test]
fn good_multiplayer() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_Good.zip");
	assert!(test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, true);
	assert_eq!(save_record.error_list.len(), 0);

	assert_eq!(save_record.single_farm, false);

	let actual = json!(save_record);

	let expected_record = json!({
		"errorList": [],
		"isValid": true,
		"mapMod": "FS22_BackRoadsCounty",
		"mapTitle": "Back Roads County",
		"modCount": 38,
		"name": "BRC",
		"playTime": "306:40",
		"saveDate": "2022-10-14",
		"singleFarm": false
	});

	assert_json_include!(actual : actual, expected : expected_record);

	/* cSpell: disable */
	let expected_farms = json!({
		"farms": {
			"2": { "name": "PUBLIC", "cash": 878837, "loan": 0, "color": 8 },
			"5": { "name": "THE CROFT", "cash": 42937, "loan": 0, "color": 6 },
			"4": { "name": "BELLWETHER RANCH", "cash": 110758, "loan": 0, "color": 2 },
			"0": { "name": "--unowned--", "cash": 0, "loan": 0, "color": 1 },
			"3": { "name": "joinFSG.gg", "cash": 100000, "loan": 0, "color": 1 },
			"1": { "name": "HENNESSEY ACRES", "cash": 46198, "loan": 230000, "color": 7 }
		},
	});
	/* cSpell: enable */

	assert_json_include!(actual : actual, expected : expected_farms);

	let expected_mod = json!({
		"mods" : {
			"FS22_BackRoadsCounty": {
				"version": "1.0.0.2",
				"title": "Back Roads County",
				"farms": [ 0, 1, 4, 5, 15 ]
			},
		}
	});

	assert_json_include!(actual : actual, expected : expected_mod);
}


#[test]
fn good_single_player() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_Single_Farm.zip");
	assert!(test_file_path.exists());

	let save_record = parse_savegame(test_file_path);
	let _ = save_record.to_json();

	assert_eq!(save_record.is_valid, true);
	assert_eq!(save_record.error_list.len(), 0);

	assert_eq!(save_record.single_farm, true);

	let actual = json!(save_record);

	/* cSpell: disable */
	let expected_record = json!({
		"errorList": [],
		"isValid": true,
		"mapMod": "MapFR",
		"mapTitle": "Haut-Beyleron",
		"modCount": 0,
		"name": "Mój zapis gry",
		"playTime": "13330:03",
		"saveDate": "2024-03-18",
		"singleFarm": true
	});
	/* cSpell: enable */

	assert_json_include!(actual : actual, expected : expected_record);

	/* cSpell: disable */
	let expected_farms = json!({
		"farms": {
			"0": { "name": "--unowned--", "cash": 0, "loan": 0, "color": 1 },
			"1": { "name": "Moje gospodarstwo", "cash": 13656933, "loan": 0, "color": 1 }
		},
	});
	/* cSpell: enable */

	assert_json_include!(actual : actual, expected : expected_farms);

}

#[test]
fn mod_parse_save_detection_with_scan() {
	let test_file_path = Path::new("./tests/test_mods/SAVEGAME_Good.zip");
	assert!(test_file_path.exists());

	let options = ModParserOptions{
		include_save_game : true,
		..Default::default()
	};

	let mod_record = parse_mod_with_options(test_file_path, &options);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorLikelySaveGame]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : true,
		pconly   : false,
		problem  : false,
		savegame : true,
	});

	assert!(mod_record.include_save_game.is_some());

	let byte_length = mod_record.to_json_pretty().len() as i32;
	let byte_expected:i32 = 7615;
	let byte_margin = 100;
	assert!(
		(byte_length - byte_expected).abs() < byte_margin,
		"assertion failed: `(left !== right)` \
		(left: `{:?}`, right: `{:?}`, expect diff: `{:?}`, real diff: `{:?}`)",
		byte_length,
		byte_expected,
		byte_margin,
		(byte_length - byte_expected).abs()
	);
}