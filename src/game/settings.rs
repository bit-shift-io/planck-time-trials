use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Settings {
    pub player_name: Option<String>,
    pub show_debug_info: Option<bool>,
}

impl Settings {
    pub fn load() -> Self {
        let path = Path::new("settings.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Settings::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write("settings.json", content)
    }
}
