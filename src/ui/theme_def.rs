//! Theme definition data model.
//!
//! Every visual parameter in the app is captured in [`ThemeDefinition`].
//! Themes are serialised as JSON with `#RRGGBB` / `#RRGGBBAA` colour strings
//! so that end-users can hand-edit them.
//!
//! All fields carry `#[serde(default)]` so that a partial JSON file is valid:
//! missing keys silently fall back to the built-in defaults.

use egui::Color32;
use serde::{Deserialize, Serialize};

// ─── Hex-colour serde helper ────────────────────────────────────────────────

pub mod hex_color {
    use egui::Color32;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let [r, g, b, a] = color.to_array();
        if a == 255 {
            serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", r, g, b))
        } else {
            serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_hex_color(&s).map_err(serde::de::Error::custom)
    }

    pub fn parse_hex_color(s: &str) -> Result<Color32, String> {
        let s = s.trim().trim_start_matches('#');
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
                Ok(Color32::from_rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
                let a = u8::from_str_radix(&s[6..8], 16).map_err(|e| e.to_string())?;
                Ok(Color32::from_rgba_unmultiplied(r, g, b, a))
            }
            _ => Err(format!("Invalid hex color '{}': expected 6 or 8 hex digits", s)),
        }
    }
}

/// Serde helper for `Vec<Color32>` stored as an array of hex strings.
mod hex_color_vec {
    use egui::Color32;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(colors: &Vec<Color32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(colors.len()))?;
        for c in colors {
            let [r, g, b, a] = c.to_array();
            let s = if a == 255 {
                format!("#{:02X}{:02X}{:02X}", r, g, b)
            } else {
                format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
            };
            seq.serialize_element(&s)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Color32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .iter()
            .map(|s| super::hex_color::parse_hex_color(s).map_err(serde::de::Error::custom))
            .collect()
    }
}

// ─── Top-level definition ───────────────────────────────────────────────────

/// Complete theme definition. Every visual knob in the app lives here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeDefinition {
    pub meta: ThemeMeta,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub sizing: ThemeSizing,
    pub rendering: ThemeRendering,
    pub layout: ThemeLayout,
    pub motion: ThemeMotion,
    pub zoom: ThemeZoom,
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        Self {
            meta: ThemeMeta::default(),
            colors: ThemeColors::default(),
            typography: ThemeTypography::default(),
            spacing: ThemeSpacing::default(),
            sizing: ThemeSizing::default(),
            rendering: ThemeRendering::default(),
            layout: ThemeLayout::default(),
            motion: ThemeMotion::default(),
            zoom: ThemeZoom::default(),
        }
    }
}

// ─── Meta ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeMeta {
    pub name: String,
    pub author: String,
    pub description: String,
    /// "dark" or "light" — controls whether egui starts from Visuals::dark() or light().
    pub variant: String,
}

impl Default for ThemeMeta {
    fn default() -> Self {
        Self {
            name: "Default Dark".into(),
            author: "Built-in".into(),
            description: "The default dark theme.".into(),
            variant: "dark".into(),
        }
    }
}

// ─── Colors ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeColors {
    // Backgrounds
    #[serde(with = "hex_color")]
    pub bg_dark: Color32,
    #[serde(with = "hex_color")]
    pub bg_panel: Color32,
    #[serde(with = "hex_color")]
    pub bg_header: Color32,
    #[serde(with = "hex_color")]
    pub bg_row_even: Color32,
    #[serde(with = "hex_color")]
    pub bg_selected: Color32,
    #[serde(with = "hex_color")]
    pub bg_field: Color32,

    // Borders
    #[serde(with = "hex_color")]
    pub border_subtle: Color32,
    #[serde(with = "hex_color")]
    pub border_accent: Color32,

    // Text
    #[serde(with = "hex_color")]
    pub text_primary: Color32,
    #[serde(with = "hex_color")]
    pub text_secondary: Color32,
    #[serde(with = "hex_color")]
    pub text_dim: Color32,
    #[serde(with = "hex_color")]
    pub text_on_bar: Color32,

    // Semantic
    #[serde(with = "hex_color")]
    pub accent: Color32,
    #[serde(with = "hex_color")]
    pub today_line: Color32,
    #[serde(with = "hex_color")]
    pub grid_line: Color32,
    #[serde(with = "hex_color")]
    pub handle_color: Color32,
    #[serde(with = "hex_color")]
    pub weekend_shade: Color32,
    #[serde(with = "hex_color")]
    pub weekend_header_shade: Color32,
    #[serde(with = "hex_color")]
    pub progress_overlay: Color32,

    // Dependencies
    #[serde(with = "hex_color")]
    pub dep_arrow: Color32,
    #[serde(with = "hex_color")]
    pub dep_arrow_hover: Color32,
    #[serde(with = "hex_color")]
    pub dep_creating: Color32,

    // Widget colors (egui Visuals overrides)
    #[serde(with = "hex_color")]
    pub widget_bg_inactive: Color32,
    #[serde(with = "hex_color")]
    pub widget_bg_hovered: Color32,
    #[serde(with = "hex_color")]
    pub widget_bg_active: Color32,
    #[serde(with = "hex_color")]
    pub widget_bg_open: Color32,
    #[serde(with = "hex_color")]
    pub faint_bg: Color32,
    #[serde(with = "hex_color")]
    pub extreme_bg: Color32,

    // Status bar
    #[serde(with = "hex_color")]
    pub status_bar_bg: Color32,

    // Row selection stroke
    #[serde(with = "hex_color")]
    pub row_selected_stroke: Color32,
    #[serde(with = "hex_color")]
    pub row_unselected_stroke: Color32,

    // Task palette (auto-assigned to new tasks)
    #[serde(with = "hex_color_vec")]
    pub task_palette: Vec<Color32>,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            bg_dark: Color32::from_rgb(24, 24, 32),
            bg_panel: Color32::from_rgb(27, 30, 39),
            bg_header: Color32::from_rgb(31, 35, 46),
            bg_row_even: Color32::from_rgba_unmultiplied(255, 255, 255, 6),
            bg_selected: Color32::from_rgba_unmultiplied(95, 145, 220, 34),
            bg_field: Color32::from_rgb(20, 20, 28),

            border_subtle: Color32::from_rgb(47, 51, 63),
            border_accent: Color32::from_rgb(90, 140, 220),

            text_primary: Color32::from_rgb(230, 232, 240),
            text_secondary: Color32::from_rgb(162, 168, 186),
            text_dim: Color32::from_rgb(111, 118, 136),
            text_on_bar: Color32::from_rgb(255, 255, 255),

            accent: Color32::from_rgb(80, 140, 220),
            today_line: Color32::from_rgb(240, 75, 75),
            grid_line: Color32::from_rgb(40, 44, 56),
            handle_color: Color32::from_rgb(255, 255, 255),
            weekend_shade: Color32::from_rgba_unmultiplied(8, 9, 11, 20),
            weekend_header_shade: Color32::from_rgba_unmultiplied(10, 12, 14, 26),
            progress_overlay: Color32::from_rgba_unmultiplied(0, 0, 0, 55),

            dep_arrow: Color32::from_rgba_unmultiplied(219, 174, 94, 110),
            dep_arrow_hover: Color32::from_rgba_unmultiplied(242, 202, 134, 180),
            dep_creating: Color32::from_rgba_unmultiplied(120, 200, 255, 180),

            widget_bg_inactive: Color32::from_rgb(38, 42, 54),
            widget_bg_hovered: Color32::from_rgb(48, 53, 67),
            widget_bg_active: Color32::from_rgb(57, 62, 78),
            widget_bg_open: Color32::from_rgb(46, 50, 64),
            faint_bg: Color32::from_rgb(30, 30, 40),
            extreme_bg: Color32::from_rgb(19, 21, 29),

            status_bar_bg: Color32::from_rgb(26, 26, 36),

            row_selected_stroke: Color32::from_rgba_unmultiplied(110, 165, 245, 140),
            row_unselected_stroke: Color32::from_rgba_unmultiplied(255, 255, 255, 10),

            task_palette: vec![
                Color32::from_rgb(66, 133, 244),  // Google blue
                Color32::from_rgb(52, 168, 83),   // Green
                Color32::from_rgb(171, 71, 188),  // Purple
                Color32::from_rgb(251, 140, 0),   // Orange
                Color32::from_rgb(3, 169, 244),   // Light blue
                Color32::from_rgb(229, 57, 53),   // Red
                Color32::from_rgb(0, 188, 212),   // Cyan
                Color32::from_rgb(255, 193, 7),   // Amber
            ],
        }
    }
}

// ─── Typography ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeTypography {
    pub font_header_size: f32,
    pub font_sub_size: f32,
    pub font_bar_size: f32,
    pub font_small_size: f32,
    pub font_body_size: f32,
    /// Menu / toolbar button text
    pub font_menu_size: f32,
    /// Editor field label size
    pub font_label_size: f32,
    /// Status bar text size
    pub font_status_size: f32,
}

impl Default for ThemeTypography {
    fn default() -> Self {
        Self {
            font_header_size: 12.0,
            font_sub_size: 10.5,
            font_bar_size: 11.5,
            font_small_size: 9.5,
            font_body_size: 12.0,
            font_menu_size: 12.0,
            font_label_size: 10.0,
            font_status_size: 11.0,
        }
    }
}

// ─── Spacing ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeSpacing {
    pub item_spacing_x: f32,
    pub item_spacing_y: f32,
    pub button_padding_x: f32,
    pub button_padding_y: f32,
    /// Rounding radius applied to most widgets
    pub widget_rounding: f32,
    /// Rounding for the application window
    pub window_rounding: f32,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            item_spacing_x: 8.0,
            item_spacing_y: 4.0,
            button_padding_x: 8.0,
            button_padding_y: 4.0,
            widget_rounding: 6.0,
            window_rounding: 8.0,
        }
    }
}

// ─── Sizing ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeSizing {
    pub row_height: f32,
    pub row_gap: f32,
    pub header_height: f32,
    pub handle_width: f32,
    pub bar_rounding: f32,
    pub bar_inset: f32,
    pub status_bar_height: f32,
    pub side_panel_default_width: f32,
    pub side_panel_min_width: f32,
}

impl Default for ThemeSizing {
    fn default() -> Self {
        Self {
            row_height: 30.0,
            row_gap: 2.0,
            header_height: 44.0,
            handle_width: 7.0,
            bar_rounding: 5.0,
            bar_inset: 3.0,
            status_bar_height: 24.0,
            side_panel_default_width: 340.0,
            side_panel_min_width: 240.0,
        }
    }
}

// ─── Rendering ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeRendering {
    // Bar shadow
    pub bar_shadow_alpha_1: u8,
    pub bar_shadow_offset_y_1: f32,
    pub bar_shadow_alpha_2: u8,
    pub bar_shadow_offset_x_2: f32,
    pub bar_shadow_offset_y_2: f32,

    // Bar material
    pub bar_darken_factor: f32,
    pub bar_glaze_alpha: u8,
    pub bar_glaze_top_frac: f32,
    pub bar_glaze_height_frac: f32,
    pub bar_highlight_alpha: u8,
    pub bar_highlight_height_frac: f32,
    pub bar_bottom_edge_alpha: u8,

    // Progress tick
    pub progress_tick_alpha: u8,

    // Selection glow
    pub selection_glow_outer_alpha: u8,
    pub selection_glow_outer_expand: f32,
    pub selection_glow_inner_expand: f32,

    // Milestone shadow
    pub milestone_shadow_alpha: u8,

    // Dependency arrows
    pub dep_arrow_width: f32,
    pub dep_arrow_head_len: f32,
    pub dep_arrow_head_width: f32,
    pub dep_corner_radius: f32,

    // Row border width
    pub row_border_width: f32,
    /// Header bottom border width
    pub header_border_width: f32,
    /// Grid line width
    pub grid_line_width: f32,

    // Sticky header shadow
    pub sticky_shadow_height: f32,
    pub sticky_shadow_alpha: u8,

    // Weekend separator
    pub weekend_sep_alpha: u8,

    // Today marker
    pub today_diamond_size: f32,
}

impl Default for ThemeRendering {
    fn default() -> Self {
        Self {
            bar_shadow_alpha_1: 28,
            bar_shadow_offset_y_1: 1.0,
            bar_shadow_alpha_2: 22,
            bar_shadow_offset_x_2: 1.0,
            bar_shadow_offset_y_2: 3.0,

            bar_darken_factor: 0.90,
            bar_glaze_alpha: 90,
            bar_glaze_top_frac: 0.18,
            bar_glaze_height_frac: 0.62,
            bar_highlight_alpha: 38,
            bar_highlight_height_frac: 0.36,
            bar_bottom_edge_alpha: 46,

            progress_tick_alpha: 60,

            selection_glow_outer_alpha: 55,
            selection_glow_outer_expand: 3.0,
            selection_glow_inner_expand: 1.5,

            milestone_shadow_alpha: 40,

            dep_arrow_width: 0.9,
            dep_arrow_head_len: 6.0,
            dep_arrow_head_width: 3.4,
            dep_corner_radius: 6.0,

            row_border_width: 0.5,
            header_border_width: 1.0,
            grid_line_width: 0.5,

            sticky_shadow_height: 10.0,
            sticky_shadow_alpha: 38,

            weekend_sep_alpha: 140,

            today_diamond_size: 5.5,
        }
    }
}

// ─── Layout ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeLayout {
    pub panel_inner_margin: f32,
    pub editor_inner_margin: f32,
    pub dialog_width: f32,
    pub about_dialog_width: f32,
    pub about_dialog_height: f32,
}

impl Default for ThemeLayout {
    fn default() -> Self {
        Self {
            panel_inner_margin: 10.0,
            editor_inner_margin: 10.0,
            dialog_width: 320.0,
            about_dialog_width: 300.0,
            about_dialog_height: 160.0,
        }
    }
}

// ─── Motion ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeMotion {
    /// Duration (s) for row reorder animation.
    pub reorder_anim_duration: f32,
}

impl Default for ThemeMotion {
    fn default() -> Self {
        Self {
            reorder_anim_duration: 0.14,
        }
    }
}

// ─── Zoom ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeZoom {
    pub default_pixels_per_day: f32,
    pub min_pixels_per_day: f32,
    pub max_pixels_per_day: f32,
    pub zoom_factor: f32,
    pub vertical_scale_min: f32,
    pub vertical_scale_max: f32,
}

impl Default for ThemeZoom {
    fn default() -> Self {
        Self {
            default_pixels_per_day: 18.0,
            min_pixels_per_day: 2.0,
            max_pixels_per_day: 80.0,
            zoom_factor: 1.2,
            vertical_scale_min: 0.8,
            vertical_scale_max: 1.9,
        }
    }
}
