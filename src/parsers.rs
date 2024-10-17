use std::path::Path;
use super::data::{flags::ModError, functions::*, structs::{new_record, NOT_MALWARE}};
use regex::RegexBuilder;


pub fn parse_base_mod(full_path :&Path, is_folder: bool) -> String {
    let mut mod_record = new_record(&full_path, is_folder);

    mod_record.can_not_use = !test_file_name(&mut mod_record);

    if mod_record.can_not_use {
        mod_record.add_issue(ModError::FileErrorNameInvalid);
        return mod_record.update_badges().to_string();
    }

    let mut abstract_file: Box<dyn super::files::AbstractFileHandle> = if is_folder 
        {
            mod_record.add_issue(ModError::InfoNoMultiplayerUnzipped);
            match super::files::new_abstract_folder(&full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.add_issue(e);
                    mod_record.can_not_use = true;
                    return mod_record.update_badges().to_string();
                }
            }
        } else {
            match super::files::new_abstract_zip_file(&full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.add_issue(e);
                    mod_record.can_not_use = true;
                    return mod_record.update_badges().to_string();
                } 
            }
        };

    let abstract_file_list = abstract_file.list();

    match std::fs::metadata(&full_path) {
        Ok(meta) => {
            match meta.created() {
                Ok(time) => mod_record.file_detail.file_date = sys_time_to_string(time),
                Err(..) => {},
            }
            if ! abstract_file.is_folder() {
                mod_record.file_detail.file_size = meta.len();
            } else {
                let mut full_size:u64 = 0;
                for entry in &abstract_file_list {
                    full_size += entry.size;
                }
                mod_record.file_detail.file_size = full_size;
            }
        },
        Err(..) => {},
    }

    if abstract_file.exists("careerSavegame.xml") {
        mod_record.file_detail.is_save_game = true;
        mod_record.add_issue(ModError::FileErrorLikelySaveGame);
        return mod_record.update_badges().to_string();
    }

    if ! abstract_file.is_folder() {
        if abstract_file_list.iter().all(|x| x.name.ends_with(".zip")) {
            mod_record.file_detail.is_mod_pack = true;
            mod_record.add_issue(ModError::FileErrorLikelyZipPack);
            return mod_record.update_badges().to_string();
        }
    }

    let mod_desc_content = match abstract_file.as_text("modDesc.xml") {
        Ok(content) => content,
        Err(..) => {
            mod_record.add_issue(ModError::ModDescMissing);
            return mod_record.update_badges().to_string();
        },
    };

    let mod_desc_doc = match parse_xml(&mod_desc_content) {
        Ok(tree) => tree,
        Err(..) => {
            mod_record.add_issue(ModError::ModDescParseError);
            return mod_record.update_badges().to_string();
        }
    };

    do_file_counts(&mut mod_record, &abstract_file_list);
    mod_desc_basics(&mut mod_record, &mod_desc_doc);

    mod_record.mod_desc.icon_image = match &mod_record.mod_desc.icon_file_name {
        Some(filename) => load_mod_icon(abstract_file.as_bin(&filename).unwrap()),
        None => None,
    };

    if ! NOT_MALWARE.contains(&mod_record.file_detail.short_name.clone().as_str()) {
        let re_1 = RegexBuilder::new(r"\.deleteFolder")
            .multi_line(true)
            .build()
            .unwrap();
        let re_2 = RegexBuilder::new(r"\.deleteFile")
            .multi_line(true)
            .build()
            .unwrap();

        for lua_file in abstract_file_list.iter().filter(|n|n.name.ends_with(".lua")) {
            match abstract_file.as_text(&lua_file.name) {
                Ok(content) => {
                    if re_1.is_match(&content.as_str()) { mod_record.add_issue(ModError::InfoMaliciousCode); }
                    if re_2.is_match(&content.as_str()) { mod_record.add_issue(ModError::InfoMaliciousCode); }
                },
                Err(..) => {},
            }
        }
    }

    super::data::maps::read_map_basics(&mut mod_record, &mut abstract_file);

    // match &mod_record.mod_desc.map_config_file {
    //     Some(filename) => {
    //         match abstract_file.as_text(filename) {
    //             Ok(content) => {
    //                 match parse_xml(&content) {
    //                     Ok(map_config_tree) => {
    //                         let fruit_types = nullify_base_game_entry(&map_config_tree, "fruitTypes");
    //                         let growth = nullify_base_game_entry(&map_config_tree, "growth");
    //                         let environment_included = nullify_base_game_entry(&map_config_tree, "environment");
    //                         let environment_base = get_base_game_entry_key(&map_config_tree);
                            

    //                     },
    //                     Err(..) => {}
    //                 };
    //             },
    //             Err(..) => {},
    //         }
    //     },
    //     None => {},
    // }

    mod_record.update_badges().to_string()
}


