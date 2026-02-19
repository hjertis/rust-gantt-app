# Changelog

All notable changes to this project are documented in this file.

## [0.2.0] - 2026-02-19

### Added

- Task hierarchy support with parent/child relationships (one nesting level).
- Summary (parent) bars rendered as bracket-style bars with aggregated progress.
- Collapse/expand controls for parent groups in both chart and task table.
- Right-click task context menu actions for adding subtasks and deleting tasks/groups.
- Subtask creation flow that inserts under the parent and keeps grouped ordering.
- Dependency picker in the task editor with type selection (FS/SS/FF/SF).
- Dependency creation from the editor with scoped candidates:
  - Child task: only siblings in the same parent group.
  - Top-level task: only other top-level tasks.
- In-editor dependency kind help text/tooltips.
- Phosphor icon font integration for consistent icon rendering.

### Changed

- Parent task dates and progress are now auto-calculated from children and shown read-only.
- Parent date ranges are recalculated after relevant task/dependency edits.
- Task deletion cascades to child tasks and related dependencies.
- Sidebar editor dependency controls improved for discoverability.
- App version bumped to `0.2.0`.

### Fixed

- Dependency arrow routing in overlap/backtrack cases to avoid crossing task bars.
- Context-menu positioning now stays fixed at click location.
- Sidebar width growth caused by dependency picker combo sizing.
- Missing-glyph/square icon issues by replacing unsupported glyphs and using icon font fallbacks.
- Build warnings from unused hierarchy helper methods.
