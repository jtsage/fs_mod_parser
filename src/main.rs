use std::path::{self};
use std::time::Instant;
use glob::glob;

mod data;
mod files;
mod base_parse;

fn main() {
    let start_time = Instant::now();
    let mut counter: u64 = 0;
    // let pattern = "./test_mods/*";
    let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\fsg_realism\\*";
    for entry in glob(pattern).unwrap().filter_map(Result::ok) {
        let file_metadata = std::fs::metadata(&entry).unwrap();
        // if ! entry.ends_with("EXAMPLE_Malicious_Code.zip") { continue; }
        // if ! entry.is_dir() { continue; }
        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = base_parse::open_file_or_folder(abs_path.as_path(), file_metadata.is_dir());
                counter += 1;
                print!("{}\n", entry.clone().to_str().unwrap());
                // print!("{}\n", _output);
            },
            Err(e) => panic!("{}", e),
        };
    }
    let elapsed = start_time.elapsed();
    println!("Elapsed: {:.2?} for {} files", elapsed, counter);
}


