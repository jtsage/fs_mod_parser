use std::path::Path;
use fs_mod_parser::mod_basic::parser;
use fs_mod_parser::shared::errors::ModError;

#[test]
fn main() {
	let test_file_path = Path::new("./tests/test_mods/FAILURE_Broken_Zip_File.zip");
	assert!(test_file_path.exists());

	let mod_record = parser(test_file_path, false);

	assert_eq!(mod_record.can_not_use, true);

	assert_eq!(mod_record.issues.len(), 1);
	assert!(mod_record.issues.contains(&ModError::FileErrorUnreadableZip));

	assert_eq!(mod_record.badge_array.broken, true);
	assert_eq!(mod_record.badge_array.folder, false);
	assert_eq!(mod_record.badge_array.malware, false);
	assert_eq!(mod_record.badge_array.no_mp, false);
	assert_eq!(mod_record.badge_array.notmod, false);
	assert_eq!(mod_record.badge_array.pconly, false);
	assert_eq!(mod_record.badge_array.problem, false);
	assert_eq!(mod_record.badge_array.savegame, false);
}
