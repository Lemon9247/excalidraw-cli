# excalidraw-cli

A fast CLI tool for editing [Excalidraw](https://excalidraw.com/) diagrams from the command line. Built in Rust for near-instant startup (~8ms).

Works with `.excalidraw` JSON files used by the Excalidraw app and the Obsidian Excalidraw plugin.

## Install

### Let your agent do it

Add something like this to your agent's project instructions:

> Use excalidraw-cli (https://github.com/Lemon9247/excalidraw-cli) to create and edit `.excalidraw` diagrams. Read the README for usage.

### Manual

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
# Binary at target/release/excalidraw-rs
```

## Usage

### Create a new diagram

```bash
excalidraw-rs init diagram.excalidraw
```

### Read diagram state

```bash
excalidraw-rs read diagram.excalidraw
```

Outputs a markdown summary of all nodes, edges, frames, and groups:

```
## No Frame
### Ungrouped
🟠 Sol --(light)--> 🔵 Terra
🔵 Terra --(orbits)--> 🟣 Luna
```

### Create a node

```bash
excalidraw-rs create-node diagram.excalidraw \
    --label "My Node" \
    --shape rectangle \
    --color light-blue
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--label` | `-l` | *(required)* | Node text (`\n` for newlines) |
| `--shape` | `-s` | `rectangle` | `rectangle`, `ellipse`, `diamond` |
| `--color` | `-c` | `light-blue` | Color preset (see below) |
| `--link` | | | URL to attach to the node |
| `--x` | | auto | X coordinate |
| `--y` | | auto | Y coordinate |
| `--width` | | auto | Explicit width |
| `--height` | | auto | Explicit height |

If `--x`/`--y` are omitted, the node is placed below the lowest existing node.

### Create an edge

```bash
excalidraw-rs create-edge diagram.excalidraw \
    --from "Source Node" \
    --to "Target Node" \
    --label "connects to"
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--from` | `-f` | *(required)* | Source node (ID or label) |
| `--to` | `-t` | *(required)* | Target node (ID or label) |
| `--label` | `-l` | | Edge label text |
| `--style` | | `solid` | `solid` or `dashed` |

### Delete an element

```bash
excalidraw-rs delete diagram.excalidraw --id "Node Label"
```

Accepts an element ID or label text. Cascades to bound text elements and cleans up references.

### Batch mode

Execute multiple operations with a single file load/save:

```bash
excalidraw-rs batch diagram.excalidraw --script '
# Comments start with #
node "API" --color blue --x 0 --y 0
node "Database" --color green --x 300 --y 0
edge --from "API" --to "Database" --label "queries"
delete "old node"
'
```

Or pipe from stdin:

```bash
cat ops.txt | excalidraw-rs batch diagram.excalidraw
```

Errors are collected and reported at the end — one bad line doesn't stop the rest.

## Colors

| Preset | Background | Stroke |
|--------|-----------|--------|
| `transparent` | — | `#1e1e1e` |
| `light-blue` | `#dae8fc` | `#6c8ebf` |
| `light-green` | `#d5e8d4` | `#82b366` |
| `light-yellow` | `#fff2cc` | `#d6b656` |
| `light-red` | `#f8cecc` | `#b85450` |
| `light-orange` | `#ffe6cc` | `#d79b00` |
| `light-purple` | `#e1d5e7` | `#9673a6` |
| `blue` | `#6c8ebf` | `#1e1e1e` |
| `green` | `#82b366` | `#1e1e1e` |
| `yellow` | `#d6b656` | `#1e1e1e` |
| `red` | `#b85450` | `#1e1e1e` |
| `orange` | `#d79b00` | `#1e1e1e` |
| `purple` | `#9673a6` | `#1e1e1e` |

## File Format

`.excalidraw` — Raw JSON, used by the Excalidraw app and the Obsidian Excalidraw plugin.

## License

MIT
