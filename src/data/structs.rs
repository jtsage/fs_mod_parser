use std::{collections::HashSet, path::Path};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModRecord {
	pub badge_array        : Vec<String>,
	pub can_not_use        : bool,
	pub current_collection : String,
	pub file_detail        : ModFile,
	pub issues             : HashSet<super::flags::ModError>,
	// pub l10n               : this.#l10n,
	pub md5_sum            : Option<String>,
	pub mod_desc           : ModDesc,
	pub uuid               : String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDesc {
	// actions : Vec<Actions>,
	//binds          : Vec<Binds>,
	pub author          : String,
	pub script_files    : u32,
	pub store_items     : u32,
	pub crop_info       : bool,
	pub crop_weather    : Option<String>,
	pub depend          : Vec<String>,
	pub desc_version    : u32,
	pub icon_file_name  : Option<String>,
	pub icon_image      : Option<String>,
	pub map_config_file : Option<String>,
	pub map_is_south    : bool,
	pub multi_player    : bool,
	pub version         : String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
	pub copy_name     : Option<String>,
	pub extra_files   : Vec<String>,
	pub file_date     : String,
	pub file_size     : u32,
	pub full_path     : String,
	pub i3d_files     : Vec<String>,
	#[serde(rename = "imageDDS")]
	pub image_dds     : Vec<String>,
	#[serde(rename = "imageNonDDS")]
	pub image_non_dds : Vec<String>,
	pub is_folder     : bool,
	pub is_save_game  : bool,
	pub is_mod_pack   : bool,
	pub png_texture   : Vec<String>,
	pub short_name    : String,
	pub space_files   : Vec<String>,
	pub too_big_files : Vec<String>,
}



pub fn new_record(full_path: &Path, is_folder : bool) -> ModRecord {
	ModRecord {
		badge_array        : vec![],
		can_not_use        : true,
		current_collection : "".to_owned(),
		file_detail        : file_detail_new_record(full_path, is_folder),
		issues             : HashSet::new(),
		// l10n               : this.#l10n,
		md5_sum            : None,
		mod_desc           : mod_desc_new_record(),
		uuid               : format!("{:?}", md5::compute(full_path.to_str().unwrap()))
	}
}

fn file_detail_new_record(file : &Path, is_folder : bool) -> ModFile {
	ModFile {
		copy_name     : None,
		extra_files   : vec![],
		file_date     : "".to_owned(),
		file_size     : 0,
		full_path     : format!("{}", file.to_str().unwrap()),
		i3d_files     : vec![],
		image_dds     : vec![],
		image_non_dds : vec![],
		is_folder     : is_folder.to_owned(),
		is_save_game  : false,
		is_mod_pack   : false,
		png_texture   : vec![],
		short_name    : file.file_stem().unwrap().to_str().unwrap().to_owned(),
		space_files   : vec![],
		too_big_files : vec![],
	}
}
fn mod_desc_new_record() -> ModDesc {
	ModDesc {
		author          : "--".to_owned(),
		script_files    : 0,
		store_items     : 0,
		crop_info       : false,
		crop_weather    : None,
		depend          : vec![],
		desc_version    : 0,
		icon_file_name  : None,
		icon_image      : None,
		map_config_file : None,
		map_is_south    : false,
		multi_player    : false,
		version         : "--".to_owned(),
	}
}
