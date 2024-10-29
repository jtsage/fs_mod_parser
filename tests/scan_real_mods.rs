use fs_mod_parser::{parse_mod_with_options, ModParserOptions};
use glob::glob;
use rayon::prelude::*;
use std::path::{self, PathBuf};
use std::time::Instant;

#[test]
#[ignore]
fn scan_test_items() {
    let options = ModParserOptions {
        skip_detail_icons: true,
        skip_mod_icons: false,
        include_mod_detail: true,
        include_save_game: true,
        ..Default::default()
    };

    let start_time = Instant::now();

    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";

    let file_list: Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry| {
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse_mod_with_options(abs_path.as_path(), &options).to_json_pretty();

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
