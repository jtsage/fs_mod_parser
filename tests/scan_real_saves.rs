use fs_mod_parser::parse_savegame;
use glob::glob;
use rayon::prelude::*;
use std::path::{self, PathBuf};
use std::time::Instant;

#[test]
#[ignore]
fn scan_real_saves() {
    let start_time = Instant::now();

    let pattern_1 = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegame[0-9]";
    let pattern_2 =
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegame[0-9][0-9]";
    let pattern_3 =
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegameBackup\\*";

    let mut file_list: Vec<PathBuf> = glob(pattern_1).unwrap().filter_map(Result::ok).collect();
    file_list.extend(
        glob(pattern_2)
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<PathBuf>>(),
    );
    file_list.extend(
        glob(pattern_3)
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<PathBuf>>(),
    );

    let counter = file_list.len();

    file_list.par_iter().for_each(|entry| {
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_savegame(abs_path.as_path()).to_json_pretty();

                println!(
                    "{} in {:.2?}",
                    entry.clone().to_str().unwrap(),
                    this_file_start.elapsed()
                );
                // println!("{}", _output);
            }
            Err(e) => panic!("{}", e),
        };
    });

    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {:.2?} for {} files", elapsed, counter);
}
