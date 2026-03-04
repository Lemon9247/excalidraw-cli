use std::fs;
use std::path::Path;

use crate::model::ExcalidrawFile;

pub fn load(path: &Path) -> Result<ExcalidrawFile, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

pub fn save(path: &Path, file: &ExcalidrawFile) -> Result<(), String> {
    let json = serde_json::to_string_pretty(file)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::write(path, &json)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

pub fn init_file(path: &Path) -> Result<(), String> {
    let file = ExcalidrawFile {
        elements: Vec::new(),
        extra: serde_json::Map::new(),
    };
    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}
