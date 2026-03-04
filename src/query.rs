use crate::model::{Element, SHAPES};

/// Result of trying to find a node by label.
pub enum FindResult {
    Found(usize), // index into elements
    NotFound,
    Ambiguous(Vec<(String, String)>), // (id, label) pairs
}

/// Find an element by ID — returns index.
pub fn find_by_id(elements: &[Element], id: &str) -> Option<usize> {
    elements.iter().position(|el| el.id == id && !el.is_deleted)
}

/// Find a node by its label text. Searches text elements that have a containerId,
/// returns the container (shape) element.
pub fn find_by_label(elements: &[Element], label: &str) -> FindResult {
    let normalized = label.trim().to_lowercase();

    let matching: Vec<(String, String)> = elements
        .iter()
        .filter(|el| {
            el.element_type == "text"
                && !el.is_deleted
                && el.container_id.is_some()
                && el.text
                    .as_ref()
                    .is_some_and(|t| t.trim().to_lowercase() == normalized)
        })
        .map(|el| {
            (
                el.container_id.clone().unwrap(),
                el.text.clone().unwrap().trim().to_string(),
            )
        })
        .collect();

    match matching.len() {
        0 => FindResult::NotFound,
        1 => {
            let container_id = &matching[0].0;
            match find_by_id(elements, container_id) {
                Some(idx) => FindResult::Found(idx),
                None => FindResult::NotFound,
            }
        }
        _ => FindResult::Ambiguous(matching),
    }
}

/// Resolve a node by ID or label. Returns index.
pub fn resolve_node(elements: &[Element], identifier: &str) -> Result<usize, String> {
    // Try by ID first
    if let Some(idx) = find_by_id(elements, identifier) {
        return Ok(idx);
    }

    // Try by label
    match find_by_label(elements, identifier) {
        FindResult::Found(idx) => Ok(idx),
        FindResult::NotFound => Err(format!("Error: Could not find node \"{}\"", identifier)),
        FindResult::Ambiguous(matches) => {
            let options: String = matches
                .iter()
                .map(|(id, label)| format!("  - \"{}\" (ID: {})", label, id))
                .collect::<Vec<_>>()
                .join("\n");
            Err(format!(
                "Error: Multiple nodes found with label \"{}\". Use ID instead:\n{}",
                identifier, options
            ))
        }
    }
}

/// Resolve with a role label for better error messages.
pub fn resolve_node_by_role(
    elements: &[Element],
    identifier: &str,
    role: &str,
) -> Result<usize, String> {
    resolve_node(elements, identifier).map_err(|e| {
        e.replace("Could not find node", &format!("Could not find {} node", role))
    })
}

/// Calculate the next auto-position below existing nodes.
pub fn calculate_next_position(elements: &[Element]) -> (f64, f64) {
    let shapes: std::collections::HashSet<&str> =
        SHAPES.iter().copied().collect();

    let nodes: Vec<&Element> = elements
        .iter()
        .filter(|el| shapes.contains(el.element_type.as_str()) && !el.is_deleted)
        .collect();

    if nodes.is_empty() {
        return (100.0, 100.0);
    }

    let mut max_y = f64::NEG_INFINITY;
    let mut corresponding_x = 100.0;

    for node in &nodes {
        let bottom = node.y + node.height;
        if bottom > max_y {
            max_y = bottom;
            corresponding_x = node.x;
        }
    }

    (corresponding_x, max_y + 50.0)
}
