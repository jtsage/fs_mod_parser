#![allow(dead_code)]
use std::sync::LazyLock;
use std::fs;

use pretty_assertions::assert_eq;

use fs_mod_parser::{ParseOption, ParseOptions, parse, parse_with_options};
use fs_mod_parser::parser::Record;
use fs_mod_parser::files::AbstractFile;

static OPTIONS: LazyLock<ParseOptions> = LazyLock::new(|| {
    ParseOptions::from(vec![
        ParseOption::IncludeDetail,
        ParseOption::IncludeMap,
        ParseOption::IncludeSaveGame,
    ])
});

pub fn icon_record_from_file<S: AsRef<str>>(filename : S, dump : bool) {
    let filename = filename.as_ref();
    let file_zip = format!("./tests/test_mods/{filename}");
    let file_json = format!("{filename}.json");
    record_from_file_check(file_zip, file_json, dump, true);
}

pub fn record_from_file<S: AsRef<str>>(filename : S, dump : bool) {
    let filename = filename.as_ref();
    let file_zip = format!("./tests/test_mods/{filename}");
    let file_json = format!("{filename}.json");
    record_from_file_check(file_zip, file_json, dump, false);
}

fn record_from_file_check(file_zip : String, file_json : String, dump : bool, icons : bool) {
    let actual = if icons {
            parse(&file_zip)
        } else {
            parse_with_options(&file_zip, &OPTIONS)
        };

    if dump { println!("{file_zip} :::\n\n{}", serde_json::to_string_pretty(&actual).unwrap()) }

    let mut json_handle = AbstractFile::new("tests/test_mod_json");

    match json_handle {
        AbstractFile::Null(_) => panic!("JSON output folder not found"),
        _ => ()
    }

    let Ok(json_record) = json_handle.text(&file_json) else {
        panic!("{file_json} JSON output not found")
    };

    let expected:Record = serde_json::from_str(&json_record).expect(&format!("{file_json} serde parse failed"));
    assert_eq!(actual, expected, "{file_zip} comparison failed");
}

pub fn populate_result_file<S: AsRef<str>>(filename : S) {
    let filename = filename.as_ref();
    let file_zip = format!("./tests/test_mods/{filename}");
    let file_json = format!("./tests/test_mod_json/{filename}.json");
    let mut actual = parse_with_options(&file_zip, &OPTIONS);

    actual.ident = String::new();
    actual.file.age_hash = String::new();
    actual.file.full_path = file_zip;

    fs::write(file_json, serde_json::to_string_pretty(&actual).unwrap()).unwrap();
}

pub fn populate_icon_result_file<S: AsRef<str>>(filename : S) {
    let filename = filename.as_ref();
    let file_zip = format!("./tests/test_mods/{filename}");
    let file_json = format!("./tests/test_mod_json/{filename}.json");

    let mut actual = parse(&file_zip);

    actual.ident = String::new();
    actual.file.age_hash = String::new();
    actual.file.full_path = file_zip;

    fs::write(file_json, serde_json::to_string_pretty(&actual).unwrap()).unwrap();
}