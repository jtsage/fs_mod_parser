use fs_mod_parser::parse_savegame;
use glob::glob;
use rayon::prelude::*;
use std::path::{self, PathBuf};
use std::time::Instant;

#[test]
#[ignore]
fn scan_real_saves() {
    let start_time = Instant::now();

    let patterns:[&str; 6] = [
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegame[0-9]",
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegame[0-9][0-9]",
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\savegameBackup\\*",
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2025\\savegame[0-9]",
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2025\\savegame[0-9][0-9]",
        "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2025\\savegameBackup\\*",
    ];

    let mut file_list:Vec<PathBuf> = vec![];
    for pattern in patterns {
        file_list.extend(glob(pattern).unwrap().filter_map(Result::ok).collect::<Vec<PathBuf>>());
    }

    let counter = file_list.len();

    file_list.par_iter().for_each(|entry| {
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_savegame(abs_path.as_path());

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
