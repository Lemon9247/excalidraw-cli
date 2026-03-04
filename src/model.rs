use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Top-level Excalidraw file structure.
/// Only `elements` is typed; everything else passes through.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExcalidrawFile {
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// A single Excalidraw element (shape, text, arrow, frame, etc.).
/// We type the fields we read/write; everything else is in `extra`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub is_deleted: bool,

    // Text fields (present when type == "text")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_resize: Option<bool>,

    // Binding fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_elements: Option<Vec<BoundRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_binding: Option<Binding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_binding: Option<Binding>,

    // Arrow points
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<[f64; 2]>>,

    // Style
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default)]
    pub background_color: String,
    #[serde(default = "default_stroke_style")]
    pub stroke_style: String,
    #[serde(default)]
    pub stroke_width: f64,

    // Position / layout
    #[serde(default)]
    pub angle: f64,
    #[serde(default = "default_fill_style")]
    pub fill_style: String,
    #[serde(default = "default_roughness")]
    pub roughness: u32,
    #[serde(default = "default_opacity")]
    pub opacity: u32,
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<String>,
    #[serde(default)]
    pub seed: u32,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub version_nonce: u32,
    #[serde(default)]
    pub updated: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(default)]
    pub locked: bool,

    // Roundness
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roundness: Option<Roundness>,

    // Frame name (only for frames)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    // Everything else passes through untouched
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BoundRef {
    pub id: String,
    #[serde(rename = "type")]
    pub ref_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub element_id: String,
    #[serde(default)]
    pub focus: f64,
    #[serde(default = "default_gap")]
    pub gap: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Roundness {
    #[serde(rename = "type")]
    pub roundness_type: u32,
}

fn default_stroke_color() -> String {
    "#1e1e1e".to_string()
}
fn default_stroke_style() -> String {
    "solid".to_string()
}
fn default_fill_style() -> String {
    "solid".to_string()
}
fn default_roughness() -> u32 {
    1
}
fn default_opacity() -> u32 {
    100
}
fn default_version() -> u32 {
    1
}
fn default_gap() -> f64 {
    4.0
}

// --- Parsed diagram types (for read/format) ---

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub label: Option<String>,
    pub shape: String,
    pub background_color: Option<String>,
    pub emoji: Option<String>,
    pub link: Option<String>,
    pub group_ids: Vec<String>,
    pub frame_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub label: Option<String>,
    pub group_ids: Vec<String>,
    pub frame_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Diagram {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub frames: Vec<Frame>,
}

// --- Color presets ---

pub struct ColorConfig {
    pub background: &'static str,
    pub stroke: &'static str,
}

pub const COLOR_PRESETS: &[(&str, ColorConfig)] = &[
    ("transparent", ColorConfig { background: "transparent", stroke: "#1e1e1e" }),
    ("light-blue", ColorConfig { background: "#dae8fc", stroke: "#6c8ebf" }),
    ("light-green", ColorConfig { background: "#d5e8d4", stroke: "#82b366" }),
    ("light-yellow", ColorConfig { background: "#fff2cc", stroke: "#d6b656" }),
    ("light-red", ColorConfig { background: "#f8cecc", stroke: "#b85450" }),
    ("light-orange", ColorConfig { background: "#ffe6cc", stroke: "#d79b00" }),
    ("light-purple", ColorConfig { background: "#e1d5e7", stroke: "#9673a6" }),
    ("blue", ColorConfig { background: "#6c8ebf", stroke: "#1e1e1e" }),
    ("green", ColorConfig { background: "#82b366", stroke: "#1e1e1e" }),
    ("yellow", ColorConfig { background: "#d6b656", stroke: "#1e1e1e" }),
    ("red", ColorConfig { background: "#b85450", stroke: "#1e1e1e" }),
    ("orange", ColorConfig { background: "#d79b00", stroke: "#1e1e1e" }),
    ("purple", ColorConfig { background: "#9673a6", stroke: "#1e1e1e" }),
];

pub fn get_color(name: &str) -> &'static ColorConfig {
    COLOR_PRESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| c)
        .unwrap_or(&COLOR_PRESETS[1].1) // default: light-blue
}

pub const SHAPES: &[&str] = &["rectangle", "ellipse", "diamond"];
pub const EDGE_STYLES: &[&str] = &["solid", "dashed"];
pub const COLOR_NAMES: &[&str] = &[
    "transparent", "light-blue", "light-green", "light-yellow",
    "light-red", "light-orange", "light-purple",
    "blue", "green", "yellow", "red", "orange", "purple",
];
