use std::collections::HashSet;
use std::path::Path;
use fs_mod_parser::mod_basic::parser;
use fs_mod_parser::shared::errors::ModError;
use fs_mod_parser::shared::structs::{ModBadges, ZipPackFile};

#[test]
fn is_seven_zip() {
	let test_file_path = Path::new("./tests/test_mods/UNSUPPORTED.7z");
	assert!(!test_file_path.exists());

	let mod_record = parser(test_file_path);
	let _ = mod_record.to_json();

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([
		ModError::FileErrorUnsupportedArchive,
		ModError::FileErrorUnreadableZip,
		ModError::FileErrorNameInvalid
	]);
	assert_eq!(mod_record.issues, expected_errors);
}

#[test]
fn is_rar() {
	let test_file_path = Path::new("./tests/test_mods/UNSUPPORTED.rar");
	assert!(!test_file_path.exists());

	let mod_record = parser(test_file_path);
	let _ = mod_record.to_json();

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([
		ModError::FileErrorUnsupportedArchive,
		ModError::FileErrorUnreadableZip,
		ModError::FileErrorNameInvalid
	]);
	assert_eq!(mod_record.issues, expected_errors);
}

#[test]
fn is_zip_pack() {
	let test_file_path = Path::new("./tests/test_mods/VARIANT_Mod_Pack.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path);
	let _ = mod_record.to_json();

	assert_eq!(mod_record.can_not_use, true);

	let expected_errors:HashSet<ModError> = HashSet::from([ModError::FileErrorLikelyZipPack]);
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

	let contained_files:Vec<ZipPackFile> = vec![
		ZipPackFile { name : String::from("EXAMPLE_No_DescVersion.zip"), size : 12025 },
		ZipPackFile { name : String::from("EXAMPLE_No_Version.zip"), size : 12033 },
		ZipPackFile { name : String::from("EXAMPLE_Missing_ModDesc.zip"), size : 152 },
	];

	assert_eq!(mod_record.file_detail.zip_files, contained_files)
}
