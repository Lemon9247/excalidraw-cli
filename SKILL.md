---
name: excalidraw
description: Create and edit Excalidraw diagrams — add nodes, connect them with edges, delete elements, read diagram state.
---

# /excalidraw — Diagram Editing

Edit Excalidraw diagrams (`.excalidraw` JSON files).

## Script

The entrypoint is `excalidraw.sh` in the skill directory:

```bash
EXCLI="SKILL_DIR/excalidraw.sh"
```

### Init a new diagram

```bash
$EXCLI init "/path/to/diagram.excalidraw"
```

Creates an empty `.excalidraw` file.

### Read diagram state

```bash
$EXCLI read "/path/to/diagram.excalidraw"
```

Returns a markdown representation of all nodes, edges, frames, and groups.

### Create a node

```bash
$EXCLI create-node "/path/to/diagram.excalidraw" \
    --label "My Node" \
    --shape rectangle \
    --color light-blue
```

Options:
- `--label, -l` *(required)* — Node text. Use `\n` for newlines.
- `--shape, -s` — `rectangle` (default), `ellipse`, `diamond`
- `--color, -c` — Color preset (see below). Default: `light-blue`
- `--link` — URL to attach (e.g. `file://...`, `https://...`)
- `--x`, `--y` — Position. If omitted, auto-positions below the last node.
- `--width`, `--height` — Explicit size. If omitted, auto-sizes from label.

### Create an edge

```bash
$EXCLI create-edge "/path/to/diagram.excalidraw" \
    --from "Source Node" \
    --to "Target Node" \
    --label "connects to"
```

Options:
- `--from, -f` *(required)* — Source node ID or label text
- `--to, -t` *(required)* — Target node ID or label text
- `--label, -l` — Edge label
- `--style` — `solid` (default) or `dashed`

Nodes can be referenced by their label text or by their element ID (from `read` output or creation output).

### Delete an element

```bash
$EXCLI delete "/path/to/diagram.excalidraw" \
    --id "Node Label"
```

Options:
- `--id, -i` *(required)* — Element ID or label text to delete

### Batch mode

Execute multiple operations in a single file load/save cycle:

```bash
$EXCLI batch "/path/to/diagram.excalidraw" --script '
# Comments start with #
node "CLI" --color blue --x 60 --y 0
node "GUI" --color blue --x 340 --y 0
edge --from "CLI" --to "GUI" --label "calls"
delete "old node"
'
```

Or pipe from stdin:

```bash
cat operations.txt | $EXCLI batch "/path/to/diagram.excalidraw"
```

Batch commands: `node`, `edge`, `delete`. Each line uses the same flags as the corresponding subcommand. Errors are collected and reported at the end — one bad line doesn't stop the rest.

## Colors

| Preset | Use for |
|--------|---------|
| `transparent` | Minimal / wireframe |
| `light-blue` | Default, general purpose |
| `light-green` | Success, positive, data flow |
| `light-yellow` | Warning, notes, highlights |
| `light-red` | Error, danger, hot paths |
| `light-orange` | Attention, intermediate |
| `light-purple` | Special, abstract, meta |
| `blue` `green` `yellow` `red` `orange` `purple` | Bold/solid variants |

## Layout Tips

- **Read first.** Always `read` the diagram before modifying to understand existing layout and node positions.
- **Explicit coordinates** for anything that needs spatial meaning (flowcharts, architecture diagrams). Auto-positioning just stacks vertically.
- **Consistent spacing:** 200–300px horizontal gaps, 100–150px vertical gaps work well.
- **Use labels on edges** to make relationships self-documenting.
- **Color semantically** — pick a color scheme for the diagram and stick with it (e.g. blue=services, green=data, red=errors).
- **Use batch mode** for multi-operation edits — one file load/save instead of N.

## File Format

`.excalidraw` — Raw JSON, used by the Excalidraw app and the Obsidian Excalidraw plugin.

## Finding Diagrams

```bash
find ~/Documents/Obsidian/Claude/Excalidraw -name "*.excalidraw" | head -20
```
