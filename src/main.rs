use data::{flags::ModError, functions::*, structs::ModRecord};
// use regex::{Regex, RegexBuilder};
mod data;

// mod structs;
// pub use crate::structs::*;

fn main() {
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Bad_ModDesc_CRC.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Broken_Zip_File.zip")));
    print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Fake_Cracked_DLC.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Garbage_File.txt")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Good_Mod (2).zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Good_Mod.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Icon_Not_Found.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Malicious_Code.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Missing_ModDesc.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_No_DescVersion.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_No_Version.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Really_Malformed_ModDesc.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/TestMod_TotallyValidZIP.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/savegame8.zip")));
    // print!("{}\n\n", open_zip_file(String::from("./test_mods/EXAMPLE_Mod_Pack.zip")));
}

fn badge_and_output(mod_record: &mut ModRecord) -> String {
    serde_json::to_string(&mod_record).unwrap()
}


fn open_zip_file(file_path_string : String) -> String {
    let full_path = std::path::Path::new(&file_path_string);
    let mut mod_record = data::structs::new_record(&full_path, false);

    mod_record.can_not_use = !data::functions::test_file_name(&mut mod_record);

    if mod_record.can_not_use {
        mod_record.issues.insert(ModError::FileErrorNameInvalid);
        return badge_and_output(&mut mod_record)
    }

    let zip_file_result = std::fs::File::open(full_path);

    let zip_file = match zip_file_result {
        Ok(file) => file,
        Err(..) => {
            mod_record.issues.insert(ModError::FileErrorUnreadableZip);
            return badge_and_output(&mut mod_record)
        },
    };

    let archive_result = zip::ZipArchive::new(zip_file);

    let mut archive = match archive_result {
        Ok(file) => file,
        Err(..) => {
            mod_record.issues.insert(ModError::FileErrorUnreadableZip);
            return badge_and_output(&mut mod_record)
        },
    };

    if zip_file_exists(&mut archive, "careerSavegame.xml") {
        mod_record.file_detail.is_save_game = true;
        mod_record.issues.insert(ModError::FileErrorLikelySaveGame);
        return badge_and_output(&mut mod_record)
    }

    let zip_file_list = zip_list_files(&mut archive);

    for file in zip_file_list.iter() {
        print!("{}\n", file.name)
    }

    if zip_file_list.iter().all(|x| x.name.ends_with(".zip")) {
        mod_record.file_detail.is_mod_pack = true;
        mod_record.issues.insert(ModError::FileErrorLikelyZipPack);
        return badge_and_output(&mut mod_record);
    }

    let mod_desc_content = match zip_file_text(&mut archive, "modDesc.xml") {
        Ok(content) => content,
        Err(..) => {
            mod_record.issues.insert(ModError::ModDescMissing);
            return badge_and_output(&mut mod_record);
        },
    };

    print!("{}", mod_desc_content);

    do_file_counts(&mut mod_record, zip_file_list);

    badge_and_output(&mut mod_record)
}
