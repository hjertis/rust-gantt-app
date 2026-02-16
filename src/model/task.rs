use chrono::NaiveDate;
use egui::Color32;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the type of dependency between two tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    FinishToStart,
    StartToStart,
    FinishToFinish,
    StartToFinish,
}

/// A dependency link between two tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from_task: Uuid,
    pub to_task: Uuid,
    pub kind: DependencyKind,
}

/// A single task or milestone in the Gantt chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
    /// Progress from 0.0 (not started) to 1.0 (complete).
    pub progress: f32,
    /// Optional group/category name for organizing tasks.
    pub group: Option<String>,
    /// Display color for the task bar (stored as RGBA).
    #[serde(with = "color_serde")]
    pub color: Color32,
    /// If true, this is a milestone (rendered as a diamond, zero-duration).
    pub is_milestone: bool,
}

impl Task {
    /// Create a new task with sensible defaults.
    pub fn new(name: impl Into<String>, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            start,
            end,
            progress: 0.0,
            group: None,
            color: Color32::from_rgb(70, 130, 180), // Steel blue
            is_milestone: false,
        }
    }

    /// Create a new milestone.
    pub fn new_milestone(name: impl Into<String>, date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            start: date,
            end: date,
            progress: 0.0,
            group: None,
            color: Color32::from_rgb(255, 165, 0), // Orange
            is_milestone: true,
        }
    }

}

/// Serde helper for `Color32`.
mod color_serde {
    use egui::Color32;
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        rgba.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rgba: [u8; 4] = Deserialize::deserialize(deserializer)?;
        Ok(Color32::from_rgba_premultiplied(
            rgba[0], rgba[1], rgba[2], rgba[3],
        ))
    }
}
