use crate::model::Task;
use crate::model::task::{Dependency, DependencyKind, TaskPriority};
use crate::ui::theme;
use egui::{Color32, Id, RichText, Ui};
use uuid::Uuid;

/// Actions the editor can request.
pub enum EditorAction {
    None,
    Changed,
    RemoveDependency(Uuid, Uuid),
    AddSubtask(Uuid),
    AddDependency(Dependency),
}

/// Persistent state for the "add dependency" picker.
#[derive(Clone)]
struct DepPickerState {
    target_id: Option<Uuid>,
    kind: DependencyKind,
}

/// Short label for a dependency from this task's perspective.
fn dep_kind_label(kind: DependencyKind, is_outgoing: bool) -> String {
    let arrow = if is_outgoing {
        egui_phosphor::regular::ARROW_RIGHT
    } else {
        egui_phosphor::regular::ARROW_LEFT
    };
    format!("[{}] {}", kind.short_label(), arrow)
}

/// Render an inline task editor for the selected task.
/// Also shows dependencies involving this task.
pub fn show_task_editor(
    task: &mut Task,
    all_tasks: &[Task],
    dependencies: &[Dependency],
    ui: &mut Ui,
) -> EditorAction {
    let mut action = EditorAction::None;
    let task_id = task.id;

    // Section header
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Edit Task")
                .strong()
                .size(13.0)
                .color(theme::text_primary()),
        );
    });
    ui.add_space(4.0);

    let frame = egui::Frame {
        fill: theme::bg_dark(),
        rounding: egui::Rounding::same(theme::widget_rounding_val()),
        inner_margin: egui::Margin::same(theme::layout().editor_inner_margin),
        outer_margin: egui::Margin::ZERO,
        stroke: egui::Stroke::new(1.0, theme::border_subtle()),
        shadow: egui::epaint::Shadow::NONE,
    };

    frame.show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        // Force dark text-field backgrounds
        ui.visuals_mut().extreme_bg_color = theme::bg_field();

        // ── Task Name ──────────────────────────────────────────────────
        ui.label(
            RichText::new("Name")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        let name_edit = ui.add_sized(
            [ui.available_width(), 24.0],
            egui::TextEdit::singleline(&mut task.name)
                .font(egui::FontId::proportional(12.0))
                .text_color(theme::text_primary()),
        );
        if name_edit.changed() {
            action = EditorAction::Changed;
        }

        ui.add_space(2.0);

        // ── Priority ──────────────────────────────────────────────────
        ui.label(
            RichText::new("Priority")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        let pri_label = format!("{} {}", task.priority.icon(), task.priority.label());
        egui::ComboBox::from_id_salt("priority_combo")
            .selected_text(RichText::new(&pri_label).size(11.0))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for p in TaskPriority::all() {
                    let lbl = format!("{} {}", p.icon(), p.label());
                    if ui.selectable_value(&mut task.priority, *p, lbl).changed() {
                        action = EditorAction::Changed;
                    }
                }
            });

        ui.add_space(2.0);

        // ── Parent Task (Phase/Group) ────────────────────────────────
        ui.label(
            RichText::new("Phase / Parent")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        let parent_label = task
            .parent_id
            .and_then(|pid| all_tasks.iter().find(|t| t.id == pid))
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "— None —".to_string());

        // Collect valid parent candidates:
        // - not self, not own children, and not already a child (one-level only)
        let candidates: Vec<(Uuid, String)> = all_tasks
            .iter()
            .filter(|t| {
                t.id != task_id
                    && t.parent_id != Some(task_id)  // not own child
                    && t.parent_id.is_none()          // only top-level tasks can be parents
            })
            .map(|t| (t.id, t.name.clone()))
            .collect();

        egui::ComboBox::from_id_salt("parent_combo")
            .selected_text(RichText::new(&parent_label).size(11.0))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(task.parent_id.is_none(), "— None —")
                    .clicked()
                {
                    task.parent_id = None;
                    action = EditorAction::Changed;
                }
                for (cid, cname) in &candidates {
                    if ui
                        .selectable_label(task.parent_id == Some(*cid), cname.as_str())
                        .clicked()
                    {
                        task.parent_id = Some(*cid);
                        action = EditorAction::Changed;
                    }
                }
            });

        ui.add_space(2.0);

        // ── Dates ───────────────────────────────────────────────────
        // For parent tasks, dates are auto-calculated from children (read-only).
        let is_parent_task = task.has_children(all_tasks);
        if is_parent_task {
            ui.label(RichText::new("Dates").size(10.0).color(theme::text_dim()).strong());
            ui.horizontal(|ui| {
                ui.label(RichText::new(task.start.format("%Y-%m-%d").to_string()).size(11.0).color(theme::text_secondary()));
                ui.label(RichText::new(egui_phosphor::regular::ARROW_RIGHT).size(10.0).color(theme::text_dim()));
                ui.label(RichText::new(task.end.format("%Y-%m-%d").to_string()).size(11.0).color(theme::text_secondary()));
                ui.label(RichText::new("(auto)").size(9.0).color(theme::text_dim()));
            });
            ui.add_space(2.0);
            // Progress: read-only for parent
            ui.label(RichText::new("Progress").size(10.0).color(theme::text_dim()).strong());
            ui.label(RichText::new(format!("{:.0}%  (auto-calculated)", task.progress * 100.0)).size(11.0).color(theme::text_secondary()));
            ui.add_space(4.0);
            // Add subtask button
            let btn = egui::Button::new(RichText::new(egui_phosphor::regular::PLUS.to_string() + "  Add Subtask").color(Color32::WHITE).size(12.0))
                .fill(theme::accent())
                .rounding(egui::Rounding::same(4.0));
            if ui.add_sized([ui.available_width(), 26.0], btn).clicked() {
                action = EditorAction::AddSubtask(task_id);
            }
            ui.add_space(2.0);
        } else if !task.is_milestone {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Start")
                            .size(10.0)
                            .color(theme::text_dim())
                            .strong(),
                    );
                    let resp = ui.add(
                        egui_extras::DatePickerButton::new(&mut task.start)
                            .id_salt("dp_start"),
                    );
                    if resp.changed() {
                        if task.start > task.end {
                            task.end = task.start;
                        }
                        action = EditorAction::Changed;
                    }
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("End")
                            .size(10.0)
                            .color(theme::text_dim())
                            .strong(),
                    );
                    let resp = ui.add(
                        egui_extras::DatePickerButton::new(&mut task.end)
                            .id_salt("dp_end"),
                    );
                    if resp.changed() {
                        if task.end < task.start {
                            task.start = task.end;
                        }
                        action = EditorAction::Changed;
                    }
                });
            });
        } else {
            // Milestone: single date
            ui.label(
                RichText::new("Date")
                    .size(10.0)
                    .color(theme::text_dim())
                    .strong(),
            );
            let resp = ui.add(
                egui_extras::DatePickerButton::new(&mut task.start)
                    .id_salt("dp_milestone"),
            );
            if resp.changed() {
                task.end = task.start;
                action = EditorAction::Changed;
            }
        }

        ui.add_space(2.0);

        // ── Progress ──────────────────────────────────────────────────
        // Only show editable slider for non-parent tasks (parents auto-calculate from children)
        if !is_parent_task {
            ui.label(
                RichText::new("Progress")
                    .size(10.0)
                    .color(theme::text_dim())
                    .strong(),
            );
            ui.horizontal(|ui| {
                let slider = egui::Slider::new(&mut task.progress, 0.0..=1.0)
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                    .custom_parser(|s| {
                        let s = s.trim().trim_end_matches('%');
                        s.parse::<f64>().ok().map(|v| v / 100.0)
                    });
                let resp = ui.add_sized([ui.available_width(), 20.0], slider);
                if resp.changed() {
                    action = EditorAction::Changed;
                }
            });
        }

        ui.add_space(2.0);

        // ── Notes / Description ───────────────────────────────────────
        ui.label(
            RichText::new("Notes")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        let notes_resp = ui.add_sized(
            [ui.available_width(), 60.0],
            egui::TextEdit::multiline(&mut task.description)
                .font(egui::FontId::proportional(11.0))
                .text_color(theme::text_secondary())
                .hint_text("Add notes or description..."),
        );
        if notes_resp.changed() {
            action = EditorAction::Changed;
        }

        ui.add_space(2.0);

        // ── Color ─────────────────────────────────────────────────────
        ui.label(
            RichText::new("Color")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            let palette = theme::task_palette();
            for color in &palette {
                let is_current = task.color == *color;
                let size = if is_current { 20.0 } else { 16.0 };
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                let rounding = egui::Rounding::same(3.0);
                ui.painter().rect_filled(rect, rounding, *color);

                if is_current {
                    ui.painter().rect_stroke(
                        rect.expand(1.0),
                        egui::Rounding::same(4.0),
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }

                if resp.on_hover_text("Click to set color").clicked() {
                    task.color = *color;
                    action = EditorAction::Changed;
                }
            }
        });

        ui.add_space(2.0);

        // ── Milestone toggle ──────────────────────────────────────────
        ui.horizontal(|ui| {
            let mut is_milestone = task.is_milestone;
            let resp = ui.checkbox(&mut is_milestone, "");
            ui.label(
                RichText::new("Milestone")
                    .size(11.0)
                    .color(theme::text_secondary()),
            );
            if resp.changed() {
                task.is_milestone = is_milestone;
                if is_milestone {
                    task.end = task.start;
                }
                action = EditorAction::Changed;
            }
        });

        ui.add_space(4.0);

        // ── Dependencies ─────────────────────────────────────────────
        let task_deps: Vec<&Dependency> = dependencies
            .iter()
            .filter(|d| d.from_task == task_id || d.to_task == task_id)
            .collect();

        ui.separator();
        ui.add_space(2.0);
        ui.label(
            RichText::new("Dependencies")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        ui.add_space(2.0);

        if task_deps.is_empty() {
            ui.label(
                RichText::new("No dependencies yet")
                    .size(9.5)
                    .color(theme::text_dim()),
            );
        } else {
            for dep in &task_deps {
                let is_outgoing = dep.from_task == task_id;
                let other_id = if is_outgoing { dep.to_task } else { dep.from_task };
                let other_name = all_tasks
                    .iter()
                    .find(|t| t.id == other_id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "?".to_string());

                let kind_lbl = dep_kind_label(dep.kind, is_outgoing);
                let label = format!("{} {}", kind_lbl, other_name);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&label)
                            .size(11.0)
                            .color(theme::text_secondary()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del = ui.add(
                            egui::Button::new(
                                RichText::new(egui_phosphor::regular::X).size(9.0).color(theme::text_dim()),
                            )
                            .frame(false),
                        );
                        if del.on_hover_text("Remove dependency").clicked() {
                            action = EditorAction::RemoveDependency(dep.from_task, dep.to_task);
                        }
                    });
                });
            }
        }

        ui.add_space(4.0);

        // ── Add dependency picker ─────────────────────────────────────
        // Candidates: if this task is a child, only siblings (same parent);
        // otherwise all tasks except self, parent tasks (summary bars), and
        // tasks already linked to this one.
        let already_linked: std::collections::HashSet<Uuid> = dependencies
            .iter()
            .filter(|d| d.from_task == task_id || d.to_task == task_id)
            .map(|d| if d.from_task == task_id { d.to_task } else { d.from_task })
            .collect();

        let candidates: Vec<(Uuid, String)> = all_tasks
            .iter()
            .filter(|t| {
                if t.id == task_id { return false; }
                if already_linked.contains(&t.id) { return false; }
                // Don't link to/from summary (parent) tasks
                let t_is_parent = t.has_children(all_tasks);
                if t_is_parent { return false; }
                // If this task is a child, restrict to siblings (same parent)
                if let Some(my_parent) = task.parent_id {
                    return t.parent_id == Some(my_parent);
                }
                // Top-level task: only link to other top-level tasks
                t.parent_id.is_none()
            })
            .map(|t| (t.id, t.name.clone()))
            .collect();

        if !candidates.is_empty() {
            let picker_id = Id::new(("dep-picker", task_id));
            let mut state: DepPickerState = ui.ctx().data_mut(|d| {
                d.get_temp(picker_id).unwrap_or(DepPickerState {
                    target_id: None,
                    kind: DependencyKind::FinishToStart,
                })
            });

            let target_label = state.target_id
                .and_then(|id| candidates.iter().find(|(cid, _)| *cid == id))
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| "— pick task —".to_string());

            ui.label(
                RichText::new("Add dependency")
                    .size(10.0)
                    .color(theme::text_dim())
                    .strong(),
            );

            // Row: [kind combo] [target combo] [+ button]
            ui.horizontal(|ui| {
                // Kind selector (narrow)
                let kind_resp = egui::ComboBox::from_id_salt("new-dep-kind")
                    .selected_text(RichText::new(state.kind.short_label()).size(11.0))
                    .width(46.0)
                    .show_ui(ui, |ui| {
                        for k in &[
                            DependencyKind::FinishToStart,
                            DependencyKind::StartToStart,
                            DependencyKind::FinishToFinish,
                            DependencyKind::StartToFinish,
                        ] {
                            let row = ui.selectable_label(state.kind == *k, k.short_label());
                            if row.on_hover_text(k.description()).clicked() {
                                state.kind = *k;
                            }
                        }
                    });
                kind_resp.response.on_hover_text(state.kind.description());

                // Target task selector — use a fixed width to avoid stretching the panel
                let combo_w = (ui.available_width() - 30.0).clamp(60.0, 220.0);
                egui::ComboBox::from_id_salt("new-dep-target")
                    .selected_text(RichText::new(&target_label).size(11.0))
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for (cid, cname) in &candidates {
                            if ui.selectable_label(state.target_id == Some(*cid), cname.as_str()).clicked() {
                                state.target_id = Some(*cid);
                            }
                        }
                    });

                // Add button
                let can_add = state.target_id.is_some();
                let btn = egui::Button::new(RichText::new("＋").size(13.0).color(Color32::WHITE))
                    .fill(if can_add { theme::accent() } else { theme::bg_field() })
                    .rounding(egui::Rounding::same(4.0));
                let add_resp = ui.add_enabled(can_add, btn);
                if add_resp.clicked() {
                    if let Some(target) = state.target_id {
                        action = EditorAction::AddDependency(Dependency {
                            from_task: task_id,
                            to_task: target,
                            kind: state.kind,
                        });
                        state.target_id = None; // reset after adding
                    }
                }
            });

            ui.ctx().data_mut(|d| d.insert_temp(picker_id, state));

            ui.add_space(2.0);
            ui.label(
                RichText::new("Or Shift+drag between bars")
                    .size(9.0)
                    .color(theme::text_dim()),
            );
        } else {
            ui.label(
                RichText::new("Shift+drag between bars to link tasks")
                    .size(9.5)
                    .color(theme::text_dim()),
            );
        }
    });

    action
}
