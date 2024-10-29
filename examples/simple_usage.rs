//! Example of scanning a mod

fn main() {
    let options = fs_mod_parser::ModParserOptions {
        skip_detail_icons: true,
        ..Default::default()
    };
    let out =
        fs_mod_parser::parse_mod_with_options("./tests/test_mods/DETAIL_Samples.zip", &options)
            .to_json_pretty();
    println!("{out}");
}
