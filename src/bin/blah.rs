// use fs_mod_parser::{parse_mod_with_options, ModParserOptions};
//! fuck off
use glob::glob;
// use std::collections::HashMap;
use std::path::{self, PathBuf};
use std::time::Instant;
// use std::fs;
use rayon::prelude::*;

use fs_mod_parser::{ParseOption, ParseOptions};
use fs_mod_parser::parser::parse_with_options;

fn main() {
    // rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();

    let options:ParseOptions = vec![
        ParseOption::IncludeDetail,
        ParseOption::IncludeMap,
        ParseOption::IncludeSaveGame,
        ParseOption::ImageMod,
        // ParseOption::ImageDetail,
    ].into();

    // let mut items:HashMap<String, fs_mod_parser::parser::Record> = HashMap::new();
    let start_time = Instant::now();

    // let pattern = "./tests/test_mods/DETAIL*";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\test-downloading\\*.zip";
    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*.zip";

    let file_list: Vec<PathBuf> = glob(pattern).expect("die").filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry| {
    // for entry in file_list {
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _result = parse_with_options(abs_path, &options);
                println!(
                    "{} in {:.2?}",
                    entry.clone().to_string_lossy(),
                    this_file_start.elapsed(),
                );
                // items.insert(entry.to_string_lossy().to_string(), result);
                // println!("{}", serde_json::to_string_pretty(&result.issues).expect(""));s
            }
            Err(e) => panic!("{}", e),
        };
    });

    // fs::write("./blah_result.json", serde_json::to_string_pretty(&items).expect("")).expect("Unable to write file");

    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {elapsed:.2?} for {counter} files");
}
