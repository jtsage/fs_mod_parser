//! Simple command line mod file parser
use fs_mod_parser::{parse_with_options, ParseOption, ParseOptions};
use std::env;
use std::path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:\n  parse_file [path_to_mod]\n");
        println!("No input file specified");
        std::process::exit(0);
    }

    let options:ParseOptions = vec![
        ParseOption::IncludeDetail,
        ParseOption::IncludeMap,
        ParseOption::IncludeSaveGame,
        ParseOption::ImageMod,
        ParseOption::ImageMap,
    ].into();

    if let Ok(file) = path::absolute(&args[1]) {
        let output = parse_with_options(file.as_path(), &options);

        println!("{}", serde_json::to_string_pretty(&output).expect("fail!"));
    }
}
