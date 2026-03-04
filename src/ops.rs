use rand::Rng;
use serde_json::Map;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::*;
use crate::query;

// --- ID generation ---

pub fn generate_id() -> String {
    use rand::distributions::Alphanumeric;
    let mut rng = rand::thread_rng();
    let part1: String = (0..13).map(|_| rng.sample(Alphanumeric) as char).collect();
    let part2: String = (0..13).map(|_| rng.sample(Alphanumeric) as char).collect();
    format!("{}{}", part1, part2).to_lowercase()
}

fn generate_seed() -> u32 {
    rand::thread_rng().gen_range(0..2147483647)
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// --- Text sizing ---

pub fn estimate_text_width(text: &str, font_size: f64) -> f64 {
    let max_line_len = text
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    max_line_len as f64 * font_size * 0.6
}

pub fn estimate_text_height(text: &str, font_size: f64) -> f64 {
    let line_count = text.lines().count().max(1);
    line_count as f64 * font_size * 1.25
}

// --- Base element ---

fn create_base_element(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    stroke_color: &str,
    background_color: &str,
    stroke_width: f64,
    stroke_style: &str,
    link: Option<String>,
) -> Element {
    Element {
        id: generate_id(),
        element_type: String::new(), // caller sets this
        x,
        y,
        width,
        height,
        is_deleted: false,
        text: None,
        original_text: None,
        container_id: None,
        font_size: None,
        font_family: None,
        text_align: None,
        vertical_align: None,
        line_height: None,
        auto_resize: None,
        bound_elements: None,
        start_binding: None,
        end_binding: None,
        points: None,
        stroke_color: stroke_color.to_string(),
        background_color: background_color.to_string(),
        stroke_style: stroke_style.to_string(),
        stroke_width,
        angle: 0.0,
        fill_style: "solid".to_string(),
        roughness: 1,
        opacity: 100,
        group_ids: Vec::new(),
        frame_id: None,
        seed: generate_seed(),
        version: 1,
        version_nonce: generate_seed(),
        updated: now_millis(),
        link,
        locked: false,
        roundness: None,
        name: None,
        extra: Map::new(),
    }
}

// --- Create node ---

pub struct CreateNodeOptions {
    pub label: String,
    pub shape: String,
    pub color: String,
    pub link: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

/// Creates a shape element + bound text element. Returns 2 elements.
pub fn create_node(
    elements: &[Element],
    opts: CreateNodeOptions,
) -> Vec<Element> {
    let colors = get_color(&opts.color);
    let font_size = 16.0;
    let text_width = estimate_text_width(&opts.label, font_size);
    let text_height = estimate_text_height(&opts.label, font_size);
    let padding = 20.0;
    let width = opts.width.unwrap_or((text_width + padding * 2.0).max(100.0));
    let height = opts.height.unwrap_or((text_height + padding * 2.0).max(60.0));

    let (x, y) = match (opts.x, opts.y) {
        (Some(x), Some(y)) => (x, y),
        _ => query::calculate_next_position(elements),
    };

    let mut shape = create_base_element(
        x, y, width, height,
        colors.stroke,
        colors.background,
        1.4,
        "solid",
        opts.link,
    );
    shape.element_type = opts.shape.clone();
    shape.roundness = if opts.shape == "rectangle" {
        Some(Roundness { roundness_type: 3 })
    } else {
        None
    };

    let mut text = create_base_element(
        x + (width - text_width) / 2.0,
        y + (height - text_height) / 2.0,
        text_width,
        text_height,
        "#1e1e1e",
        "transparent",
        0.0,
        "solid",
        None,
    );
    text.element_type = "text".to_string();
    text.text = Some(opts.label.clone());
    text.original_text = Some(opts.label);
    text.font_size = Some(font_size);
    text.font_family = Some(1);
    text.text_align = Some("center".to_string());
    text.vertical_align = Some("middle".to_string());
    text.line_height = Some(1.25);
    text.auto_resize = Some(true);
    text.container_id = Some(shape.id.clone());

    shape.bound_elements = Some(vec![BoundRef {
        id: text.id.clone(),
        ref_type: "text".to_string(),
    }]);

    vec![shape, text]
}

// --- Create edge ---

struct Anchor {
    x: f64,
    y: f64,
}

fn get_node_anchors(el: &Element) -> (Anchor, Anchor, Anchor, Anchor, Anchor) {
    let (x, y, w, h) = (el.x, el.y, el.width, el.height);
    (
        Anchor { x: x + w / 2.0, y },           // top
        Anchor { x: x + w / 2.0, y: y + h },    // bottom
        Anchor { x, y: y + h / 2.0 },            // left
        Anchor { x: x + w, y: y + h / 2.0 },     // right
        Anchor { x: x + w / 2.0, y: y + h / 2.0 }, // center
    )
}

fn calculate_edge_anchors(from: &Element, to: &Element) -> (Anchor, Anchor) {
    let (ftop, fbottom, fleft, fright, fcenter) = get_node_anchors(from);
    let (ttop, tbottom, tleft, tright, tcenter) = get_node_anchors(to);

    let dx = tcenter.x - fcenter.x;
    let dy = tcenter.y - fcenter.y;

    if dx.abs() > dy.abs() {
        if dx > 0.0 {
            (fright, tleft)
        } else {
            (fleft, tright)
        }
    } else if dy > 0.0 {
        (fbottom, ttop)
    } else {
        (ftop, tbottom)
    }
}

pub struct CreateEdgeOptions {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: String,
}

/// Creates an edge (arrow) between two nodes, resolving by ID or label.
/// Mutates the elements vec in place (updates boundElements on source/target).
/// Returns new elements to append (the arrow + optional label text).
pub fn create_edge(
    elements: &mut Vec<Element>,
    opts: CreateEdgeOptions,
) -> Result<Vec<Element>, String> {
    let from_idx = query::resolve_node_by_role(elements, &opts.from, "source")?;
    let to_idx = query::resolve_node_by_role(elements, &opts.to, "target")?;

    let (from_anchor, to_anchor) = {
        let from_el = &elements[from_idx];
        let to_el = &elements[to_idx];
        calculate_edge_anchors(from_el, to_el)
    };

    let from_id = elements[from_idx].id.clone();
    let to_id = elements[to_idx].id.clone();

    let edge_id = generate_id();

    let mut arrow = create_base_element(
        from_anchor.x,
        from_anchor.y,
        (to_anchor.x - from_anchor.x).abs(),
        (to_anchor.y - from_anchor.y).abs(),
        "#1e1e1e",
        "transparent",
        1.4,
        &opts.style,
        None,
    );
    arrow.id = edge_id.clone();
    arrow.element_type = "arrow".to_string();
    arrow.roundness = Some(Roundness { roundness_type: 2 });
    arrow.points = Some(vec![
        [0.0, 0.0],
        [to_anchor.x - from_anchor.x, to_anchor.y - from_anchor.y],
    ]);
    arrow.start_binding = Some(Binding {
        element_id: from_id.clone(),
        focus: 0.0,
        gap: 4.0,
    });
    arrow.end_binding = Some(Binding {
        element_id: to_id.clone(),
        focus: 0.0,
        gap: 4.0,
    });

    // Extra arrow fields
    arrow.extra.insert("lastCommittedPoint".to_string(), serde_json::Value::Null);
    arrow.extra.insert("startArrowhead".to_string(), serde_json::Value::Null);
    arrow.extra.insert("endArrowhead".to_string(), serde_json::Value::String("arrow".to_string()));
    arrow.extra.insert("elbowed".to_string(), serde_json::Value::Bool(false));

    let mut result = Vec::new();

    // Label
    let label_id = format!("{}-label", edge_id);
    if let Some(ref label_text) = opts.label {
        arrow.bound_elements = Some(vec![BoundRef {
            id: label_id.clone(),
            ref_type: "text".to_string(),
        }]);

        let font_size = 16.0;
        let tw = estimate_text_width(label_text, font_size);
        let th = estimate_text_height(label_text, font_size);
        let mid_x = from_anchor.x + (to_anchor.x - from_anchor.x) / 2.0 - tw / 2.0;
        let mid_y = from_anchor.y + (to_anchor.y - from_anchor.y) / 2.0 - th / 2.0;

        let mut label_el = create_base_element(
            mid_x, mid_y, tw, th, "#1e1e1e", "transparent", 0.0, "solid", None,
        );
        label_el.id = label_id;
        label_el.element_type = "text".to_string();
        label_el.text = Some(label_text.clone());
        label_el.original_text = Some(label_text.clone());
        label_el.font_size = Some(font_size);
        label_el.font_family = Some(5);
        label_el.text_align = Some("center".to_string());
        label_el.vertical_align = Some("middle".to_string());
        label_el.line_height = Some(1.25);
        label_el.auto_resize = Some(true);
        label_el.container_id = Some(edge_id.clone());

        result.push(arrow);
        result.push(label_el);
    } else {
        arrow.bound_elements = Some(Vec::new());
        result.push(arrow);
    }

    // Update boundElements on source and target nodes
    let arrow_ref = BoundRef {
        id: edge_id,
        ref_type: "arrow".to_string(),
    };

    {
        let from_el = &mut elements[from_idx];
        match &mut from_el.bound_elements {
            Some(bounds) => bounds.push(arrow_ref.clone()),
            None => from_el.bound_elements = Some(vec![arrow_ref.clone()]),
        }
    }
    {
        let to_el = &mut elements[to_idx];
        match &mut to_el.bound_elements {
            Some(bounds) => bounds.push(arrow_ref),
            None => to_el.bound_elements = Some(vec![arrow_ref]),
        }
    }

    Ok(result)
}

// --- Delete ---

/// Delete an element by ID or label. Marks as deleted, cascades to bound text,
/// and removes references from other elements' boundElements.
pub fn delete_element(elements: &mut Vec<Element>, identifier: &str) -> Result<String, String> {
    let idx = query::resolve_node(elements, identifier)?;
    let target_id = elements[idx].id.clone();

    for el in elements.iter_mut() {
        if el.id == target_id || el.container_id.as_deref() == Some(&target_id) {
            el.is_deleted = true;
        }
        if let Some(ref mut bounds) = el.bound_elements {
            bounds.retain(|b| b.id != target_id);
        }
    }

    Ok(format!("Deleted element: {}", target_id))
}
