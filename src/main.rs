mod batch;
mod format;
mod io;
mod model;
mod ops;
mod query;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use format::{format_diagram_markdown, parse_diagram};
use model::COLOR_NAMES;
use ops::{CreateEdgeOptions, CreateNodeOptions};

#[derive(Parser)]
#[command(name = "excalidraw", about = "Excalidraw diagram CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new empty .excalidraw or .excalidraw.md file
    Init {
        /// Path for the new file
        path: PathBuf,
    },

    /// Print the diagram state as markdown
    Read {
        /// Path to .excalidraw or .excalidraw.md file
        path: PathBuf,
    },

    /// Create a new node in the diagram
    #[command(name = "create-node")]
    CreateNode {
        /// Path to .excalidraw or .excalidraw.md file
        path: PathBuf,

        /// Node label text (use \n for newlines)
        #[arg(short, long)]
        label: String,

        /// Node shape
        #[arg(short, long, default_value = "rectangle", value_parser = ["rectangle", "ellipse", "diamond"])]
        shape: String,

        /// Color preset
        #[arg(short, long, default_value = "light-blue",
              value_parser = clap::builder::PossibleValuesParser::new(COLOR_NAMES))]
        color: String,

        /// URL to link the node to
        #[arg(long)]
        link: Option<String>,

        /// X coordinate
        #[arg(long)]
        x: Option<f64>,

        /// Y coordinate
        #[arg(long)]
        y: Option<f64>,

        /// Node width
        #[arg(long)]
        width: Option<f64>,

        /// Node height
        #[arg(long)]
        height: Option<f64>,
    },

    /// Create an arrow connecting two nodes
    #[command(name = "create-edge")]
    CreateEdge {
        /// Path to .excalidraw or .excalidraw.md file
        path: PathBuf,

        /// Source node ID or label text
        #[arg(short, long)]
        from: String,

        /// Target node ID or label text
        #[arg(short, long)]
        to: String,

        /// Edge label text
        #[arg(short, long)]
        label: Option<String>,

        /// Line style
        #[arg(long, default_value = "solid", value_parser = ["solid", "dashed"])]
        style: String,
    },

    /// Delete a node or edge by ID or label
    Delete {
        /// Path to .excalidraw or .excalidraw.md file
        path: PathBuf,

        /// Element ID or label text to delete
        #[arg(short, long)]
        id: String,
    },

    /// Execute a batch script of operations
    Batch {
        /// Path to .excalidraw or .excalidraw.md file
        path: PathBuf,

        /// Batch script (read from stdin if not provided)
        #[arg(short, long)]
        script: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            if let Err(e) = io::init_file(&path) {
                eprintln!("{}", e);
                process::exit(1);
            }
            println!("Created {}", path.display());
        }

        Commands::Read { path } => {
            let file = load_or_exit(&path);
            let diagram = parse_diagram(&file.elements);
            println!("{}", format_diagram_markdown(&diagram));
        }

        Commands::CreateNode {
            path, label, shape, color, link, x, y, width, height,
        } => {
            let mut file = load_or_exit(&path);
            let label = label.replace("\\n", "\n");

            let new_elements = ops::create_node(
                &file.elements,
                CreateNodeOptions {
                    label,
                    shape,
                    color,
                    link,
                    x,
                    y,
                    width,
                    height,
                },
            );

            let id = new_elements[0].id.clone();
            file.elements.extend(new_elements);
            save_or_exit(&path, &file);
            println!("Created node with ID: {}", id);
        }

        Commands::CreateEdge {
            path, from, to, label, style,
        } => {
            let mut file = load_or_exit(&path);

            match ops::create_edge(
                &mut file.elements,
                CreateEdgeOptions {
                    from,
                    to,
                    label,
                    style,
                },
            ) {
                Ok(new_elements) => {
                    let id = new_elements[0].id.clone();
                    file.elements.extend(new_elements);
                    save_or_exit(&path, &file);
                    println!("Created edge with ID: {}", id);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Delete { path, id } => {
            let mut file = load_or_exit(&path);
            match ops::delete_element(&mut file.elements, &id) {
                Ok(msg) => {
                    save_or_exit(&path, &file);
                    println!("{}", msg);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Batch { path, script } => {
            let script_content = match script {
                Some(s) => s,
                None => {
                    use std::io::Read;
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buf)
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to read stdin: {}", e);
                            process::exit(1);
                        });
                    buf
                }
            };

            let (messages, errors) = batch::execute_batch(&path, &script_content);

            for msg in &messages {
                println!("{}", msg);
            }
            for err in &errors {
                eprintln!("{}", err);
            }

            if !errors.is_empty() {
                process::exit(1);
            }
        }
    }
}

fn load_or_exit(path: &PathBuf) -> model::ExcalidrawFile {
    match io::load(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn save_or_exit(path: &PathBuf, file: &model::ExcalidrawFile) {
    if let Err(e) = io::save(path, file) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
