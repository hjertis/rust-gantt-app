# Rust Gantt App

A Gantt chart desktop application built with Rust and [egui](https://github.com/emilk/egui).

## Features

- Visual Gantt chart with colored task bars and progress indicators
- Task table with add/delete operations
- Timeline with day/week/month scales
- Zoom and scroll with Ctrl+scroll wheel
- Today-line indicator
- Milestones (diamond markers)
- Save/Load projects as JSON files
- Native file dialogs

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Keyboard Shortcuts

| Shortcut    | Action       |
| ----------- | ------------ |
| Ctrl+S      | Save project |
| Ctrl+Scroll | Zoom in/out  |

## Project File Format

Projects are saved as `.gantt.json` files containing tasks, dependencies, and metadata.
