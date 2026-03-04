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
    let mut extra = serde_json::Map::new();
    extra.insert("type".into(), "excalidraw".into());
    extra.insert("version".into(), 2.into());
    extra.insert("source".into(), "https://github.com/Lemon9247/excalidraw-cli".into());
    extra.insert("appState".into(), serde_json::json!({
        "theme": "light",
        "viewBackgroundColor": "#ffffff",
        "currentItemStrokeColor": "#1e1e1e",
        "currentItemBackgroundColor": "transparent",
        "currentItemFillStyle": "solid",
        "currentItemStrokeWidth": 2,
        "currentItemStrokeStyle": "solid",
        "currentItemRoughness": 1,
        "currentItemOpacity": 100,
        "currentItemFontFamily": 5,
        "currentItemFontSize": 20,
        "currentItemTextAlign": "left",
        "currentItemStartArrowhead": null,
        "currentItemEndArrowhead": "arrow",
        "currentItemArrowType": "round",
        "gridSize": 20,
        "gridStep": 5,
        "gridModeEnabled": false,
        "objectsSnapModeEnabled": false
    }));
    extra.insert("files".into(), serde_json::json!({}));

    let file = ExcalidrawFile {
        elements: Vec::new(),
        extra,
    };
    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}
