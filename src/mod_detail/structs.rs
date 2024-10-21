use std::collections::{HashMap, HashSet};
// use serde::ser::{Serialize, Serializer, SerializeSeq};

#[derive(serde::Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum ModDetailError {
    FileReadFail,
    NotModModDesc,
    BrandMissingIcon,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetail {
    pub brands     : BrandDefinition,
    pub issues     : HashSet<ModDetailError>,
    #[serde(skip_serializing)]
    pub l10n       : LanguageDefinition,
    pub placeables : String,
    pub vehicles   : String,
}

impl ModDetail {
    pub fn new() -> Self {
        ModDetail {
            brands     : HashMap::new(),
            issues     : HashSet::new(),
            l10n       : HashMap::new(),
            placeables : String::new(),
            vehicles   : String::new(),
        }
    }

    pub fn add_issue(&mut self, issue : ModDetailError) -> &mut Self {
        self.issues.insert(issue);
        self
    }
    pub fn add_lang(&mut self, language : &str, key : &str, value : &str) -> &mut Self{
        let this_language = self.l10n.entry(language.to_string()).or_default();
    
        this_language.insert(key.to_string().to_lowercase(), value.to_string());

        self
        
    }
    pub fn add_brand(&mut self, key_name : &str, title: Option<&str>) -> &mut ModDetailBrand{
        let this_brand = self.brands.entry(key_name.to_string()).or_default();

        this_brand.title = match title {
            Some(title) => title.to_string(),
            None => key_name.to_string()
        };
        this_brand
    }
    pub fn pretty_print(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }
}

impl Default for ModDetail {
    fn default() -> Self {
        ModDetail::new()
    }
}

impl std::fmt::Display for ModDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}
type LanguageDefinition = HashMap<String, HashMap<String, String>>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailBrand {
    pub title : String,
    pub icon_file : Option<String>,
    pub icon_base : Option<String>
}

impl ModDetailBrand {
    fn new() -> Self {
        ModDetailBrand { title: String::new(), icon_file: None, icon_base: None }
    }
}
impl Default for ModDetailBrand {
    fn default() -> Self {
        ModDetailBrand::new()
    }
}

type BrandDefinition = HashMap<String, ModDetailBrand>;
