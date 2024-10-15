use std::path::{self, Path};
use data::{flags::ModError, functions::*, structs::ModRecord};
// use regex::{Regex, RegexBuilder};
use glob::glob;
mod data;
mod files;

use std::time::Instant;

// mod structs;
// pub use crate::structs::*;

fn main() {
    let start_time = Instant::now();
    // let pattern = "./test_mods/*";
    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\**\\*";
    for entry in glob(pattern).unwrap().filter_map(Result::ok) {
        let file_metadata = std::fs::metadata(&entry).unwrap();
        // if ! entry.ends_with("FS22_Dunalka.zip") { continue; }
        // if ! entry.ends_with("FS22_SwathingAddon.zip") { continue; }
        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let output = open_file_or_folder(abs_path.as_path(), file_metadata.is_dir());
                print!("{}\n", entry.clone().to_str().unwrap());
            },
            Err(e) => panic!("{}", e),
        };
    }
    let elapsed = start_time.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn badge_and_output(mod_record: &mut ModRecord) -> String {
    serde_json::to_string_pretty(&mod_record).unwrap()
}


fn open_file_or_folder(full_path :&Path, is_folder: bool) -> String {
    let mut mod_record = data::structs::new_record(&full_path, is_folder);

    mod_record.can_not_use = !data::functions::test_file_name(&mut mod_record);

    if mod_record.can_not_use {
        mod_record.issues.insert(ModError::FileErrorNameInvalid);
        return badge_and_output(&mut mod_record)
    }

    let mut abstract_file: Box<dyn files::AbstractFileHandle> = if is_folder 
        {
            match files::new_abstract_folder(&full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.issues.insert(e);
                    mod_record.can_not_use = true;
                    return badge_and_output(&mut mod_record);
                }
            }
        } else {
            match files::new_abstract_zip_file(&full_path) {
                Ok(archive) => Box::new(archive),
                Err(e) => {
                    mod_record.issues.insert(e);
                    mod_record.can_not_use = true;
                    return badge_and_output(&mut mod_record);
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
        mod_record.issues.insert(ModError::FileErrorLikelySaveGame);
        return badge_and_output(&mut mod_record)
    }

    if ! abstract_file.is_folder() {
        if abstract_file_list.iter().all(|x| x.name.ends_with(".zip")) {
            mod_record.file_detail.is_mod_pack = true;
            mod_record.issues.insert(ModError::FileErrorLikelyZipPack);
            return badge_and_output(&mut mod_record);
        }
    }

    let mod_desc_content = match abstract_file.as_text("modDesc.xml") {
        Ok(content) => content,
        Err(..) => {
            mod_record.issues.insert(ModError::ModDescMissing);
            return badge_and_output(&mut mod_record);
        },
    };

    let mod_desc_doc = match parse_xml(&mod_desc_content) {
        Ok(tree) => tree,
        Err(..) => {
            mod_record.issues.insert(ModError::ModDescParseError);
            return badge_and_output(&mut mod_record);
        }
    };

    // print!("{:#?}", mod_desc_doc);
    do_file_counts(&mut mod_record, abstract_file_list);
    mod_desc_basics(&mut mod_record, &mod_desc_doc);

    mod_record.mod_desc.icon_image = match &mod_record.mod_desc.icon_file_name {
        Some(filename) => load_mod_icon(abstract_file.as_bin(&filename).unwrap()),
        None => None,
    };

    badge_and_output(&mut mod_record)
}

fn parse_xml(content: &str) -> Result<roxmltree::Document<'_>, roxmltree::Error> {
    roxmltree::Document::parse(content)
}
