use crate::ModParserOptions;
use crate::mod_detail::structs::{ModDetail, ModDetailError};
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile, FileDefinition};
use crate::shared::convert_mod_icon;
use std::path::Path;

pub mod structs;
mod vehicles;
mod places;

pub fn parser<P: AsRef<Path>>(full_path :P) -> ModDetail {
    parser_with_options(full_path, &ModParserOptions::default())
}

pub fn parser_with_options<P: AsRef<Path>>(full_path :P, options : &ModParserOptions) -> ModDetail {
    let is_folder = full_path.as_ref().is_dir();

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder 
        {
            if let Ok(archive) = AbstractFolder::new(full_path) {
                Box::new(archive)
            } else {
                return ModDetail::fast_fail(ModDetailError::FileReadFail);
            }
        } else if let Ok(archive) = AbstractZipFile::new(full_path) {
            Box::new(archive)
        } else {
            return ModDetail::fast_fail(ModDetailError::FileReadFail);
        };

    let abstract_file_list = abstract_file.list();

    let Ok(mod_desc_content) = abstract_file.as_text("modDesc.xml") else {
        return ModDetail::fast_fail(ModDetailError::NotModModDesc);
    };

    let Ok(mod_desc_doc) = roxmltree::Document::parse(&mod_desc_content) else {
        return ModDetail::fast_fail(ModDetailError::NotModModDesc);
    };

    parse_open_file(abstract_file, &mod_desc_doc, &abstract_file_list, options)
}

#[must_use]
pub fn parse_open_file( mut abstract_file: Box<dyn AbstractFileHandle>, mod_desc_doc : &roxmltree::Document, abstract_file_list: &[FileDefinition], options : &ModParserOptions ) -> ModDetail {
    let mut mod_detail = ModDetail::new();

    do_languages(&mut mod_detail, &mut abstract_file, mod_desc_doc, abstract_file_list);
    do_brands(&mut mod_detail, &mut abstract_file, mod_desc_doc);

    for store_item in mod_desc_doc.descendants().filter(|n| n.has_tag_name("storeItem")) {
        if let Some(file_name) = store_item.attribute("xmlFilename") {
            let Ok(file_content) = abstract_file.as_text(&file_name.to_string().replace('\\', "/")) else { continue; };
            let Ok(file_tree) = roxmltree::Document::parse(&file_content) else { continue; };

            if file_tree.root_element().has_tag_name("vehicle") {
                mod_detail.vehicles.push(vehicles::vehicle_parse(&file_tree, &mut abstract_file, options));
            } else if file_tree.root_element().has_tag_name("placeable") {
                mod_detail.placeables.push(places::place_parse(&file_tree, &mut abstract_file, options));
            }
        }
    }

    mod_detail
}

fn do_brands(
    mod_detail: &mut ModDetail,
    file_handle: &mut Box<dyn AbstractFileHandle>,
    mod_desc_doc : &roxmltree::Document
) {
    let Some(brand_key) = mod_desc_doc.descendants().find(|n| n.has_tag_name("brands") ) else { return; };

    for brand in brand_key.children().filter(|n| n.has_tag_name("brand")) {
        let Some(brand_name) = brand.attribute("name") else { continue; };
        let this_brand = mod_detail.add_brand(brand_name.to_uppercase().as_str(), brand.attribute("title"));

        let Some(brand_icon_file) = brand.attribute("image") else { continue; };

        if let Some(base_path) = brand_icon_file.strip_prefix("$data/") {
            // Is base path
            if let Some(base_filename) = Path::new(base_path).file_stem() {
                this_brand.icon_base = Some(base_filename.to_string_lossy().to_string().to_lowercase());
            }
        } else {
            // Load from disk, if it exists.
            let Ok(bin_file) = file_handle.as_bin(&normalize_icon_name(brand_icon_file)) else {
                mod_detail.add_issue(ModDetailError::BrandMissingIcon);
                continue;
            };
            this_brand.icon_file = convert_mod_icon(bin_file);
        }
    }
}

fn do_languages(
    mod_detail: &mut ModDetail,
    file_handle: &mut Box<dyn AbstractFileHandle>,
    mod_desc_doc : &roxmltree::Document,
    file_list: &[FileDefinition]
) {
    // <l10n filenamePrefix="languages/l10n" />
    //   *OR*
    // <l10n><text name="key"><en>value</en></text></l10n>

    let Some(lang_key) = mod_desc_doc.descendants().find(|n| n.has_tag_name("l10n") ) else { return; };

    if lang_key.has_children() {
        for lang_entry in lang_key.children() {
            let Some(l10n_key) = lang_entry.attribute("name") else { continue; };
            lang_entry.children().for_each(|n| {
                if let Some(l10n_value) = n.text() {
                    mod_detail.add_lang(
                        n.tag_name().name(),
                        l10n_key,
                        l10n_value
                    );
                }
            });
        }
    }

    if let Some(prefix) = lang_key.attribute("filenamePrefix") {
        for file_to_scan in file_list.iter().filter(|n| n.name.starts_with(prefix)) {
            let Ok(l10n_contents) = file_handle.as_text(&file_to_scan.name) else { continue; };
            let Ok(l10n_tree) = roxmltree::Document::parse(&l10n_contents) else { continue; };
            let lang_code = &file_to_scan.name[file_to_scan.name.len()-6..file_to_scan.name.len()-4];

            //<text name="key" text="value" /> style
            for entry in l10n_tree.descendants().filter(|n|n.has_tag_name("text")) {
                let Some(l10n_key) = entry.attribute("name") else { continue; };
                let Some(l10n_value) = entry.attribute("text") else { continue; };
                mod_detail.add_lang(lang_code,l10n_key, l10n_value);
            }

            // <e k="key" v="value"/> style
            for entry in l10n_tree.descendants().filter(|n|n.has_tag_name("e")) {
                let Some(l10n_key) = entry.attribute("k") else { continue; };
                let Some(l10n_value) = entry.attribute("v") else { continue; };
                mod_detail.add_lang(lang_code, l10n_key, l10n_value);
            }
        }
    }
}


fn xml_extract_text_as_opt_u32(xml_tree : &roxmltree::Document, key : &str) -> Option<u32> {
    xml_tree
        .descendants()
        .find(|n|n.has_tag_name(key))
        .and_then(|n|n.text())
        .and_then(|n| n.parse::<u32>().ok())
}
fn xml_extract_text_as_opt_string(xml_tree : &roxmltree::Document, key : &str) -> Option<String> {
    xml_tree
        .descendants()
        .find(|n|n.has_tag_name(key))
        .and_then(|n|n.text().map(std::string::ToString::to_string))
}

fn normalize_icon_name(name : &str) -> String {
    let mut value_string = name.to_string();
    if let Some(index) = value_string.find(".png") {
        value_string.replace_range(index..value_string.len(), ".dds");
    }
    value_string.replace('\\', "/")
}