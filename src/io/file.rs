use crate::model::Project;
use std::path::PathBuf;

/// Save a project to a JSON file.
pub fn save_project(project: &Project, path: &PathBuf) -> Result<(), String> {
    let json = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

/// Load a project from a JSON file.
pub fn load_project(path: &PathBuf) -> Result<Project, String> {
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}
