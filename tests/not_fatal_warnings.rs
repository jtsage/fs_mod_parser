use std::collections::HashSet;
use std::path::Path;
use fs_mod_parser::mod_basic::parser;
use fs_mod_parser::shared::errors::ModError;
use fs_mod_parser::shared::structs::ModBadges;

#[test]
fn piracy_warning() {
	let test_file_path = Path::new("./tests/test_mods/WARNING_Fake_Cracked_DLC.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);

	assert_eq!(mod_record.can_not_use, false);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::InfoLikelyPiracy, ModError::PerformanceQuantityExtra]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : true,
		savegame : false,
	});
}


#[test]
fn icon_not_found() {
	let test_file_path = Path::new("./tests/test_mods/WARNING_Icon_Not_Found.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);

	assert_eq!(mod_record.can_not_use, false);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::ModDescNoModIcon]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : true,
		savegame : false,
	});
}


#[test]
fn malicious_code_check() {
	let test_file_path = Path::new("./tests/test_mods/WARNING_Malicious_Code.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);

	assert_eq!(mod_record.can_not_use, false);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::InfoMaliciousCode, ModError::PerformanceFileSpaces, ModError::ModDescNoModIcon]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : true,
		no_mp    : false,
		notmod   : false,
		pconly   : true,
		problem  : true,
		savegame : false,
	});
}


#[test]
fn no_version() {
	let test_file_path = Path::new("./tests/test_mods/WARNING_No_Version.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);

	assert_eq!(mod_record.can_not_use, false);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::ModDescNoModVersion]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : true,
		savegame : false,
	});
}



#[test]
fn server_warnings() {
	let test_file_path = Path::new("./tests/test_mods/WARNING_Size_Test_Mod.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);

	assert_eq!(mod_record.can_not_use, false);

	let expected_errors:HashSet<ModError> = HashSet::from([
		ModError::PerformanceOversizeDDS,
		ModError::PerformanceOversizeGDM,
		ModError::PerformanceOversizeI3D,
		ModError::PerformanceOversizeSHAPES,
		ModError::PerformanceOversizeXML,
		ModError::PerformanceQuantityGRLE,
		ModError::PerformanceQuantityPDF,
		ModError::PerformanceQuantityTXT
	]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : false,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : true,
		savegame : false,
	});
}

