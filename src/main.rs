use std::path::{self};
use std::time::Instant;
use glob::glob;

mod data;
mod files;
mod parsers;

fn main() {
    // print!("{}", serde_json::to_string_pretty(&data::maps::BG_CROP_WEATHER).unwrap());
    let start_time = Instant::now();
    let mut counter: u64 = 0;
    let mut last_duration;

    let pattern = "./test_mods/FS22_*";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\*\\*";
    // let pattern = "C:\\Users\\jtsag\\Documents\\My Games\\FarmingSimulator2022\\mods\\fsg_realism\\*";

    for entry in glob(pattern).unwrap().filter_map(Result::ok) {
        last_duration = start_time.elapsed();
        let file_metadata = std::fs::metadata(&entry).unwrap();

        // if ! entry.ends_with("EXAMPLE_Garbage_File.txt") { continue; }


        match path::absolute(entry.clone()) {
            Ok(abs_path) => {
                let _output = parsers::parse_base_mod(abs_path.as_path(), file_metadata.is_dir());
                counter += 1;
                print!("{} in {:.2?}\n\n", entry.clone().to_str().unwrap(), start_time.elapsed() - last_duration);
                print!("{}\n", _output);
            },
            Err(e) => panic!("{}", e),
        };
    }

    let elapsed = start_time.elapsed();
    println!("Elapsed: {:.2?} for {} files", elapsed, counter);
}


