use std::collections::{HashMap, HashSet};

use crate::model::*;

// --- Emoji mapping ---

fn hex_to_rgb(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.trim().trim_start_matches('#');
    let hex = if hex.len() == 3 {
        hex.chars()
            .map(|c| format!("{}{}", c, c))
            .collect::<String>()
    } else if hex.len() == 6 {
        hex.to_string()
    } else {
        return None;
    };
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    Some((r, g, b))
}

fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;

    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        (((g - b) / delta) % 6.0) * 60.0
    } else if max == g {
        ((b - r) / delta + 2.0) * 60.0
    } else {
        ((r - g) / delta + 4.0) * 60.0
    };
    let h = (h + 360.0) % 360.0;

    (h, s, l)
}

fn emoji_for_color_and_shape(hex: &str, shape: &str) -> Option<&'static str> {
    let shape = match shape {
        "rectangle" => "rectangle",
        "ellipse" => "ellipse",
        "diamond" => "diamond",
        _ => return None,
    };

    let (r, g, b) = hex_to_rgb(hex)?;
    let (h, s, l) = rgb_to_hsl(r, g, b);

    if s <= 0.15 {
        // Gray
        let tone = if l < 0.5 { "dark" } else { "light" };
        return match (tone, shape) {
            ("dark", "rectangle") => Some("⬛"),
            ("dark", "ellipse") => Some("⚫"),
            ("dark", "diamond") => Some("◇"),
            ("light", "rectangle") => Some("⬜"),
            ("light", "ellipse") => Some("⚪"),
            ("light", "diamond") => Some("◆"),
            _ => None,
        };
    }

    let family = if h < 20.0 || h >= 340.0 {
        "red"
    } else if h < 50.0 {
        "orange"
    } else if h < 70.0 {
        "yellow"
    } else if h < 170.0 {
        "green"
    } else if h < 250.0 {
        "blue"
    } else {
        "purple"
    };

    match (family, shape) {
        ("red", "rectangle") => Some("🟥"),
        ("red", "ellipse") => Some("🔴"),
        ("red", "diamond") => Some("♦️"),
        ("orange", "rectangle") => Some("🟧"),
        ("orange", "ellipse") => Some("🟠"),
        ("orange", "diamond") => Some("🔶"),
        ("yellow", "rectangle") => Some("🟨"),
        ("yellow", "ellipse") => Some("🟡"),
        ("green", "rectangle") => Some("🟩"),
        ("green", "ellipse") => Some("🟢"),
        ("blue", "rectangle") => Some("🟦"),
        ("blue", "ellipse") => Some("🔵"),
        ("blue", "diamond") => Some("🔷"),
        ("purple", "rectangle") => Some("🟪"),
        ("purple", "ellipse") => Some("🟣"),
        _ => None,
    }
}

// --- Parse elements into Diagram ---

pub fn parse_diagram(elements: &[Element]) -> Diagram {
    let live: Vec<&Element> = elements.iter().filter(|e| !e.is_deleted).collect();

    // Build label lookup: container_id -> text
    let mut text_by_container: HashMap<&str, &str> = HashMap::new();
    for el in &live {
        if el.element_type == "text" {
            if let (Some(ref cid), Some(ref text)) = (&el.container_id, &el.text) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    text_by_container.insert(cid.as_str(), trimmed);
                }
            }
        }
    }

    let node_shapes: HashSet<&str> = SHAPES.iter().copied().collect();

    let frames: Vec<Frame> = live
        .iter()
        .filter(|el| el.element_type == "frame")
        .map(|el| Frame {
            id: el.id.clone(),
            name: el.name.clone().filter(|n| !n.trim().is_empty()),
        })
        .collect();

    let nodes: Vec<Node> = live
        .iter()
        .filter(|el| {
            node_shapes.contains(el.element_type.as_str())
                || (el.element_type == "text" && el.container_id.is_none())
        })
        .map(|el| {
            let label = text_by_container
                .get(el.id.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    el.text
                        .as_ref()
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                });

            let bg = if el.background_color.is_empty()
                || el.background_color == "transparent"
            {
                None
            } else {
                Some(el.background_color.clone())
            };

            let emoji = if el.element_type != "text" {
                bg.as_deref()
                    .and_then(|hex| emoji_for_color_and_shape(hex, &el.element_type))
                    .map(|s| s.to_string())
            } else {
                None
            };

            Node {
                id: el.id.clone(),
                label,
                shape: el.element_type.clone(),
                background_color: bg,
                emoji,
                link: el.link.clone().filter(|l| !l.is_empty()),
                group_ids: el.group_ids.clone(),
                frame_id: el.frame_id.clone(),
            }
        })
        .collect();

    let edges: Vec<Edge> = live
        .iter()
        .filter(|el| el.element_type == "arrow")
        .map(|el| {
            let label = text_by_container
                .get(el.id.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    el.text
                        .as_ref()
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                });

            Edge {
                id: el.id.clone(),
                from: el
                    .start_binding
                    .as_ref()
                    .map(|b| b.element_id.clone()),
                to: el
                    .end_binding
                    .as_ref()
                    .map(|b| b.element_id.clone()),
                label,
                group_ids: el.group_ids.clone(),
                frame_id: el.frame_id.clone(),
            }
        })
        .collect();

    Diagram {
        nodes,
        edges,
        frames,
    }
}

// --- Format Diagram as Markdown ---

fn node_inline(node: Option<&Node>) -> String {
    match node {
        None => "?".to_string(),
        Some(n) => {
            let name = n.label.as_deref().unwrap_or(&n.id);
            let linked = match &n.link {
                Some(url) => format!("[{}]({})", name, url),
                None => name.to_string(),
            };
            match &n.emoji {
                Some(e) => format!("{} {}", e, linked),
                None => linked,
            }
        }
    }
}

fn edge_line(
    edge: &Edge,
    node_map: &HashMap<&str, &Node>,
) -> String {
    let from_node = edge.from.as_deref().and_then(|id| node_map.get(id).copied());
    let to_node = edge.to.as_deref().and_then(|id| node_map.get(id).copied());
    let from_str = node_inline(from_node);
    let to_str = node_inline(to_node);
    let connector = match &edge.label {
        Some(l) => format!("--({})-->", l),
        None => "-->".to_string(),
    };
    let self_loop = match (&edge.from, &edge.to) {
        (Some(f), Some(t)) if f == t => " (self-loop)",
        _ => "",
    };
    format!("{} {} {}{}", from_str, connector, to_str, self_loop)
}

fn frame_title(frame_id: Option<&str>, frame_name: Option<&str>) -> String {
    match frame_id {
        None => "No Frame".to_string(),
        Some(id) => match frame_name {
            Some(name) => format!("Frame: {}", name),
            None => format!("Frame: {}", id),
        },
    }
}

pub fn format_diagram_markdown(diagram: &Diagram) -> String {
    if diagram.nodes.is_empty() && diagram.edges.is_empty() {
        return "No nodes or edges.".to_string();
    }

    let node_map: HashMap<&str, &Node> = diagram
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();

    let frame_map: HashMap<&str, &Frame> = diagram
        .frames
        .iter()
        .map(|f| (f.id.as_str(), f))
        .collect();

    // Connected node IDs (referenced by edges)
    let connected: HashSet<&str> = diagram
        .edges
        .iter()
        .flat_map(|e| {
            let mut ids = Vec::new();
            if let Some(ref f) = e.from { ids.push(f.as_str()); }
            if let Some(ref t) = e.to { ids.push(t.as_str()); }
            ids
        })
        .collect();

    // Group items by frame, then by group
    struct GroupBucket {
        title: String,
        edges: Vec<String>,
        nodes: Vec<String>,
    }

    // frame_id -> group_id -> bucket
    let mut sections: HashMap<Option<&str>, HashMap<Option<&str>, GroupBucket>> = HashMap::new();

    // Add edges
    for edge in &diagram.edges {
        let frame_key = edge.frame_id.as_deref();
        let group_key = edge.group_ids.first().map(|s| s.as_str());
        let buckets = sections.entry(frame_key).or_default();
        let bucket = buckets.entry(group_key).or_insert_with(|| GroupBucket {
            title: match group_key {
                Some(g) => format!("Group {}", g),
                None => "Ungrouped".to_string(),
            },
            edges: Vec::new(),
            nodes: Vec::new(),
        });
        bucket.edges.push(edge_line(edge, &node_map));
    }

    // Add disconnected nodes
    for node in &diagram.nodes {
        if connected.contains(node.id.as_str()) {
            continue;
        }
        let frame_key = node.frame_id.as_deref();
        let group_key = node.group_ids.first().map(|s| s.as_str());
        let buckets = sections.entry(frame_key).or_default();
        let bucket = buckets.entry(group_key).or_insert_with(|| GroupBucket {
            title: match group_key {
                Some(g) => format!("Group {}", g),
                None => "Ungrouped".to_string(),
            },
            edges: Vec::new(),
            nodes: Vec::new(),
        });
        bucket.nodes.push(node_inline(Some(node)));
    }

    // Add empty frames
    for frame in &diagram.frames {
        sections.entry(Some(frame.id.as_str())).or_default();
    }

    // Render
    let mut output = Vec::new();

    for (frame_id, groups) in &sections {
        let frame_name = frame_id.and_then(|id| frame_map.get(id).and_then(|f| f.name.as_deref()));
        let title = frame_title(*frame_id, frame_name);
        let mut section = format!("## {}\n", title);

        if groups.is_empty() {
            section.push_str("No nodes or edges.");
        } else {
            let group_strs: Vec<String> = groups
                .values()
                .map(|bucket| {
                    let edges: Vec<&str> = bucket.edges.iter().map(|s| s.as_str()).collect();
                    let nodes: Vec<&str> = bucket.nodes.iter().map(|s| s.as_str()).collect();

                    let all: Vec<&str> = edges.into_iter().chain(nodes).collect();

                    let body = if all.is_empty() {
                        "- none".to_string()
                    } else {
                        all.join("\n")
                    };
                    format!("### {}\n{}", bucket.title, body)
                })
                .collect();
            section.push_str(&group_strs.join("\n\n"));
        }

        output.push(section);
    }

    output.join("\n\n").trim_end().to_string()
}
