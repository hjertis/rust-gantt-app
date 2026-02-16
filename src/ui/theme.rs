#![allow(dead_code)]
//! Theme accessor facade.
//!
//! The rest of the codebase calls functions like `theme::bg_dark()` instead of
//! reading `const` values.  Under the hood every accessor reads from the
//! [`ThemeDefinition`] that was installed for the current frame via
//! [`set_active`].
//!
//! This file replaces the original `theme.rs` that had `pub const` values.

use crate::ui::theme_def::ThemeDefinition;
use egui::{Color32, FontId, Rounding, Stroke, Visuals};
use std::cell::RefCell;

// ─── Thread-local active theme ──────────────────────────────────────────────

thread_local! {
    static ACTIVE: RefCell<ThemeDefinition> = RefCell::new(ThemeDefinition::default());
}

/// Install a theme definition for the current frame.
/// Call this once at the top of `update()` before any UI code runs.
pub fn set_active(def: &ThemeDefinition) {
    ACTIVE.with(|cell| {
        *cell.borrow_mut() = def.clone();
    });
}

/// Read the full definition (rarely needed; prefer the named accessors below).
pub fn with_active<R>(f: impl FnOnce(&ThemeDefinition) -> R) -> R {
    ACTIVE.with(|cell| f(&cell.borrow()))
}

// ─── Colour accessors ──────────────────────────────────────────────────────

macro_rules! color_accessor {
    ($name:ident, $field:ident) => {
        pub fn $name() -> Color32 {
            ACTIVE.with(|cell| cell.borrow().colors.$field)
        }
    };
}

color_accessor!(bg_dark, bg_dark);
color_accessor!(bg_panel, bg_panel);
color_accessor!(bg_header, bg_header);
color_accessor!(bg_row_even, bg_row_even);
color_accessor!(bg_selected, bg_selected);
color_accessor!(bg_field, bg_field);

color_accessor!(border_subtle, border_subtle);
color_accessor!(border_accent, border_accent);

color_accessor!(text_primary, text_primary);
color_accessor!(text_secondary, text_secondary);
color_accessor!(text_dim, text_dim);
color_accessor!(text_on_bar, text_on_bar);

color_accessor!(accent, accent);
color_accessor!(today_line, today_line);
color_accessor!(grid_line, grid_line);
color_accessor!(handle_color, handle_color);
color_accessor!(weekend_shade, weekend_shade);
color_accessor!(weekend_header_shade, weekend_header_shade);
color_accessor!(progress_overlay, progress_overlay);

color_accessor!(dep_arrow, dep_arrow);
color_accessor!(dep_arrow_hover, dep_arrow_hover);
color_accessor!(dep_creating, dep_creating);

color_accessor!(widget_bg_inactive, widget_bg_inactive);
color_accessor!(widget_bg_hovered, widget_bg_hovered);
color_accessor!(widget_bg_active, widget_bg_active);
color_accessor!(widget_bg_open, widget_bg_open);
color_accessor!(faint_bg, faint_bg);
color_accessor!(extreme_bg, extreme_bg);

color_accessor!(status_bar_bg, status_bar_bg);

color_accessor!(row_selected_stroke, row_selected_stroke);
color_accessor!(row_unselected_stroke, row_unselected_stroke);

// ─── Task palette ──────────────────────────────────────────────────────────

pub fn task_palette() -> Vec<Color32> {
    ACTIVE.with(|cell| cell.borrow().colors.task_palette.clone())
}

/// Convenience: get palette colour by wrapping index.
pub fn task_color(index: usize) -> Color32 {
    ACTIVE.with(|cell| {
        let p = &cell.borrow().colors.task_palette;
        if p.is_empty() {
            Color32::from_rgb(70, 130, 180)
        } else {
            p[index % p.len()]
        }
    })
}

// ─── Typography accessors ──────────────────────────────────────────────────

pub fn font_header() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_header_size))
}

pub fn font_sub() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_sub_size))
}

pub fn font_bar() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_bar_size))
}

pub fn font_small() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_small_size))
}

pub fn font_body() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_body_size))
}

pub fn font_menu() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_menu_size))
}

pub fn font_label() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_label_size))
}

pub fn font_status() -> FontId {
    FontId::proportional(ACTIVE.with(|c| c.borrow().typography.font_status_size))
}

// ─── Sizing accessors ──────────────────────────────────────────────────────

macro_rules! sizing_accessor {
    ($name:ident, $field:ident) => {
        pub fn $name() -> f32 {
            ACTIVE.with(|cell| cell.borrow().sizing.$field)
        }
    };
}

sizing_accessor!(row_height, row_height);
sizing_accessor!(row_gap, row_gap);
sizing_accessor!(header_height, header_height);
sizing_accessor!(handle_width, handle_width);
sizing_accessor!(bar_rounding, bar_rounding);
sizing_accessor!(bar_inset, bar_inset);
sizing_accessor!(status_bar_height, status_bar_height);
sizing_accessor!(side_panel_default_width, side_panel_default_width);
sizing_accessor!(side_panel_min_width, side_panel_min_width);

// ─── Spacing accessor ──────────────────────────────────────────────────────

pub fn widget_rounding_val() -> f32 {
    ACTIVE.with(|c| c.borrow().spacing.widget_rounding)
}

pub fn window_rounding_val() -> f32 {
    ACTIVE.with(|c| c.borrow().spacing.window_rounding)
}

// ─── Rendering accessors ───────────────────────────────────────────────────

pub fn rendering() -> crate::ui::theme_def::ThemeRendering {
    ACTIVE.with(|c| c.borrow().rendering.clone())
}

// ─── Layout accessors ──────────────────────────────────────────────────────

pub fn layout() -> crate::ui::theme_def::ThemeLayout {
    ACTIVE.with(|c| c.borrow().layout.clone())
}

// ─── Motion accessors ──────────────────────────────────────────────────────

pub fn reorder_anim_duration() -> f32 {
    ACTIVE.with(|c| c.borrow().motion.reorder_anim_duration)
}

// ─── Zoom accessors ────────────────────────────────────────────────────────

pub fn zoom() -> crate::ui::theme_def::ThemeZoom {
    ACTIVE.with(|c| c.borrow().zoom.clone())
}

pub fn is_light() -> bool {
    ACTIVE.with(|c| c.borrow().meta.variant == "light")
}

// ─── Apply to egui Context ─────────────────────────────────────────────────

/// Applies the currently installed theme to the egui context.
/// Must be called once per frame after [`set_active`].
pub fn apply_theme(ctx: &egui::Context) {
    let def = ACTIVE.with(|cell| cell.borrow().clone());
    let c = &def.colors;
    let s = &def.spacing;

    let mut visuals = if def.meta.variant == "light" {
        Visuals::light()
    } else {
        Visuals::dark()
    };

    visuals.override_text_color = Some(c.text_primary);
    visuals.panel_fill = c.bg_panel;
    visuals.window_fill = c.bg_panel;
    visuals.extreme_bg_color = c.extreme_bg;
    visuals.faint_bg_color = c.bg_row_even;

    let wr = Rounding::same(s.widget_rounding);

    visuals.widgets.noninteractive.bg_fill = c.bg_panel;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, c.border_subtle);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, c.text_secondary);
    visuals.widgets.noninteractive.rounding = wr;

    visuals.widgets.inactive.bg_fill = c.widget_bg_inactive;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, c.border_subtle);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, c.text_primary);
    visuals.widgets.inactive.rounding = wr;

    visuals.widgets.hovered.bg_fill = c.widget_bg_hovered;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, c.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, c.text_primary);
    visuals.widgets.hovered.rounding = wr;

    visuals.widgets.active.bg_fill = c.widget_bg_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, c.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    visuals.widgets.active.rounding = wr;

    visuals.widgets.open.bg_fill = c.widget_bg_open;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, c.accent);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, c.text_primary);
    visuals.widgets.open.rounding = wr;

    visuals.selection.bg_fill = c.bg_selected;
    visuals.selection.stroke = Stroke::new(1.0, c.accent);

    visuals.window_rounding = Rounding::same(s.window_rounding);
    visuals.window_stroke = Stroke::new(1.0, c.border_subtle);

    visuals.striped = false;
    visuals.faint_bg_color = c.faint_bg;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(s.item_spacing_x, s.item_spacing_y);
    style.spacing.button_padding = egui::vec2(s.button_padding_x, s.button_padding_y);
    ctx.set_style(style);
}
