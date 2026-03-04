use std::path::Path;

use crate::io;
use crate::ops::{self, CreateEdgeOptions, CreateNodeOptions};

/// Parse and execute a batch script against a diagram file.
/// Each line is one operation; errors are collected, not fatal.
/// Returns (messages, errors).
pub fn execute_batch(path: &Path, script: &str) -> (Vec<String>, Vec<String>) {
    let mut file = match io::load(path) {
        Ok(f) => f,
        Err(e) => return (Vec::new(), vec![e]),
    };

    let mut messages = Vec::new();
    let mut errors = Vec::new();

    for (line_num, raw_line) in script.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let tokens = match shell_tokenize(line) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("Line {}: parse error: {}", line_num + 1, e));
                continue;
            }
        };

        if tokens.is_empty() {
            continue;
        }

        let cmd = tokens[0].as_str();
        let args = &tokens[1..];

        match cmd {
            "node" => match parse_node_args(args) {
                Ok(opts) => {
                    let new_els = ops::create_node(&file.elements, opts);
                    let id = new_els[0].id.clone();
                    file.elements.extend(new_els);
                    messages.push(format!("Created node with ID: {}", id));
                }
                Err(e) => errors.push(format!("Line {}: {}", line_num + 1, e)),
            },
            "edge" => match parse_edge_args(args) {
                Ok(opts) => match ops::create_edge(&mut file.elements, opts) {
                    Ok(new_els) => {
                        let id = new_els[0].id.clone();
                        file.elements.extend(new_els);
                        messages.push(format!("Created edge with ID: {}", id));
                    }
                    Err(e) => errors.push(format!("Line {}: {}", line_num + 1, e)),
                },
                Err(e) => errors.push(format!("Line {}: {}", line_num + 1, e)),
            },
            "delete" => {
                if args.is_empty() {
                    errors.push(format!("Line {}: delete requires an argument", line_num + 1));
                } else {
                    let identifier = args[0].as_str();
                    match ops::delete_element(&mut file.elements, identifier) {
                        Ok(msg) => messages.push(msg),
                        Err(e) => errors.push(format!("Line {}: {}", line_num + 1, e)),
                    }
                }
            }
            _ => {
                errors.push(format!("Line {}: unknown command \"{}\"", line_num + 1, cmd));
            }
        }
    }

    // Save if we had any successful operations
    if !messages.is_empty() {
        if let Err(e) = io::save(path, &file) {
            errors.push(format!("Failed to save: {}", e));
        }
    }

    (messages, errors)
}

fn parse_node_args(args: &[String]) -> Result<CreateNodeOptions, String> {
    // First positional arg is label, rest are --flags
    if args.is_empty() {
        return Err("node requires a label".to_string());
    }

    let mut label = String::new();
    let mut shape = "rectangle".to_string();
    let mut color = "light-blue".to_string();
    let mut link: Option<String> = None;
    let mut x: Option<f64> = None;
    let mut y: Option<f64> = None;
    let mut width: Option<f64> = None;
    let mut height: Option<f64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--shape" | "-s" => {
                i += 1;
                shape = get_arg(args, i, "shape")?;
            }
            "--color" | "-c" => {
                i += 1;
                color = get_arg(args, i, "color")?;
            }
            "--link" => {
                i += 1;
                link = Some(get_arg(args, i, "link")?);
            }
            "--x" => {
                i += 1;
                x = Some(parse_f64(args, i, "x")?);
            }
            "--y" => {
                i += 1;
                y = Some(parse_f64(args, i, "y")?);
            }
            "--width" => {
                i += 1;
                width = Some(parse_f64(args, i, "width")?);
            }
            "--height" => {
                i += 1;
                height = Some(parse_f64(args, i, "height")?);
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown flag: {}", other));
            }
            _ => {
                if label.is_empty() {
                    label = args[i].clone();
                } else {
                    return Err(format!("unexpected argument: {}", args[i]));
                }
            }
        }
        i += 1;
    }

    if label.is_empty() {
        return Err("node requires a label".to_string());
    }

    // Handle \n in labels
    let label = label.replace("\\n", "\n");

    Ok(CreateNodeOptions {
        label,
        shape,
        color,
        link,
        x,
        y,
        width,
        height,
    })
}

fn parse_edge_args(args: &[String]) -> Result<CreateEdgeOptions, String> {
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;
    let mut label: Option<String> = None;
    let mut style = "solid".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" | "-f" => {
                i += 1;
                from = Some(get_arg(args, i, "from")?);
            }
            "--to" | "-t" => {
                i += 1;
                to = Some(get_arg(args, i, "to")?);
            }
            "--label" | "-l" => {
                i += 1;
                label = Some(get_arg(args, i, "label")?);
            }
            "--style" => {
                i += 1;
                style = get_arg(args, i, "style")?;
            }
            other => return Err(format!("unexpected argument: {}", other)),
        }
        i += 1;
    }

    let from = from.ok_or("edge requires --from")?;
    let to = to.ok_or("edge requires --to")?;

    Ok(CreateEdgeOptions {
        from,
        to,
        label,
        style,
    })
}

fn get_arg(args: &[String], i: usize, name: &str) -> Result<String, String> {
    args.get(i)
        .cloned()
        .ok_or_else(|| format!("--{} requires a value", name))
}

fn parse_f64(args: &[String], i: usize, name: &str) -> Result<f64, String> {
    let s = get_arg(args, i, name)?;
    s.parse::<f64>()
        .map_err(|_| format!("--{}: expected number, got \"{}\"", name, s))
}

/// Simple shell-style tokenizer: splits on whitespace, respects double quotes.
fn shell_tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            '\\' if in_quotes => {
                if let Some(&next) = chars.peek() {
                    match next {
                        '"' | '\\' => {
                            chars.next();
                            current.push(next);
                        }
                        'n' => {
                            chars.next();
                            current.push('\n');
                        }
                        _ => current.push(c),
                    }
                } else {
                    current.push(c);
                }
            }
            _ => current.push(c),
        }
    }

    if in_quotes {
        return Err("unterminated quote".to_string());
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}
