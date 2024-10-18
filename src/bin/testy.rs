//! A rust crate for parsing Farming Simulator mod files
//! 
//! Aims to provide a JSON representation of FS mod files
//! optionally including information on contained store
//! items, along with savegame processing, and basic mod
//! pack processing.
use std::path::{self, PathBuf};
use std::time::Instant;
use glob::glob;
use fs_mod_parser::parse_basic_mod;
use rayon::prelude::*;

fn main() {
    let start_time = Instant::now();

    // let pattern = "./tests/test_mods/*";
    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\maps_test\\*";

    let file_list:Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();
        let file_metadata = std::fs::metadata(&entry).unwrap();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_basic_mod(abs_path.as_path(), file_metadata.is_dir());
                // counter += 1;
                print!("{} in {:.2?}\n", entry.clone().to_str().unwrap(), this_file_start.elapsed());
                // print!("{}\n", _output);
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}


