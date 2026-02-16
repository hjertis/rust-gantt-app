#![allow(dead_code)]
//! Theme manager — loads, saves, switches, and enumerates themes.

use crate::ui::theme_def::ThemeDefinition;
use std::path::PathBuf;

/// Persisted user settings (lives in the OS config directory).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub active_theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            active_theme: "Default Dark".into(),
        }
    }
}

/// Manages all available themes and the active selection.
pub struct ThemeManager {
    /// All loaded themes, keyed by `meta.name`.
    themes: Vec<ThemeDefinition>,
    /// Index into `themes` for the currently active theme.
    active_index: usize,
    /// Path to the user themes directory.
    themes_dir: PathBuf,
    /// Path to the settings file.
    settings_path: PathBuf,
}

impl ThemeManager {
    /// Initialise the theme manager: discover config dir, load built-in +
    /// user themes, apply persisted preference.
    pub fn new() -> Self {
        let (themes_dir, settings_path) = Self::config_paths();

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&themes_dir);

        // Load settings
        let settings = Self::load_settings(&settings_path);

        // Built-in themes
        let mut themes = builtin_themes();

        // Load user themes from disk
        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        match serde_json::from_str::<ThemeDefinition>(&contents) {
                            Ok(def) => {
                                // Don't add if name collides with existing
                                if !themes.iter().any(|t| t.meta.name == def.meta.name) {
                                    themes.push(def);
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: failed to parse theme {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        // Write reference theme on first run (so users have an example to copy)
        let reference_path = themes_dir.join("_reference_default.json");
        if !reference_path.exists() {
            let reference = ThemeDefinition::default();
            if let Ok(json) = serde_json::to_string_pretty(&reference) {
                let _ = std::fs::write(&reference_path, json);
            }
        }

        // Resolve active theme
        let active_index = themes
            .iter()
            .position(|t| t.meta.name == settings.active_theme)
            .unwrap_or(0);

        Self {
            themes,
            active_index,
            themes_dir,
            settings_path,
        }
    }

    // ── Getters ─────────────────────────────────────────────────

    /// The currently active theme.
    pub fn active(&self) -> &ThemeDefinition {
        &self.themes[self.active_index]
    }

    /// List of (index, name) for all themes.
    pub fn list(&self) -> Vec<(usize, String)> {
        self.themes
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.meta.name.clone()))
            .collect()
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn themes_dir(&self) -> &PathBuf {
        &self.themes_dir
    }

    // ── Switching ───────────────────────────────────────────────

    /// Switch to a theme by index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.themes.len() {
            self.active_index = index;
            self.save_settings();
        }
    }

    /// Switch to a theme by name.
    pub fn set_active_by_name(&mut self, name: &str) {
        if let Some(idx) = self.themes.iter().position(|t| t.meta.name == name) {
            self.set_active(idx);
        }
    }

    /// Reload user themes from disk (e.g. after the user edits a JSON file).
    pub fn reload_user_themes(&mut self) {
        let active_name = self.themes[self.active_index].meta.name.clone();

        // Keep only builtins
        let builtins = builtin_themes();
        self.themes = builtins;

        // Re-scan user dir
        if let Ok(entries) = std::fs::read_dir(&self.themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        if let Ok(def) = serde_json::from_str::<ThemeDefinition>(&contents) {
                            if !self.themes.iter().any(|t| t.meta.name == def.meta.name) {
                                self.themes.push(def);
                            }
                        }
                    }
                }
            }
        }

        // Restore selection
        self.active_index = self
            .themes
            .iter()
            .position(|t| t.meta.name == active_name)
            .unwrap_or(0);
    }

    // ── Persistence helpers ─────────────────────────────────────

    fn config_paths() -> (PathBuf, PathBuf) {
        if let Some(proj_dirs) =
            directories::ProjectDirs::from("", "", "RustGanttApp")
        {
            let config = proj_dirs.config_dir().to_path_buf();
            let themes = config.join("themes");
            let settings = config.join("settings.json");
            (themes, settings)
        } else {
            // Fallback
            let dir = PathBuf::from(".");
            (dir.join("themes"), dir.join("settings.json"))
        }
    }

    fn load_settings(path: &PathBuf) -> AppSettings {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_settings(&self) {
        let settings = AppSettings {
            active_theme: self.themes[self.active_index].meta.name.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = std::fs::create_dir_all(self.settings_path.parent().unwrap_or(&self.settings_path));
            let _ = std::fs::write(&self.settings_path, json);
        }
    }
}

// ─── Built-in preset themes ────────────────────────────────────────────────

fn builtin_themes() -> Vec<ThemeDefinition> {
    vec![
        default_dark(),
        midnight_theme(),
        warm_earth_theme(),
        dark_material_theme(),
        clean_light_theme(),
    ]
}

/// The default dark theme (matches original hard-coded values).
fn default_dark() -> ThemeDefinition {
    ThemeDefinition::default()
}

/// A deep midnight blue theme.
fn midnight_theme() -> ThemeDefinition {
    use egui::Color32;
    let mut t = ThemeDefinition::default();
    t.meta = crate::ui::theme_def::ThemeMeta {
        name: "Midnight".into(),
        author: "Built-in".into(),
        description: "Deep midnight blue tones.".into(),
        variant: "dark".into(),
    };
    t.colors.bg_dark = Color32::from_rgb(12, 14, 24);
    t.colors.bg_panel = Color32::from_rgb(16, 20, 34);
    t.colors.bg_header = Color32::from_rgb(20, 26, 44);
    t.colors.border_subtle = Color32::from_rgb(34, 40, 62);
    t.colors.border_accent = Color32::from_rgb(70, 120, 210);
    t.colors.accent = Color32::from_rgb(60, 120, 220);
    t.colors.grid_line = Color32::from_rgb(28, 32, 50);
    t.colors.bg_field = Color32::from_rgb(12, 14, 24);
    t.colors.widget_bg_inactive = Color32::from_rgb(24, 30, 48);
    t.colors.widget_bg_hovered = Color32::from_rgb(34, 42, 66);
    t.colors.widget_bg_active = Color32::from_rgb(42, 50, 76);
    t.colors.widget_bg_open = Color32::from_rgb(30, 38, 60);
    t.colors.faint_bg = Color32::from_rgb(16, 18, 30);
    t.colors.extreme_bg = Color32::from_rgb(10, 12, 20);
    t.colors.status_bar_bg = Color32::from_rgb(14, 16, 28);
    t.colors.task_palette = vec![
        Color32::from_rgb(50, 120, 240),
        Color32::from_rgb(40, 160, 90),
        Color32::from_rgb(160, 60, 200),
        Color32::from_rgb(240, 130, 20),
        Color32::from_rgb(20, 160, 240),
        Color32::from_rgb(220, 50, 60),
        Color32::from_rgb(0, 180, 200),
        Color32::from_rgb(245, 185, 10),
    ];
    t
}

/// A warm earthy tone theme.
fn warm_earth_theme() -> ThemeDefinition {
    use egui::Color32;
    let mut t = ThemeDefinition::default();
    t.meta = crate::ui::theme_def::ThemeMeta {
        name: "Warm Earth".into(),
        author: "Built-in".into(),
        description: "Warm, earthy tones with amber accents.".into(),
        variant: "dark".into(),
    };
    t.colors.bg_dark = Color32::from_rgb(28, 24, 20);
    t.colors.bg_panel = Color32::from_rgb(34, 30, 26);
    t.colors.bg_header = Color32::from_rgb(42, 36, 30);
    t.colors.border_subtle = Color32::from_rgb(62, 54, 44);
    t.colors.border_accent = Color32::from_rgb(200, 150, 60);
    t.colors.accent = Color32::from_rgb(210, 160, 50);
    t.colors.text_primary = Color32::from_rgb(235, 225, 210);
    t.colors.text_secondary = Color32::from_rgb(185, 170, 150);
    t.colors.text_dim = Color32::from_rgb(130, 118, 100);
    t.colors.grid_line = Color32::from_rgb(50, 44, 36);
    t.colors.today_line = Color32::from_rgb(230, 90, 50);
    t.colors.bg_field = Color32::from_rgb(24, 20, 16);
    t.colors.widget_bg_inactive = Color32::from_rgb(44, 38, 32);
    t.colors.widget_bg_hovered = Color32::from_rgb(56, 48, 40);
    t.colors.widget_bg_active = Color32::from_rgb(66, 56, 46);
    t.colors.widget_bg_open = Color32::from_rgb(50, 44, 36);
    t.colors.faint_bg = Color32::from_rgb(32, 28, 22);
    t.colors.extreme_bg = Color32::from_rgb(20, 18, 14);
    t.colors.status_bar_bg = Color32::from_rgb(30, 26, 22);
    t.colors.dep_arrow = Color32::from_rgba_unmultiplied(200, 160, 80, 110);
    t.colors.dep_arrow_hover = Color32::from_rgba_unmultiplied(230, 190, 100, 180);
    t.colors.task_palette = vec![
        Color32::from_rgb(200, 140, 50),
        Color32::from_rgb(100, 160, 70),
        Color32::from_rgb(170, 90, 130),
        Color32::from_rgb(80, 140, 180),
        Color32::from_rgb(190, 100, 60),
        Color32::from_rgb(120, 170, 120),
        Color32::from_rgb(180, 130, 80),
        Color32::from_rgb(140, 110, 160),
    ];
    t
}

/// A flat dark theme inspired by Google's Material Design.
fn dark_material_theme() -> ThemeDefinition {
    use egui::Color32;
    let mut t = ThemeDefinition::default();
    t.meta = crate::ui::theme_def::ThemeMeta {
        name: "Dark Material".into(),
        author: "Built-in".into(),
        description: "Flat dark theme inspired by Google Material Design.".into(),
        variant: "dark".into(),
    };

    // Material dark surfaces — #121212 base with elevation overlays
    t.colors.bg_dark = Color32::from_rgb(18, 18, 18);       // #121212
    t.colors.bg_panel = Color32::from_rgb(30, 30, 30);      // elevation 1
    t.colors.bg_header = Color32::from_rgb(37, 37, 37);     // elevation 2
    t.colors.bg_row_even = Color32::from_rgba_unmultiplied(255, 255, 255, 5);
    t.colors.bg_selected = Color32::from_rgba_unmultiplied(33, 150, 243, 28); // primary tint
    t.colors.bg_field = Color32::from_rgb(24, 24, 24);

    // Material dividers — very subtle
    t.colors.border_subtle = Color32::from_rgb(48, 48, 48);
    t.colors.border_accent = Color32::from_rgb(33, 150, 243); // Material Blue 500

    // Material dark text opacities (high/medium/disabled)
    t.colors.text_primary = Color32::from_rgb(222, 222, 222);   // 87% white
    t.colors.text_secondary = Color32::from_rgb(153, 153, 153); // 60% white
    t.colors.text_dim = Color32::from_rgb(97, 97, 97);          // 38% white
    t.colors.text_on_bar = Color32::from_rgb(255, 255, 255);

    // Material Blue primary
    t.colors.accent = Color32::from_rgb(33, 150, 243);       // Blue 500
    t.colors.today_line = Color32::from_rgb(244, 67, 54);    // Red 500
    t.colors.grid_line = Color32::from_rgb(42, 42, 42);
    t.colors.handle_color = Color32::from_rgb(224, 224, 224);
    t.colors.weekend_shade = Color32::from_rgba_unmultiplied(0, 0, 0, 16);
    t.colors.weekend_header_shade = Color32::from_rgba_unmultiplied(0, 0, 0, 20);
    t.colors.progress_overlay = Color32::from_rgba_unmultiplied(0, 0, 0, 50);

    // Dependency arrows
    t.colors.dep_arrow = Color32::from_rgba_unmultiplied(100, 181, 246, 100); // Blue 300
    t.colors.dep_arrow_hover = Color32::from_rgba_unmultiplied(144, 202, 249, 190); // Blue 200
    t.colors.dep_creating = Color32::from_rgba_unmultiplied(77, 208, 225, 180); // Cyan 300

    // Widget surface colors — Material elevation model
    t.colors.widget_bg_inactive = Color32::from_rgb(40, 40, 40);
    t.colors.widget_bg_hovered = Color32::from_rgb(50, 50, 50);
    t.colors.widget_bg_active = Color32::from_rgb(58, 58, 58);
    t.colors.widget_bg_open = Color32::from_rgb(46, 46, 46);
    t.colors.faint_bg = Color32::from_rgb(22, 22, 22);
    t.colors.extreme_bg = Color32::from_rgb(14, 14, 14);

    t.colors.status_bar_bg = Color32::from_rgb(24, 24, 24);

    t.colors.row_selected_stroke = Color32::from_rgba_unmultiplied(33, 150, 243, 100);
    t.colors.row_unselected_stroke = Color32::from_rgba_unmultiplied(255, 255, 255, 6);

    // Material Design color palette (500 variants)
    t.colors.task_palette = vec![
        Color32::from_rgb(33, 150, 243),   // Blue 500
        Color32::from_rgb(76, 175, 80),    // Green 500
        Color32::from_rgb(171, 71, 188),   // Purple 400
        Color32::from_rgb(255, 152, 0),    // Orange 500
        Color32::from_rgb(0, 188, 212),    // Cyan 500
        Color32::from_rgb(244, 67, 54),    // Red 500
        Color32::from_rgb(0, 150, 136),    // Teal 500
        Color32::from_rgb(255, 193, 7),    // Amber 500
    ];

    // Flat rendering — no shadows, no glaze, no highlights
    t.rendering.bar_shadow_alpha_1 = 0;         // no shadow — completely flat
    t.rendering.bar_shadow_offset_y_1 = 0.0;
    t.rendering.bar_shadow_alpha_2 = 0;
    t.rendering.bar_shadow_offset_x_2 = 0.0;
    t.rendering.bar_shadow_offset_y_2 = 0.0;

    t.rendering.bar_darken_factor = 1.0;        // no darkening — flat colour
    t.rendering.bar_glaze_alpha = 0;            // no glaze — flat
    t.rendering.bar_highlight_alpha = 0;        // no specular highlight — flat
    t.rendering.bar_bottom_edge_alpha = 0;      // no bottom edge — flat

    t.rendering.progress_tick_alpha = 40;
    t.rendering.selection_glow_outer_alpha = 30;
    t.rendering.selection_glow_outer_expand = 2.0;
    t.rendering.selection_glow_inner_expand = 1.0;

    t.rendering.milestone_shadow_alpha = 0;     // flat milestone too
    t.rendering.sticky_shadow_alpha = 20;

    // Material corner radius (slightly more rounded)
    t.sizing.bar_rounding = 4.0;
    t.spacing.widget_rounding = 8.0;
    t.spacing.window_rounding = 12.0;

    t
}

/// A clean light theme.
fn clean_light_theme() -> ThemeDefinition {
    use egui::Color32;
    let mut t = ThemeDefinition::default();
    t.meta = crate::ui::theme_def::ThemeMeta {
        name: "Clean Light".into(),
        author: "Built-in".into(),
        description: "A bright, clean light theme.".into(),
        variant: "light".into(),
    };
    t.colors.bg_dark = Color32::from_rgb(240, 242, 246);
    t.colors.bg_panel = Color32::from_rgb(248, 249, 252);
    t.colors.bg_header = Color32::from_rgb(235, 238, 244);
    t.colors.bg_row_even = Color32::from_rgba_unmultiplied(0, 0, 0, 6);
    t.colors.bg_selected = Color32::from_rgba_unmultiplied(60, 120, 220, 30);
    t.colors.bg_field = Color32::from_rgb(255, 255, 255);

    t.colors.border_subtle = Color32::from_rgb(210, 214, 222);
    t.colors.border_accent = Color32::from_rgb(60, 120, 220);

    t.colors.text_primary = Color32::from_rgb(30, 32, 40);
    t.colors.text_secondary = Color32::from_rgb(80, 86, 100);
    t.colors.text_dim = Color32::from_rgb(140, 146, 158);
    t.colors.text_on_bar = Color32::from_rgb(255, 255, 255);

    t.colors.accent = Color32::from_rgb(50, 110, 210);
    t.colors.today_line = Color32::from_rgb(220, 50, 50);
    t.colors.grid_line = Color32::from_rgb(218, 222, 230);
    t.colors.handle_color = Color32::from_rgb(60, 60, 70);
    t.colors.weekend_shade = Color32::from_rgba_unmultiplied(0, 0, 0, 10);
    t.colors.weekend_header_shade = Color32::from_rgba_unmultiplied(0, 0, 0, 12);
    t.colors.progress_overlay = Color32::from_rgba_unmultiplied(0, 0, 0, 40);

    t.colors.dep_arrow = Color32::from_rgba_unmultiplied(100, 120, 160, 120);
    t.colors.dep_arrow_hover = Color32::from_rgba_unmultiplied(60, 100, 180, 200);
    t.colors.dep_creating = Color32::from_rgba_unmultiplied(40, 140, 240, 180);

    t.colors.widget_bg_inactive = Color32::from_rgb(232, 235, 240);
    t.colors.widget_bg_hovered = Color32::from_rgb(220, 224, 232);
    t.colors.widget_bg_active = Color32::from_rgb(208, 212, 222);
    t.colors.widget_bg_open = Color32::from_rgb(225, 228, 236);
    t.colors.faint_bg = Color32::from_rgb(244, 246, 250);
    t.colors.extreme_bg = Color32::from_rgb(255, 255, 255);

    t.colors.status_bar_bg = Color32::from_rgb(235, 237, 242);

    t.colors.row_selected_stroke = Color32::from_rgba_unmultiplied(60, 120, 220, 140);
    t.colors.row_unselected_stroke = Color32::from_rgba_unmultiplied(0, 0, 0, 14);

    t.colors.task_palette = vec![
        Color32::from_rgb(50, 110, 220),
        Color32::from_rgb(40, 150, 70),
        Color32::from_rgb(150, 50, 180),
        Color32::from_rgb(230, 120, 10),
        Color32::from_rgb(10, 150, 230),
        Color32::from_rgb(210, 40, 50),
        Color32::from_rgb(0, 170, 190),
        Color32::from_rgb(240, 180, 10),
    ];

    t.rendering.bar_shadow_alpha_1 = 14;
    t.rendering.bar_shadow_alpha_2 = 10;
    t.rendering.bar_bottom_edge_alpha = 26;
    t.rendering.bar_highlight_alpha = 50;
    t.rendering.milestone_shadow_alpha = 20;
    t.rendering.sticky_shadow_alpha = 18;
    t
}
