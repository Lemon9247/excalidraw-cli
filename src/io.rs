use std::fs;
use std::path::Path;

use regex::Regex;

use crate::model::ExcalidrawFile;

const WRAP_WIDTH: usize = 100;

fn is_obsidian_format(path: &Path) -> bool {
    path.to_string_lossy().ends_with(".excalidraw.md")
}

fn extract_json_from_md(md: &str) -> Result<String, String> {
    let re = Regex::new(r"```compressed-json\n([\s\S]*?)\n```").unwrap();
    let caps = re
        .captures(md)
        .ok_or("No compressed-json block found in .excalidraw.md file")?;
    let b64_raw = caps[1].replace(|c: char| c.is_whitespace(), "");
    let utf16 = lz_str::decompress_from_base64(&b64_raw)
        .ok_or("LZ-String decompression failed")?;
    String::from_utf16(&utf16)
        .map_err(|e| format!("Invalid UTF-16 in decompressed data: {}", e))
}

fn wrap_lines(s: &str, width: usize) -> String {
    let mut lines = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    for chunk in chars.chunks(width) {
        lines.push(chunk.iter().collect::<String>());
    }
    lines.join("\n")
}

fn pack_json_to_md(original_md: &str, json: &str) -> Result<String, String> {
    let compressed = lz_str::compress_to_base64(json);
    let wrapped = wrap_lines(&compressed, WRAP_WIDTH);
    let re = Regex::new(r"```compressed-json\n[\s\S]*?\n```").unwrap();
    if re.is_match(original_md) {
        let replacement = format!("```compressed-json\n{}\n```", wrapped);
        Ok(re.replace(original_md, replacement.as_str()).to_string())
    } else {
        Err("Cannot pack: original .excalidraw.md has no compressed-json block".to_string())
    }
}

pub fn load(path: &Path) -> Result<ExcalidrawFile, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let json_str = if is_obsidian_format(path) {
        extract_json_from_md(&content)?
    } else {
        content
    };

    serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

pub fn save(path: &Path, file: &ExcalidrawFile) -> Result<(), String> {
    let json = serde_json::to_string_pretty(file)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    if is_obsidian_format(path) {
        let original = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let updated = pack_json_to_md(&original, &json)?;
        let updated = update_text_elements(&updated, file);
        fs::write(path, updated)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    } else {
        fs::write(path, &json)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    }

    Ok(())
}

/// Create a new empty .excalidraw file
pub fn init_file(path: &Path) -> Result<(), String> {
    let file = ExcalidrawFile {
        elements: Vec::new(),
        extra: serde_json::Map::new(),
    };
    if is_obsidian_format(path) {
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        let compressed = lz_str::compress_to_base64(&json);
        let wrapped = wrap_lines(&compressed, WRAP_WIDTH);
        let md = format!(
            "---\n\nexcalidraw-plugin: parsed\ntags: [excalidraw]\n\n---\n\
             ==⚠  Switch to EXCALIDRAW VIEW in the MORE OPTIONS menu of this document. ⚠== \
             You can decompress Drawing data with the command palette: \
             'Decompress current Excalidraw file'. For more info check in plugin settings under 'Saving'\n\n\n\
             # Excalidraw Data\n\n\
             ## Text Elements\n\n\
             %%\n## Drawing\n```compressed-json\n{}\n```\n%%",
            wrapped
        );
        fs::write(path, md)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    } else {
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    }
    Ok(())
}

/// Update the Text Elements section in an .excalidraw.md file after modifications.
fn update_text_elements(md: &str, file: &ExcalidrawFile) -> String {
    let mut text_section = String::new();
    for el in &file.elements {
        if el.is_deleted {
            continue;
        }
        if el.element_type == "text" {
            if let Some(ref text) = el.text {
                text_section.push_str(&format!("{} ^{}\n\n", text, el.id));
            }
        }
    }

    // Replace the text elements section
    let re = Regex::new(r"## Text Elements\n([\s\S]*?)\n%%").unwrap();
    if re.is_match(md) {
        re.replace(md, format!("## Text Elements\n{}\n%%", text_section).as_str())
            .to_string()
    } else {
        md.to_string()
    }
}
