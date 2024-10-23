//! A rust crate for parsing Farming Simulator mod files
//! 
//! This is a simple test program that will fail anywhere but the developers machine.
use std::path::{self, PathBuf};
use std::time::Instant;
use glob::glob;
use fs_mod_parser::{parse_detail_json_pretty, parse_mod, parse_savegame};
use rayon::prelude::*;

fn main() {
    // println!("{}", parse_detail_json_pretty("./tests/test_mods/PASS_Good_Simple_Mod.zip"));

    // let _out = parse_detail_json_pretty("./tests/test_mods/UNUSED_RealModForStoreItems.zip");

    // println!("{_out}");

    // println!("{}", parse_detail_json_pretty("./tests/test_mods/FSG_Color_Pack.zip"));
    // println!("{}", parse_detail_json_pretty("./tests/test_mods/FS22_CaseTitanPack.zip"));
    // let _out = parse_detail_json_pretty("./tests/test_mods/FS22_CaseTitanPack.zip");

    // println!("{_out}");
    detail_full_collection();
    // scan_full_collection();
    // scan_all_save_games();
}

/// Scan and optionally show output from a test file
#[allow(dead_code)]
fn scan_test_items(pattern_part: &str, show_output : bool) {
    let start_time = Instant::now();

    let pattern = format!("./tests/test_mods/{}", pattern_part);

    let file_list:Vec<PathBuf> = glob(pattern.as_str()).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_mod(abs_path.as_path()).to_json_pretty();

                println!("{} in {:.2?}", entry.clone().to_str().unwrap(), this_file_start.elapsed());
                if show_output {
                    println!("{}", _output);
                }
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}


/// Scan full set of mods
#[allow(dead_code)]
fn detail_full_collection() {
    let start_time = Instant::now();

    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";

    let file_list:Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_detail_json_pretty(abs_path.as_path());
                println!("{_output}");
                println!("{} in {:.2?}", entry.clone().to_str().unwrap(), this_file_start.elapsed());
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}

/// Scan full set of mods
#[allow(dead_code)]
fn scan_full_collection() {
    let start_time = Instant::now();

    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";

    let file_list:Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_mod(abs_path.as_path()).to_json_pretty();

                println!("{} in {:.2?}", entry.clone().to_str().unwrap(), this_file_start.elapsed());
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}


/// Check all local save games
#[allow(dead_code)]
fn scan_all_save_games() {
    let start_time = Instant::now();

    let pattern_1 = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegame[0-9]";
    let pattern_2 = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegame[0-9][0-9]";
    let pattern_3 = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegameBackup\\*";

    let mut file_list:Vec<PathBuf> = glob(pattern_1).unwrap().filter_map(Result::ok).collect();
    file_list.extend(glob(pattern_2).unwrap().filter_map(Result::ok).collect::<Vec<PathBuf>>());
    file_list.extend(glob(pattern_3).unwrap().filter_map(Result::ok).collect::<Vec<PathBuf>>());

    let counter = file_list.len();

    file_list.par_iter().for_each(|entry|{
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_savegame(abs_path.as_path()).to_json_pretty();

                println!("{} in {:.2?}", entry.clone().to_str().unwrap(), this_file_start.elapsed());
            },
            Err(e) => panic!("{}", e),
        };
    });


    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}
