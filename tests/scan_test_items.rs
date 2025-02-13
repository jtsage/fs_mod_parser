use glob::glob;
use rayon::prelude::*;
use std::path::{self, PathBuf};
use std::time::Instant;
use fs_mod_parser::parser::parse;

#[test]
#[ignore]
fn scan_test_items() {
    let start_time = Instant::now();

    let pattern = "./tests/test_mods/**/*";

    let file_list: Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    let counter = file_list.len();

    file_list.par_iter().for_each(|entry| {
        let this_file_start = Instant::now();

        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parse(abs_path.as_path());

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
