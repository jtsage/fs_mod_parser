use fs_mod_parser::{parse_mod_with_options, ModParserOptions};
use std::env;
use std::path;

static QUICK_SCAN: ModParserOptions = ModParserOptions {
    include_mod_detail: false,
    include_save_game: false,
    skip_detail_icons: true,
    skip_mod_icons: false,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:\n  parse_file [path_to_mod]\n");
        println!("No input file specified");
        std::process::exit(0);
    }

    if let Ok(file) = path::absolute(&args[1]) {
        let output = parse_mod_with_options(file.as_path(), &QUICK_SCAN).to_json_pretty();

        println!("{output}")
    }
}
