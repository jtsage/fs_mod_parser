use super::flags::ModError;
use std::{fs::File, io::Read};
use regex::{Regex, RegexBuilder};
use zip::{result::ZipError, ZipArchive};

pub struct FileDefinition {
    pub name : String,
    pub size : u64,
    pub is_folder : bool,
}

pub fn zip_file_text(archive : &mut ZipArchive<File>, needle : &str) -> Result<String, ZipError> {
    let mut file = archive.by_name(needle)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn zip_file_exists(archive : &mut ZipArchive<File>, needle : &str) -> bool {
	match archive.by_name(&needle) {
        Ok(..) => true,
        Err(..) => false,
	}
}

pub fn zip_list_files(archive: &mut ZipArchive<File>) -> Vec<FileDefinition> {
	let mut names: Vec<FileDefinition> = vec![];
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        names.push(FileDefinition{
            name      : file.enclosed_name().unwrap().to_string_lossy().into_owned(),
            size      : if file.is_dir() {0} else { file.size() },
            is_folder : file.is_dir()
        })
    }
	names
}

pub fn test_file_name(mod_record : &mut super::structs::ModRecord) -> bool {
    if !mod_record.file_detail.is_folder && ! mod_record.file_detail.full_path.ends_with(".zip") {
        if mod_record.file_detail.full_path.ends_with(".rar") {
            mod_record.issues.insert(ModError::FileErrorUnsupportedArchive);
        } else if mod_record.file_detail.full_path.ends_with(".7z") {
            mod_record.issues.insert(ModError::FileErrorUnsupportedArchive);
        } else {
            mod_record.issues.insert(ModError::FileErrorGarbageFile);
        }
        return false
    }

    let regex_zip_pack = RegexBuilder::new(r"unzip")
        .case_insensitive(true)
        .build()
        .unwrap();

    if regex_zip_pack.is_match(&mod_record.file_detail.short_name) {
        mod_record.issues.insert(ModError::FileErrorLikelyZipPack);
    }

    let regex_digit = Regex::new(r"^\d").unwrap();

    if regex_digit.is_match(&mod_record.file_detail.short_name) {
        mod_record.issues.insert(ModError::FileErrorNameStartsDigit);
    }

    let regex_good_file = Regex::new(r"^[A-Z_a-z]\w+$").unwrap();
    let regex_copy_capture = Regex::new(r"^(?<name>[A-Za-z]\w+)(?: - .+$| \(.+$)").unwrap();

    if ! regex_good_file.is_match(&mod_record.file_detail.short_name) {
        let Some(caps) = regex_copy_capture.captures(&mod_record.file_detail.short_name) else {
            return false
        };
        mod_record.issues.insert(ModError::FileErrorLikelyCopy);
        mod_record.file_detail.copy_name = Some(caps["name"].to_owned());
        return false
    }

    true
}

pub fn do_file_counts(mod_record : &mut super::structs::ModRecord, file_list : Vec<FileDefinition>) {
    let mut max_grle = 10;
    let mut max_pdf = 1;
    let mut max_png = 128;
    let mut max_txt = 2;

    let size_cache :u64 = 10485760;
    let size_dds :u64 = 12582912;
    let size_gdm :u64 = 18874368;
    let size_shapes :u64 = 268435456;
    let size_xml :u64 = 262144;

    let known_good = vec!["png", "dds", "i3d", "shapes", "lua", "gdm", "cache", "xml", "grle", "pdf", "txt", "gls", "anim", "ogg"];

    for file in file_list {
        if file.is_folder { continue }

        let this_path = std::path::Path::new(&file.name);
        let this_ext = this_path.extension().unwrap().to_str().unwrap();

        if ! known_good.contains(&this_ext) {
            if this_ext == "dat" || this_ext == "l64" {
                mod_record.issues.insert(ModError::InfoLikelyPiracy);
            }
            mod_record.issues.insert(ModError::PerformanceQuantityExtra);
            mod_record.file_detail.extra_files.push(file.name);
        } else {
            match this_ext {
                "png"  => max_png -= 1,
                "pdf"  => max_pdf -= 1,
                "grle" => max_grle -= 1,
                "txt"  => max_txt -= 1,
                "cache"  => if file.size > size_cache { mod_record.issues.insert(ModError::PerformanceOversizeI3D); },
                "dds"    => if file.size > size_dds { mod_record.issues.insert(ModError::PerformanceOversizeDDS); },
                "gdm"    => if file.size > size_gdm { mod_record.issues.insert(ModError::PerformanceOversizeGDM); },
                "shapes" => if file.size > size_shapes { mod_record.issues.insert(ModError::PerformanceOversizeSHAPES); },
                "xml"    => if file.size > size_xml { mod_record.issues.insert(ModError::PerformanceOversizeXML); },
                _ => {},
            }

            if max_grle < 0 { mod_record.issues.insert(ModError::PerformanceQuantityGRLE); }
            if max_pdf < 0 { mod_record.issues.insert(ModError::PerformanceQuantityPDF); }
            if max_png < 0 { mod_record.issues.insert(ModError::PerformanceQuantityPNG); }
            if max_txt < 0 { mod_record.issues.insert(ModError::PerformanceQuantityTXT); }
        }
    }
}