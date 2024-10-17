use std::collections::HashSet;
use std::path::Path;
use fs_mod_parser::mod_basic::parser;
use fs_mod_parser::shared::errors::ModError;
use fs_mod_parser::shared::structs::ModBadges;

#[test]
fn broken_zip_file() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Broken_Zip_File.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorUnreadableZip]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : true,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}

#[test]
fn bad_crc_moddesc() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Bad_ModDesc_CRC.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::ModDescMissing]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : true,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}


#[test]
fn garbage_file() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Garbage_File.txt");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorGarbageFile, ModError::FileErrorNameInvalid]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : true,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}

#[test]
fn missing_desc_version() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_No_DescVersion.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, false);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::ModDescVersionOldOrMissing, ModError::ModDescNoModVersion]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
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
fn missing_moddesc() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Missing_ModDesc.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::ModDescMissing]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : true,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}


#[test]
fn invalid_file_copy() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Copied_Mod (2).zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorLikelyCopy, ModError::FileErrorNameInvalid]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.file_detail.copy_name, Some("FAILURE_Copied_Mod".to_owned()));

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}


#[test]
fn malformed_moddesc() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Really_Malformed_ModDesc.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::ModDescParseError]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}



#[test]
fn starts_with_digit() {
	let test_file_path = Path::new("./tests/test_mods/0FAILURE_Starts_With_Digit.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorNameStartsDigit, ModError::FileErrorNameInvalid]);
	assert_eq!(mod_record.issues, expected_errors);

	assert_eq!(mod_record.file_detail.copy_name, None);

	assert_eq!(mod_record.badge_array, ModBadges {
		broken   : true,
		folder   : false,
		malware  : false,
		no_mp    : false,
		notmod   : false,
		pconly   : false,
		problem  : false,
		savegame : false,
	});
}
