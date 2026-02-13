use egui::{Color32, FontId, Rounding, Stroke, Visuals};

// ── Palette ──────────────────────────────────────────────────────────────────

pub const BG_DARK: Color32 = Color32::from_rgb(24, 24, 32);
pub const BG_PANEL: Color32 = Color32::from_rgb(30, 30, 40);
pub const BG_HEADER: Color32 = Color32::from_rgb(34, 37, 48);
pub const BG_ROW_EVEN: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 6);
pub const BG_ROW_HOVER: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 14);
pub const BG_SELECTED: Color32 = Color32::from_rgba_premultiplied(80, 140, 220, 45);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(50, 52, 64);
pub const BORDER_ACCENT: Color32 = Color32::from_rgb(90, 140, 220);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 232, 240);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(155, 160, 178);
pub const TEXT_DIM: Color32 = Color32::from_rgb(100, 105, 120);
pub const TEXT_ON_BAR: Color32 = Color32::from_rgb(255, 255, 255);

pub const ACCENT: Color32 = Color32::from_rgb(80, 140, 220);
pub const TODAY_LINE: Color32 = Color32::from_rgb(240, 75, 75);
pub const GRID_LINE: Color32 = Color32::from_rgb(44, 46, 58);
pub const HANDLE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);

pub const PROGRESS_OVERLAY: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 55);

// ── Sizes ────────────────────────────────────────────────────────────────────

pub const ROW_HEIGHT: f32 = 30.0;
pub const ROW_GAP: f32 = 2.0;
pub const HEADER_HEIGHT: f32 = 44.0;
pub const HANDLE_WIDTH: f32 = 7.0;
pub const BAR_ROUNDING: f32 = 5.0;
pub const BAR_INSET: f32 = 3.0; // vertical inset so bars don't touch row edges

// ── Fonts ────────────────────────────────────────────────────────────────────

pub fn font_header() -> FontId {
    FontId::proportional(12.0)
}

pub fn font_sub() -> FontId {
    FontId::proportional(10.5)
}

pub fn font_bar() -> FontId {
    FontId::proportional(11.5)
}

pub fn font_small() -> FontId {
    FontId::proportional(9.5)
}

// ── Task color palette ───────────────────────────────────────────────────────

pub const TASK_COLORS: &[Color32] = &[
    Color32::from_rgb(66, 133, 244),  // Google blue
    Color32::from_rgb(52, 168, 83),   // Green
    Color32::from_rgb(171, 71, 188),  // Purple
    Color32::from_rgb(251, 140, 0),   // Orange
    Color32::from_rgb(3, 169, 244),   // Light blue
    Color32::from_rgb(229, 57, 53),   // Red
    Color32::from_rgb(0, 188, 212),   // Cyan
    Color32::from_rgb(255, 193, 7),   // Amber
];

// ── Apply custom visuals ─────────────────────────────────────────────────────

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 28); // TextEdit bg
    visuals.faint_bg_color = BG_ROW_EVEN;

    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(42, 44, 56);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 54, 68);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);

    visuals.widgets.active.bg_fill = Color32::from_rgb(60, 62, 76);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(4.0);

    visuals.widgets.open.bg_fill = Color32::from_rgb(50, 52, 66);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.open.rounding = Rounding::same(4.0);

    visuals.selection.bg_fill = BG_SELECTED;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0, BORDER_SUBTLE);

    visuals.striped = false;
    visuals.faint_bg_color = Color32::from_rgb(30, 30, 40);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(style);
}
