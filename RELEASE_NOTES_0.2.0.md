# Rust Gantt App v0.2.0

This release introduces hierarchical planning and a much smoother dependency workflow.

## Highlights

- **Task hierarchy**: Create subtasks under a parent task (single-level nesting).
- **Summary bars**: Parent tasks are rendered as summary bars with aggregate timeline + progress.
- **Collapse/expand groups**: Hide/show child rows in both chart and task list.
- **Task context menus**: Right-click tasks for quick actions (add subtask, delete task/group).
- **Dependency picker in editor**: Add FS/SS/FF/SF links without Shift+drag.
  - Child tasks can only link to siblings in the same group.
  - Top-level tasks can only link to top-level tasks.
- **More stable UI**:
  - Fixed dependency-arrow routing edge cases.
  - Fixed drifting context menu placement.
  - Fixed sidebar width expansion caused by control sizing.
  - Replaced unstable glyphs with reliable icon rendering.

## Compatibility / Notes

- Project file format remains JSON-based (`.gantt.json`).
- Existing projects continue to load; parent date/progress values are derived from children when applicable.

## Build

```bash
cargo run --release
```

## Suggested Git tag

`v0.2.0`
