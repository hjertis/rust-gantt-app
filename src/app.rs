use chrono::NaiveDate;
use std::path::PathBuf;
use uuid::Uuid;

use crate::model::{Project, Task, TimelineViewport, UndoHistory};
use crate::model::task::TaskPriority;
use crate::ui;
use crate::ui::theme_manager::ThemeManager;

/// Main application state.
pub struct GanttApp {
    pub project: Project,
    pub viewport: TimelineViewport,
    pub file_path: Option<PathBuf>,
    pub selected_task: Option<Uuid>,

    // Dialog state
    pub show_add_task: bool,
    pub show_about: bool,
    pub show_csv_help: bool,
    pub new_task_name: String,
    pub new_task_start: String,
    pub new_task_end: String,
    pub new_task_start_date: NaiveDate,
    pub new_task_end_date: NaiveDate,
    pub new_task_is_milestone: bool,

    // Status message
    pub status_message: String,

    // Theme engine
    pub theme_manager: ThemeManager,

    // Undo / redo
    pub undo_history: UndoHistory,

    // Filter / search
    pub search_query: String,
    pub filter_priority: Option<TaskPriority>,

    // Pending actions from nested UI closures
    pub pending_add_subtask: Option<Uuid>,
    pub pending_add_dependency: Option<crate::model::task::Dependency>,
}

impl GanttApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Register Phosphor icon font as a fallback so icons render inline with text
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        _cc.egui_ctx.set_fonts(fonts);

        let project = Self::sample_project();
        let start = project
            .tasks
            .iter()
            .map(|t| t.start)
            .min()
            .unwrap_or_else(|| chrono::Local::now().date_naive())
            - chrono::Duration::days(7);
        let end = project
            .tasks
            .iter()
            .map(|t| t.end)
            .max()
            .unwrap_or_else(|| chrono::Local::now().date_naive())
            + chrono::Duration::days(30);

        let today = chrono::Local::now().date_naive();
        let default_start = today.format("%Y-%m-%d").to_string();
        let default_end = (today + chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();

        Self {
            project,
            viewport: TimelineViewport::new(start, end),
            file_path: None,
            selected_task: None,
            show_add_task: false,
            show_about: false,
            show_csv_help: false,
            new_task_name: String::new(),
            new_task_start: default_start.clone(),
            new_task_end: default_end.clone(),
            new_task_start_date: chrono::Local::now().date_naive(),
            new_task_end_date: chrono::Local::now().date_naive() + chrono::Duration::days(7),
            new_task_is_milestone: false,
            status_message: "Ready".to_string(),
            theme_manager: ThemeManager::new(),
            undo_history: UndoHistory::new(),
            search_query: String::new(),
            filter_priority: None,
            pending_add_subtask: None,
            pending_add_dependency: None,
        }
    }

    /// Generate a sample project for demonstration.
    fn sample_project() -> Project {
        let today = chrono::Local::now().date_naive();
        let mut project = Project::new("Sample Project");

        // ── Phase 1: Planning ───────────────────────────────────────
        let mut phase1 = Task::new(
            "Planning",
            today - chrono::Duration::days(5),
            today + chrono::Duration::days(8),
        );
        phase1.color = egui::Color32::from_rgb(70, 120, 180);

        let mut t1 = Task::new(
            "Project Kickoff",
            today - chrono::Duration::days(5),
            today - chrono::Duration::days(2),
        );
        t1.progress = 1.0;
        t1.color = egui::Color32::from_rgb(70, 130, 180);
        t1.parent_id = Some(phase1.id);

        let mut t2 = Task::new(
            "Requirements Gathering",
            today - chrono::Duration::days(2),
            today + chrono::Duration::days(5),
        );
        t2.progress = 0.6;
        t2.color = egui::Color32::from_rgb(60, 179, 113);
        t2.parent_id = Some(phase1.id);

        let mut m1 = Task::new_milestone("Planning Complete", today + chrono::Duration::days(8));
        m1.parent_id = Some(phase1.id);

        // ── Phase 2: Execution ──────────────────────────────────────
        let mut phase2 = Task::new(
            "Execution",
            today + chrono::Duration::days(6),
            today + chrono::Duration::days(30),
        );
        phase2.color = egui::Color32::from_rgb(180, 100, 50);

        let mut t3 = Task::new(
            "UI Design",
            today + chrono::Duration::days(6),
            today + chrono::Duration::days(18),
        );
        t3.progress = 0.0;
        t3.color = egui::Color32::from_rgb(218, 112, 214);
        t3.parent_id = Some(phase2.id);

        let mut t4 = Task::new(
            "Backend Development",
            today + chrono::Duration::days(6),
            today + chrono::Duration::days(28),
        );
        t4.progress = 0.0;
        t4.color = egui::Color32::from_rgb(106, 90, 205);
        t4.parent_id = Some(phase2.id);

        let mut t5 = Task::new(
            "Testing & QA",
            today + chrono::Duration::days(22),
            today + chrono::Duration::days(30),
        );
        t5.progress = 0.0;
        t5.color = egui::Color32::from_rgb(220, 20, 60);
        t5.parent_id = Some(phase2.id);

        let m2 = Task::new_milestone("Launch", today + chrono::Duration::days(32));

        // Sample dependencies between subtasks
        let deps = vec![
            crate::model::task::Dependency {
                from_task: t1.id,
                to_task: t2.id,
                kind: crate::model::task::DependencyKind::FinishToStart,
            },
            crate::model::task::Dependency {
                from_task: t2.id,
                to_task: t3.id,
                kind: crate::model::task::DependencyKind::FinishToStart,
            },
            crate::model::task::Dependency {
                from_task: t3.id,
                to_task: t4.id,
                kind: crate::model::task::DependencyKind::StartToStart,
            },
            crate::model::task::Dependency {
                from_task: t4.id,
                to_task: t5.id,
                kind: crate::model::task::DependencyKind::FinishToStart,
            },
            crate::model::task::Dependency {
                from_task: t5.id,
                to_task: m2.id,
                kind: crate::model::task::DependencyKind::FinishToStart,
            },
        ];

        // Order: parent, then children in sequence
        project.tasks = vec![phase1, t1, t2, m1, phase2, t3, t4, t5, m2];
        project.dependencies = deps;
        // Auto-calculate parent dates from children
        project.recalculate_parent_dates();
        project
    }

    // --- File operations ---

    pub fn new_project(&mut self) {
        self.project = Project::default();
        self.file_path = None;
        self.selected_task = None;
        self.undo_history.clear();
        self.status_message = "New project created".to_string();
    }

    pub fn open_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Gantt Project", &["gantt.json", "json"])
            .pick_file()
        {
            match crate::io::load_project(&path) {
                Ok(project) => {
                    self.project = project;
                    self.file_path = Some(path);
                    self.recalculate_viewport();
                    self.undo_history.clear();
                    self.status_message = "Project loaded".to_string();
                }
                Err(e) => {
                    self.status_message = format!("Error loading: {}", e);
                }
            }
        }
    }

    pub fn save_project(&mut self) {
        if let Some(ref path) = self.file_path.clone() {
            self.project.touch();
            match crate::io::save_project(&self.project, path) {
                Ok(()) => self.status_message = "Project saved".to_string(),
                Err(e) => self.status_message = format!("Error saving: {}", e),
            }
        } else {
            self.save_project_as();
        }
    }

    pub fn save_project_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Gantt Project", &["gantt.json", "json"])
            .set_file_name(&format!("{}.gantt.json", self.project.name))
            .save_file()
        {
            self.file_path = Some(path.clone());
            self.project.touch();
            match crate::io::save_project(&self.project, &path) {
                Ok(()) => self.status_message = "Project saved".to_string(),
                Err(e) => self.status_message = format!("Error saving: {}", e),
            }
        }
    }

    pub fn import_csv(&mut self) {
        // Guard: if current project has tasks, confirm before replacing
        if !self.project.tasks.is_empty() {
            let confirm = rfd::MessageDialog::new()
                .set_title("Import CSV")
                .set_description("This will replace the current project. Continue?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show();
            if confirm != rfd::MessageDialogResult::Yes {
                return;
            }
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV Files", &["csv", "txt"])
            .pick_file()
        {
            match crate::io::csv_import::import_csv(&path) {
                Ok((tasks, skipped)) => {
                    // Derive project name from filename
                    let proj_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Imported Project")
                        .to_string();

                    let count = tasks.len();
                    self.project = crate::model::Project::new(proj_name);
                    self.project.tasks = tasks;
                    self.file_path = None;
                    self.selected_task = None;
                    self.recalculate_viewport();

                    if skipped > 0 {
                        self.status_message = format!(
                            "Imported {} tasks ({} rows skipped)",
                            count, skipped
                        );
                    } else {
                        self.status_message = format!("Imported {} tasks", count);
                    }
                }
                Err(e) => {
                    self.status_message = format!("CSV import failed: {}", e);
                }
            }
        }
    }

    pub fn export_csv(&mut self) {
        if self.project.tasks.is_empty() {
            self.status_message = "Nothing to export — project has no tasks".to_string();
            return;
        }

        let default_name = format!("{}.csv", self.project.name);
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV Files", &["csv"])
            .set_file_name(&default_name)
            .save_file()
        {
            match crate::io::csv_export::export_csv(&self.project.tasks, &path) {
                Ok(count) => {
                    self.status_message = format!("Exported {} tasks to CSV", count);
                }
                Err(e) => {
                    self.status_message = format!("CSV export failed: {}", e);
                }
            }
        }
    }

    // --- Task operations ---

    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_history.undo(&self.project.tasks, &self.project.dependencies) {
            self.project.tasks = snap.tasks;
            self.project.dependencies = snap.dependencies;
            self.project.recalculate_parent_dates();
            self.project.sort_tasks_grouped();
            self.project.touch();
            self.selected_task = None;
            self.status_message = "Undo".to_string();
        }
    }

    pub fn redo(&mut self) {
        if let Some(snap) = self.undo_history.redo(&self.project.tasks, &self.project.dependencies) {
            self.project.tasks = snap.tasks;
            self.project.dependencies = snap.dependencies;
            self.project.recalculate_parent_dates();
            self.project.sort_tasks_grouped();
            self.project.touch();
            self.selected_task = None;
            self.status_message = "Redo".to_string();
        }
    }

    pub fn create_task_from_dialog(&mut self) {
        let name = if self.new_task_name.is_empty() {
            "New Task".to_string()
        } else {
            self.new_task_name.clone()
        };

        let start = self.new_task_start_date;
        let end = if self.new_task_end_date >= start {
            self.new_task_end_date
        } else {
            start + chrono::Duration::days(7)
        };

        let task = if self.new_task_is_milestone {
            Task::new_milestone(name, start)
        } else {
            let palette = ui::theme::task_palette();
            let color_idx = self.project.tasks.len() % palette.len().max(1);
            let mut t = Task::new(name, start, end);
            t.color = ui::theme::task_color(color_idx);
            t
        };

        self.undo_history.push(&self.project.tasks, &self.project.dependencies);
        self.project.tasks.push(task);
        self.project.sort_tasks_grouped();
        self.reset_dialog_fields();
        self.status_message = "Task added".to_string();
    }

    /// Add a subtask under the given parent. Inserts immediately after the parent's last child.
    pub fn add_subtask(&mut self, parent_id: Uuid) {
        let parent = match self.project.tasks.iter().find(|t| t.id == parent_id) {
            Some(p) => p.clone(),
            None => return,
        };
        let today = chrono::Local::now().date_naive();
        let start = parent.start.max(today);
        let end = parent.end.max(start + chrono::Duration::days(7));
        let palette = ui::theme::task_palette();
        let color_idx = self.project.tasks.len() % palette.len().max(1);
        let mut t = Task::new("New Subtask", start, end);
        t.color = ui::theme::task_color(color_idx);
        t.parent_id = Some(parent_id);

        self.undo_history.push(&self.project.tasks, &self.project.dependencies);

        // Insert after last child of parent (or right after parent if no children)
        let insert_pos = self.project.tasks.iter().rposition(|t| {
            t.parent_id == Some(parent_id) || t.id == parent_id
        }).map(|p| p + 1).unwrap_or(self.project.tasks.len());

        self.project.tasks.insert(insert_pos, t.clone());
        self.selected_task = Some(t.id);
        self.project.recalculate_parent_dates();
        self.project.touch();
        self.status_message = format!("Added subtask under '{}'", parent.name);
    }

    pub fn delete_task(&mut self, id: Uuid) {
        self.undo_history.push(&self.project.tasks, &self.project.dependencies);
        // Also delete all children of this task
        let children_ids: Vec<Uuid> = self
            .project
            .tasks
            .iter()
            .find(|t| t.id == id)
            .map(|parent| {
                parent
                    .children_ids(&self.project.tasks)
                    .into_iter()
                    .map(|child| child.id)
                    .collect()
            })
            .unwrap_or_default();
        self.project.tasks.retain(|t| t.id != id && t.parent_id != Some(id));
        self.project.dependencies.retain(|d| {
            d.from_task != id && d.to_task != id
            && !children_ids.contains(&d.from_task)
            && !children_ids.contains(&d.to_task)
        });
        self.project.recalculate_parent_dates();
        if self.selected_task == Some(id) || children_ids.contains(&self.selected_task.unwrap_or(Uuid::nil())) {
            self.selected_task = None;
        }
        self.status_message = "Task deleted".to_string();
    }

    fn reset_dialog_fields(&mut self) {
        let today = chrono::Local::now().date_naive();
        self.new_task_name = String::new();
        self.new_task_start = today.format("%Y-%m-%d").to_string();
        self.new_task_end = (today + chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        self.new_task_start_date = today;
        self.new_task_end_date = today + chrono::Duration::days(7);
        self.new_task_is_milestone = false;
    }

    fn recalculate_viewport(&mut self) {
        if let (Some(min), Some(max)) = (
            self.project.tasks.iter().map(|t| t.start).min(),
            self.project.tasks.iter().map(|t| t.end).max(),
        ) {
            self.viewport.start = min - chrono::Duration::days(7);
            self.viewport.end = max + chrono::Duration::days(30);
        }
    }
}

impl eframe::App for GanttApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::theme::set_active(self.theme_manager.active());
        ui::theme::apply_theme(ctx);

        // Handle keyboard shortcuts outside closures to avoid borrow issues
        let should_save = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S));
        let should_undo = ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Z));
        let should_redo = ctx.input(|i| i.modifiers.ctrl && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z))));
        if should_save {
            self.save_project();
        }
        if should_undo {
            self.undo();
        }
        if should_redo {
            self.redo();
        }

        // Handle pending subtask additions (from editor inside panel closure)
        if let Some(parent_id) = self.pending_add_subtask.take() {
            self.add_subtask(parent_id);
        }
        if let Some(dep) = self.pending_add_dependency.take() {
            let exists = self.project.dependencies.iter().any(|d| {
                d.from_task == dep.from_task && d.to_task == dep.to_task
            });
            if !exists {
                let from_name = self.project.tasks.iter().find(|t| t.id == dep.from_task).map(|t| t.name.clone()).unwrap_or_default();
                let to_name   = self.project.tasks.iter().find(|t| t.id == dep.to_task  ).map(|t| t.name.clone()).unwrap_or_default();
                self.undo_history.push(&self.project.tasks, &self.project.dependencies);
                self.project.dependencies.push(dep);
                self.project.touch();
                self.status_message = format!("Linked '{}' → '{}'", from_name, to_name);
            }
        }

        // Top panel: toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui::toolbar::show_toolbar(self, ui);
        });

        // Bottom panel: status bar
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(ui::theme::status_bar_height())
            .frame(
                egui::Frame::default()
                    .fill(ui::theme::status_bar_bg())
                    .inner_margin(egui::Margin::symmetric(10.0, 0.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .font(ui::theme::font_status())
                            .color(ui::theme::text_secondary()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("Tasks: {}", self.project.tasks.len()))
                                .size(10.5)
                                .color(ui::theme::text_dim()),
                        );
                        ui.label(
                            egui::RichText::new(" · ")
                                .size(10.5)
                                .color(ui::theme::text_dim()),
                        );
                        let default_ppd = ui::theme::zoom().default_pixels_per_day;
                        ui.label(
                            egui::RichText::new(format!(
                                "Zoom: {:.0}%",
                                self.viewport.pixels_per_day / default_ppd * 100.0
                            ))
                            .size(10.5)
                            .color(ui::theme::text_dim()),
                        );
                    });
                });
            });

        // Left panel: task table + editor
        let mut task_action = ui::task_table::TaskTableAction::None;
        let mut editor_changed = false;
        let mut dep_remove: Option<(Uuid, Uuid)> = None;
        egui::SidePanel::left("task_panel")
            .default_width(ui::theme::side_panel_default_width())
            .min_width(ui::theme::side_panel_min_width())
            .max_width(ui::theme::side_panel_default_width() * 2.0)
            .resizable(true)
            .frame(
                egui::Frame::default()
                    .fill(ui::theme::bg_panel())
                    .inner_margin(egui::Margin::same(ui::theme::layout().panel_inner_margin))
                    .stroke(egui::Stroke::new(1.0, ui::theme::border_subtle())),
            )
            .show(ctx, |ui| {
                // If a task is selected, show editor at the top
                if let Some(sel_id) = self.selected_task {
                    let deps_snapshot: Vec<_> = self.project.dependencies.clone();
                    let tasks_snapshot: Vec<_> = self.project.tasks.clone();
                    if let Some(task) = self.project.tasks.iter_mut().find(|t| t.id == sel_id) {
                        let result = ui::task_editor::show_task_editor(
                            task,
                            &tasks_snapshot,
                            &deps_snapshot,
                            ui,
                        );
                        match result {
                            ui::task_editor::EditorAction::Changed => {
                                editor_changed = true;
                            }
                            ui::task_editor::EditorAction::RemoveDependency(from, to) => {
                                dep_remove = Some((from, to));
                            }
                            ui::task_editor::EditorAction::AddSubtask(parent_id) => {
                                self.pending_add_subtask = Some(parent_id);
                            }
                            ui::task_editor::EditorAction::AddDependency(dep) => {
                                self.pending_add_dependency = Some(dep);
                            }
                            ui::task_editor::EditorAction::None => {}
                        }
                    }
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(2.0);
                }

                // Filter bar
                ui::filter_bar::show_filter_bar(
                    &mut self.search_query,
                    &mut self.filter_priority,
                    ui,
                );
                ui.add_space(2.0);

                task_action = ui::task_table::show_task_table(
                    &self.project.tasks,
                    self.selected_task,
                    &self.search_query,
                    self.filter_priority,
                    ui,
                );
            });

        // Handle task table actions
        match task_action {
            ui::task_table::TaskTableAction::Select(id) => {
                self.selected_task = Some(id);
            }
            ui::task_table::TaskTableAction::Delete(id) => {
                self.delete_task(id);
            }
            ui::task_table::TaskTableAction::Add => {
                self.show_add_task = true;
            }
            ui::task_table::TaskTableAction::ToggleCollapse(id) => {
                if let Some(task) = self.project.tasks.iter_mut().find(|t| t.id == id) {
                    task.collapsed = !task.collapsed;
                    self.project.touch();
                }
            }
            ui::task_table::TaskTableAction::None => {}
        }

        // If the editor modified the task, mark project dirty
        if editor_changed {
            self.project.recalculate_parent_dates();
            self.project.touch();
            self.status_message = "Task updated".to_string();
        }
        // Handle dependency removal from editor
        if let Some((from, to)) = dep_remove {
            self.undo_history.push(&self.project.tasks, &self.project.dependencies);
            self.project.dependencies.retain(|d| {
                !(d.from_task == from && d.to_task == to)
            });
            self.project.touch();
            self.status_message = "Dependency removed".to_string();
        }

        // Central panel: Gantt chart
        let chart_frame = egui::Frame::default()
            .fill(ui::theme::bg_dark())
            .inner_margin(egui::Margin::ZERO);
        egui::CentralPanel::default().frame(chart_frame).show(ctx, |ui| {
            let chart_interaction = ui::gantt_chart::show_gantt_chart(
                &mut self.project.tasks,
                &self.project.dependencies,
                &mut self.viewport,
                &mut self.selected_task,
                ui,
            );
            if chart_interaction.changed {
                self.project.recalculate_parent_dates();
                self.project.touch();
                if let Some(selected) = self.selected_task {
                    if let Some(task) = self.project.tasks.iter().find(|t| t.id == selected) {
                        self.status_message = format!(
                            "Updated '{}' ({} → {})",
                            task.name,
                            task.start.format("%Y-%m-%d"),
                            task.end.format("%Y-%m-%d")
                        );
                    } else {
                        self.status_message = "Timeline updated".to_string();
                    }
                } else {
                    self.status_message = "Timeline updated".to_string();
                }
            }
            if let Some(dep) = chart_interaction.new_dependency {
                // Avoid duplicates
                let exists = self.project.dependencies.iter().any(|d| {
                    d.from_task == dep.from_task && d.to_task == dep.to_task
                });
                if !exists {
                    let from_name = self.project.tasks.iter()
                        .find(|t| t.id == dep.from_task)
                        .map(|t| t.name.clone())
                        .unwrap_or_default();
                    let to_name = self.project.tasks.iter()
                        .find(|t| t.id == dep.to_task)
                        .map(|t| t.name.clone())
                        .unwrap_or_default();
                    self.undo_history.push(&self.project.tasks, &self.project.dependencies);
                    self.project.dependencies.push(dep);
                    self.project.touch();
                    self.status_message = format!("Linked '{}' → '{}'", from_name, to_name);
                }
            }
            if let Some((from, to)) = chart_interaction.remove_dependency {
                self.undo_history.push(&self.project.tasks, &self.project.dependencies);
                self.project.dependencies.retain(|d| {
                    !(d.from_task == from && d.to_task == to)
                });
                self.project.touch();
                self.status_message = "Dependency removed".to_string();
            }
            if let Some(parent_id) = chart_interaction.toggle_collapse {
                if let Some(task) = self.project.tasks.iter_mut().find(|t| t.id == parent_id) {
                    task.collapsed = !task.collapsed;
                    self.project.touch();
                }
            }
            if let Some(parent_id) = chart_interaction.add_subtask {
                self.add_subtask(parent_id);
            }
            if let Some(task_id) = chart_interaction.delete_task {
                self.delete_task(task_id);
            }
        });

        // Dialogs
        if self.show_add_task {
            ui::dialogs::show_add_task_dialog(self, ctx);
        }
        if self.show_about {
            ui::dialogs::show_about_dialog(self, ctx);
        }
        if self.show_csv_help {
            ui::dialogs::show_csv_help_dialog(self, ctx);
        }
    }
}
