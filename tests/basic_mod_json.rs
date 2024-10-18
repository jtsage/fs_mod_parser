use std::path::Path;
use fs_mod_parser::shared::structs::{ModBadges, ModRecord};

#[test]
fn check_json_mod_record() {
	let mut mod_record = ModRecord::new(Path::new("foo.txt"), false);

	mod_record.update_badges();

	let expected_json = "{\"badgeArray\":[\"noMP\"],\"canNotUse\":true,\"currentCollection\":\"\",\"fileDetail\":{\"copyName\":null,\"extraFiles\":[],\"fileDate\":\"\",\"fileSize\":0,\"fullPath\":\"foo.txt\",\"i3dFiles\":[],\"imageDDS\":[],\"imageNonDDS\":[],\"isFolder\":false,\"isSaveGame\":false,\"isModPack\":false,\"pngTexture\":[],\"shortName\":\"foo\",\"spaceFiles\":[],\"tooBigFiles\":[]},\"issues\":[],\"l10n\":{\"title\":{\"en\":\"--\"},\"description\":{\"en\":\"--\"}},\"md5Sum\":null,\"modDesc\":{\"actions\":{},\"binds\":{},\"author\":\"--\",\"scriptFiles\":0,\"storeItems\":0,\"cropInfo\":null,\"cropWeather\":null,\"depend\":[],\"descVersion\":0,\"iconFileName\":null,\"iconImage\":null,\"mapConfigFile\":null,\"mapIsSouth\":false,\"mapImage\":null,\"multiPlayer\":false,\"version\":\"--\"},\"uuid\":\"4fd8cc85ca9eebd2fa3c550069ce2846\"}";

	assert_eq!(mod_record.to_string(), expected_json)
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

	let expected_json = "[\"broken\",\"folder\",\"malware\",\"noMP\",\"notmod\",\"pconly\",\"problem\",\"savegame\"]";
	assert_eq!(serde_json::to_string(&mod_badges).unwrap(), expected_json)
}