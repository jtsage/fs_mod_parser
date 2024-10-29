//! Parse mod storeItems, l10n additions and brands
use crate::mod_detail::structs::{ModDetail, ModDetailError};
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile, FileDefinition};
use crate::shared::{convert_mod_icon, normalize_image_file};
use crate::ModParserOptions;
use std::path::Path;

pub mod places;
pub mod structs;
pub mod vehicles;

/// Parse the given mod for:
///
/// - store items
/// - l10n additions
/// - brand additions
///
/// This returns (optionally) a JSON object that looks like:
/// ```json
/// {
///     "brands" : [],
///     "l10n" : {
///         "langCode" : {
///             "key" : "Translated String"
///         }
///     },
///     "issues": [],
///     "placeables": [],
///     "vehicles": [],
/// }
/// ```
///
/// See also [`crate::mod_detail::places::place_parse`] and [`crate::mod_detail::vehicles::vehicle_parse`]
pub fn parser<P: AsRef<Path>>(full_path: P) -> ModDetail {
    parser_with_options(full_path, &ModParserOptions::default())
}

/// Parse mod detail with options
pub fn parser_with_options<P: AsRef<Path>>(full_path: P, options: &ModParserOptions) -> ModDetail {
    let is_folder = full_path.as_ref().is_dir();

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder {
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

/// Parse mod details with an open [`AbstractFileHandle`]
#[must_use]
pub fn parse_open_file(
    mut abstract_file: Box<dyn AbstractFileHandle>,
    mod_desc_doc: &roxmltree::Document,
    abstract_file_list: &[FileDefinition],
    options: &ModParserOptions,
) -> ModDetail {
    let mut mod_detail = ModDetail::default();

    do_languages(
        &mut mod_detail,
        &mut abstract_file,
        mod_desc_doc,
        abstract_file_list,
    );
    do_brands(&mut mod_detail, &mut abstract_file, mod_desc_doc, options);

    for store_item in mod_desc_doc
        .descendants()
        .filter(|n| n.has_tag_name("storeItem"))
    {
        if let Some(file_name) = store_item.attribute("xmlFilename") {
            let Ok(file_content) = abstract_file.as_text(&file_name.to_owned().replace('\\', "/"))
            else {
                mod_detail.add_issue(ModDetailError::StoreItemMissing);
                continue;
            };
            let Ok(file_tree) = roxmltree::Document::parse(&file_content) else {
                mod_detail.add_issue(ModDetailError::StoreItemBroken);
                continue;
            };

            if file_tree.root_element().has_tag_name("vehicle") {
                mod_detail.vehicles.insert(
                    file_name.to_owned(),
                    vehicles::vehicle_parse(&file_tree, &mut abstract_file, options),
                );
            } else if file_tree.root_element().has_tag_name("placeable") {
                mod_detail.placeables.insert(
                    file_name.to_owned(),
                    places::place_parse(&file_tree, &mut abstract_file, options),
                );
            }

            for found_item in &mod_detail.vehicles {
                if let Some(value) = found_item.1.sorting.brand.clone() {
                    mod_detail.item_brands.insert(value);
                }
                if let Some(value) = found_item.1.sorting.category.clone() {
                    mod_detail.item_categories.insert(value);
                }
            }

            for found_item in &mod_detail.placeables {
                if let Some(value) = found_item.1.sorting.category.clone() {
                    mod_detail.item_categories.insert(value);
                }
            }
        }
    }

    mod_detail
}

/// Parse added brands
fn do_brands(
    mod_detail: &mut ModDetail,
    file_handle: &mut Box<dyn AbstractFileHandle>,
    mod_desc_doc: &roxmltree::Document,
    options: &ModParserOptions,
) {
    let Some(brand_key) = mod_desc_doc
        .descendants()
        .find(|n| n.has_tag_name("brands"))
    else {
        return;
    };

    for brand in brand_key.children().filter(|n| n.has_tag_name("brand")) {
        let Some(brand_name) = brand.attribute("name") else {
            continue;
        };
        let this_brand =
            mod_detail.add_brand(brand_name.to_uppercase().as_str(), brand.attribute("title"));

        let brand_icon_record = normalize_image_file(brand.attribute("image"));

        if !options.skip_detail_icons {
            if let Some(filename) = brand_icon_record.base_game {
                this_brand.icon_base = Some(filename);
            } else if let Some(filename) = brand_icon_record.local_file {
                let Ok(bin_file) = file_handle.as_bin(&filename) else {
                    mod_detail.add_issue(ModDetailError::BrandMissingIcon);
                    continue;
                };
                this_brand.icon_file = convert_mod_icon(bin_file);
            }
        }
    }
}

/// Parse added L10N keys and strings
///
/// Covers :
///
/// `<l10n filenamePrefix="languages/l10n" />`
///   *OR*
/// `<l10n><text name="key"><en>value</en></text></l10n>`
///
/// Covers files in the format
///
/// `<text name="key" text="value" />`
///  *OR*
/// `<e k="key" v="value"/>`
///
fn do_languages(
    mod_detail: &mut ModDetail,
    file_handle: &mut Box<dyn AbstractFileHandle>,
    mod_desc_doc: &roxmltree::Document,
    file_list: &[FileDefinition],
) {
    // <l10n filenamePrefix="languages/l10n" />
    //   *OR*
    // <l10n><text name="key"><en>value</en></text></l10n>

    let Some(lang_key) = mod_desc_doc.descendants().find(|n| n.has_tag_name("l10n")) else {
        return;
    };

    if lang_key.has_children() {
        for lang_entry in lang_key.children() {
            let Some(l10n_key) = lang_entry.attribute("name") else {
                continue;
            };
            lang_entry.children().for_each(|n| {
                if n.tag_name().name() != "" {
                    if let Some(l10n_value) = n.text() {
                        mod_detail.add_lang(n.tag_name().name(), l10n_key, l10n_value);
                    }
                }
            });
        }
    }

    if let Some(prefix) = lang_key.attribute("filenamePrefix") {
        for file_to_scan in file_list.iter().filter(|n| n.name.starts_with(prefix)) {
            let Ok(l10n_contents) = file_handle.as_text(&file_to_scan.name) else {
                continue;
            };
            let Ok(l10n_tree) = roxmltree::Document::parse(&l10n_contents) else {
                continue;
            };
            let lang_code =
                &file_to_scan.name[file_to_scan.name.len() - 6..file_to_scan.name.len() - 4];

            //<text name="key" text="value" /> style
            for entry in l10n_tree.descendants().filter(|n| n.has_tag_name("text")) {
                let Some(l10n_key) = entry.attribute("name") else {
                    continue;
                };
                let Some(l10n_value) = entry.attribute("text") else {
                    continue;
                };
                mod_detail.add_lang(lang_code, l10n_key, l10n_value);
            }

            // <e k="key" v="value"/> style
            for entry in l10n_tree.descendants().filter(|n| n.has_tag_name("e")) {
                let Some(l10n_key) = entry.attribute("k") else {
                    continue;
                };
                let Some(l10n_value) = entry.attribute("v") else {
                    continue;
                };
                mod_detail.add_lang(lang_code, l10n_key, l10n_value);
            }
        }
    }
}

/// Extract an XML text element as a `u32` `Option`
fn xml_extract_text_as_opt_u32(xml_tree: &roxmltree::Document, key: &str) -> Option<u32> {
    xml_tree
        .descendants()
        .find(|n| n.has_tag_name(key))
        .and_then(|n| n.text())
        .and_then(|n| n.parse::<u32>().ok())
}

/// Extract an XML text element as a `String` `Option`
fn xml_extract_text_as_opt_string(xml_tree: &roxmltree::Document, key: &str) -> Option<String> {
    xml_tree
        .descendants()
        .find(|n| n.has_tag_name(key))
        .and_then(|n| n.text().map(std::string::ToString::to_string))
}

/// Quick Default for str->f32
#[inline]
fn default_float_parse(value: &str, default: f32) -> f32 {
    value.parse::<f32>().unwrap_or(default)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shared::files::AbstractNull;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    #[test]
    fn embedded_l10n_entries() {
        /* cSpell: disable */
        let minimum_xml = r#"<modDesc>
            <l10n>
                <text name="fillType_limestone"> <en>Limestone</en> <de>Kalkstein</de> <fr>Calcaire</fr> </text>
                <text name="fillType_gravel"> <en>Gravel</en> <de>Schotter</de> <fr>Gravier</fr> </text>
                <text name="fillType_sand"> <en>Sand</en> <de>Sand</de> <fr>Sable</fr> </text>
            </l10n>
            </modDesc>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut file_handle: Box<dyn AbstractFileHandle> = Box::new(AbstractNull::new().unwrap());
        let empty_file_list: Vec<FileDefinition> = vec![];
        let mut mod_detail = ModDetail::default();

        do_languages(
            &mut mod_detail,
            &mut file_handle,
            &minimum_doc,
            &empty_file_list,
        );
        let actual = json!(mod_detail.l10n);
        let expected = json!({
            "de": {
                "filltype_gravel": "Schotter",
                "filltype_limestone": "Kalkstein",
                "filltype_sand": "Sand"
            },
            "en": {
                "filltype_gravel": "Gravel",
                "filltype_limestone": "Limestone",
                "filltype_sand": "Sand"
            },
            "fr": {
                "filltype_gravel": "Gravier",
                "filltype_limestone": "Calcaire",
                "filltype_sand": "Sable"
            }
        });
        /* cSpell: enable */
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_eq!(actual, expected);
    }
}
