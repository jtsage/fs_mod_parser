use crate::mod_detail::structs::{ModDetail, ModDetailError};
use crate::shared::files::{AbstractFileHandle, AbstractFolder, AbstractZipFile, FileDefinition};
use std::path::Path;

pub mod structs;

pub fn parse_to_json_pretty<P: AsRef<Path>>(full_path :P) -> String {
    parser(full_path).pretty_print()
}

pub fn parse_to_json<P: AsRef<Path>>(full_path :P) -> String {
    parser(full_path).to_string()
}

pub fn parser<P: AsRef<Path>>(full_path :P) -> ModDetail {
    let mut mod_detail = ModDetail::new();
    let is_folder = full_path.as_ref().is_dir();

    let mut abstract_file: Box<dyn AbstractFileHandle> = if is_folder 
        {
            if let Ok(archive) = AbstractFolder::new(full_path) {
                Box::new(archive)
            } else {
                mod_detail.issues.insert(ModDetailError::FileReadFail);
                return mod_detail;
            }
        } else if let Ok(archive) = AbstractZipFile::new(full_path) {
            Box::new(archive)
        } else {
            mod_detail.add_issue(ModDetailError::FileReadFail);
            return mod_detail;
        };

	let abstract_file_list = abstract_file.list();

	let Ok(mod_desc_content) = abstract_file.as_text("modDesc.xml") else {
        mod_detail.add_issue(ModDetailError::NotModModDesc);
        return mod_detail;
    };

    let Ok(mod_desc_doc) = roxmltree::Document::parse(&mod_desc_content) else {
        mod_detail.add_issue(ModDetailError::NotModModDesc);
        return mod_detail;
    };


	do_languages(&mut mod_detail, &mut abstract_file, &abstract_file_list, &mod_desc_doc);
    // do_farms(&mut save_record, &mut abstract_file);
    // do_placeables(&mut save_record, &mut abstract_file);
    // do_vehicles(&mut save_record, &mut abstract_file);
    // do_career(&mut save_record, &mut abstract_file);

    mod_detail
}

fn do_languages(
	mod_detail: &mut ModDetail,
	file_handle: &mut Box<dyn AbstractFileHandle>,
	file_list: &[FileDefinition],
	mod_desc_doc : &roxmltree::Document
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