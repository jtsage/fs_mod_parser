// use fs_mod_parser::{parse_mod_with_options, ModParserOptions};
//! fuck off
use glob::glob;
use std::path::{self, PathBuf};
use std::time::Instant;

use fs_mod_parser::parser::parse;
fn main() {
    // rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();

    // let options = ModParserOptions {
    //     skip_detail_icons: true,
    //     skip_mod_icons: true,
    //     include_mod_detail: true,
    //     include_save_game: true,
    //     ..Default::default()
    // };

    let start_time = Instant::now();

    let pattern = "./tests/test_mods/*";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\test-downloading\\*.zip";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*.zip";

    let file_list: Vec<PathBuf> = glob(pattern).expect("die").filter_map(Result::ok).collect();
    let counter = file_list.len();

    // file_list.par_iter().for_each(|entry| {
    for entry in file_list {
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let result = parse(abs_path);
                println!(
                    "{} in {:.2?}",
                    entry.clone().to_string_lossy(),
                    this_file_start.elapsed(),
                );
                println!("{}", serde_json::to_string_pretty(&result.issues).expect(""));
            }
            Err(e) => panic!("{}", e),
        };
    };

    let elapsed = start_time.elapsed();
    println!("Total Elapsed: {elapsed:.2?} for {counter} files");
}
