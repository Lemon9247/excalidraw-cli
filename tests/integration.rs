use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/debug/excalidraw-rs");
    path
}

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("excalidraw-rs-test");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn run(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(binary())
        .args(args)
        .output()
        .expect("failed to execute binary");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

// --- Tests ---

#[test]
fn test_init_and_read_excalidraw() {
    let path = temp_file("test_init.excalidraw");
    let _ = std::fs::remove_file(&path);

    let (stdout, _, ok) = run(&["init", path.to_str().unwrap()]);
    assert!(ok, "init failed");
    assert!(stdout.contains("Created"));

    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok, "read failed");
    assert!(stdout.contains("No nodes or edges"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_create_node() {
    let path = temp_file("test_node.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    let (stdout, _, ok) = run(&[
        "create-node",
        path.to_str().unwrap(),
        "--label", "Hello World",
        "--shape", "rectangle",
        "--color", "light-blue",
    ]);
    assert!(ok, "create-node failed");
    assert!(stdout.contains("Created node with ID:"));

    // Read should show the node
    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok, "read failed");
    assert!(stdout.contains("Hello World"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_create_node_with_position() {
    let path = temp_file("test_node_pos.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    let (stdout, _, ok) = run(&[
        "create-node",
        path.to_str().unwrap(),
        "--label", "Positioned",
        "--x", "200",
        "--y", "300",
    ]);
    assert!(ok, "create-node failed");
    assert!(stdout.contains("Created node with ID:"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_create_edge() {
    let path = temp_file("test_edge.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    // Create two nodes
    run(&[
        "create-node", path.to_str().unwrap(),
        "--label", "Source",
        "--x", "0", "--y", "0",
    ]);
    run(&[
        "create-node", path.to_str().unwrap(),
        "--label", "Target",
        "--x", "300", "--y", "0",
    ]);

    // Create edge between them
    let (stdout, _, ok) = run(&[
        "create-edge", path.to_str().unwrap(),
        "--from", "Source",
        "--to", "Target",
        "--label", "connects",
    ]);
    assert!(ok, "create-edge failed");
    assert!(stdout.contains("Created edge with ID:"));

    // Read should show the edge
    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok, "read failed");
    assert!(stdout.contains("Source"));
    assert!(stdout.contains("Target"));
    assert!(stdout.contains("connects"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_create_edge_not_found() {
    let path = temp_file("test_edge_err.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);
    run(&[
        "create-node", path.to_str().unwrap(),
        "--label", "Only",
    ]);

    let (_, stderr, ok) = run(&[
        "create-edge", path.to_str().unwrap(),
        "--from", "Only",
        "--to", "Missing",
    ]);
    assert!(!ok, "should fail for missing target");
    assert!(stderr.contains("Could not find"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_delete_by_label() {
    let path = temp_file("test_delete.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    run(&[
        "create-node", path.to_str().unwrap(),
        "--label", "ToDelete",
    ]);

    let (stdout, _, ok) = run(&[
        "delete", path.to_str().unwrap(),
        "--id", "ToDelete",
    ]);
    assert!(ok, "delete failed");
    assert!(stdout.contains("Deleted element:"));

    // Should be gone from read
    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok, "read failed");
    assert!(stdout.contains("No nodes or edges"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_delete_not_found() {
    let path = temp_file("test_delete_err.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    let (_, stderr, ok) = run(&[
        "delete", path.to_str().unwrap(),
        "--id", "NoSuchNode",
    ]);
    assert!(!ok);
    assert!(stderr.contains("Could not find"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_batch_mode() {
    let path = temp_file("test_batch.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    let script = r#"
# Create some nodes
node "Alpha" --color blue --x 0 --y 0
node "Beta" --color green --x 300 --y 0

# Connect them
edge --from "Alpha" --to "Beta" --label "depends on"
"#;

    let (stdout, stderr, ok) = run(&[
        "batch", path.to_str().unwrap(),
        "--script", script,
    ]);
    assert!(ok, "batch failed: {}", stderr);
    assert!(stdout.contains("Created node"));
    assert!(stdout.contains("Created edge"));

    // Read to verify
    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok);
    assert!(stdout.contains("Alpha"));
    assert!(stdout.contains("Beta"));
    assert!(stdout.contains("depends on"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_batch_delete() {
    let path = temp_file("test_batch_del.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    let script = r#"
node "Keep" --x 0 --y 0
node "Remove" --x 200 --y 0
delete "Remove"
"#;

    let (stdout, stderr, ok) = run(&[
        "batch", path.to_str().unwrap(),
        "--script", script,
    ]);
    assert!(ok, "batch failed: {}", stderr);

    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok);
    assert!(stdout.contains("Keep"));
    assert!(!stdout.contains("Remove"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_all_colors() {
    let path = temp_file("test_colors.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    let colors = &[
        "transparent", "light-blue", "light-green", "light-yellow",
        "light-red", "light-orange", "light-purple",
        "blue", "green", "yellow", "red", "orange", "purple",
    ];

    for color in colors {
        let (_, _, ok) = run(&[
            "create-node", path.to_str().unwrap(),
            "--label", &format!("Node-{}", color),
            "--color", color,
        ]);
        assert!(ok, "Failed for color: {}", color);
    }

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_all_shapes() {
    let path = temp_file("test_shapes.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    for shape in &["rectangle", "ellipse", "diamond"] {
        let (_, _, ok) = run(&[
            "create-node", path.to_str().unwrap(),
            "--label", &format!("Node-{}", shape),
            "--shape", shape,
        ]);
        assert!(ok, "Failed for shape: {}", shape);
    }

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_dashed_edge() {
    let path = temp_file("test_dashed.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    run(&["create-node", path.to_str().unwrap(), "--label", "A", "--x", "0", "--y", "0"]);
    run(&["create-node", path.to_str().unwrap(), "--label", "B", "--x", "300", "--y", "0"]);

    let (stdout, _, ok) = run(&[
        "create-edge", path.to_str().unwrap(),
        "--from", "A", "--to", "B",
        "--style", "dashed",
    ]);
    assert!(ok, "dashed edge failed");
    assert!(stdout.contains("Created edge"));

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_node_with_link() {
    let path = temp_file("test_link.excalidraw");
    let _ = std::fs::remove_file(&path);
    run(&["init", path.to_str().unwrap()]);

    run(&[
        "create-node", path.to_str().unwrap(),
        "--label", "Linked",
        "--link", "https://example.com",
    ]);

    let (stdout, _, ok) = run(&["read", path.to_str().unwrap()]);
    assert!(ok);
    assert!(stdout.contains("[Linked](https://example.com)"));

    std::fs::remove_file(&path).ok();
}
