//! A rust crate for parsing Farming Simulator mod files
//! 
//! Aims to provide a JSON representation of FS mod files
//! optionally including information on contained store
//! items, along with savegame processing, and basic mod
//! pack processing.
use std::path::{self, PathBuf};
use std::time::Instant;
use glob::glob;
use fs_mod_parser::{parse_basic_mod, savegame::parser};
use rayon::prelude::*;

fn main() {
    let output = parser("./tests/test_mods/VARIANT_SaveGame.zip", false);
    print!("{}", output.pretty_print());
    // scan_full_collection();
}

#[allow(dead_code)]
fn scan_test_items(pattern_part: &str) {
    let start_time = Instant::now();

    let pattern = format!("./tests/test_mods/{}", pattern_part);

    let file_list:Vec<PathBuf> = glob(pattern.as_str()).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();
        let file_metadata = std::fs::metadata(entry).unwrap();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_basic_mod(abs_path.as_path(), file_metadata.is_dir());

                println!("{} in {:.2?}", entry.clone().to_str().unwrap(), this_file_start.elapsed());
                // print!("{}\n", _output);
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}


#[allow(dead_code)]
fn scan_full_collection() {
    let start_time = Instant::now();

    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";

    let file_list:Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();
        let file_metadata = std::fs::metadata(entry).unwrap();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_basic_mod(abs_path.as_path(), file_metadata.is_dir());

                println!("{} in {:.2?}", entry.clone().to_str().unwrap(), this_file_start.elapsed());
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}

