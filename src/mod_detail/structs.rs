use std::collections::{HashMap, HashSet};
// use serde::ser::{Serialize, Serializer, SerializeSeq};

#[derive(serde::Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum ModDetailError {
    FileReadFail,
    NotModModDesc,
}

#[derive(serde::Serialize)]
pub struct ModDetail {
    pub brands : String,
    pub icons  : String,
    pub issues : HashSet<ModDetailError>,
    pub items  : String,
    pub l10n   : LanguageDefinition,
}

impl ModDetail {
    pub fn new() -> Self {
        ModDetail {
            brands : String::new(),
            icons  : String::new(),
            issues : HashSet::new(),
            items  : String::new(),
            l10n   : HashMap::new()
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
