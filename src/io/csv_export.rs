use crate::model::Task;
use std::path::Path;

/// Map progress float back to a human-readable status string.
fn progress_to_status(progress: f32) -> &'static str {
    if progress >= 1.0 {
        "Finished"
    } else if progress >= 0.5 {
        "In Progress"
    } else if progress >= 0.25 {
        "Released"
    } else {
        "Not Started"
    }
}

/// Export tasks to a semicolon-delimited CSV file matching the import format.
///
/// Columns: Task Label ; Start Date ; End Date ; Status
/// Dates are formatted as DD/MM/YYYY.
/// Returns the number of tasks written.
pub fn export_csv(tasks: &[Task], path: &Path) -> Result<usize, String> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_path(path)
        .map_err(|e| format!("Failed to create CSV file: {}", e))?;

    // Write header
    wtr.write_record(["Task Label", "Start Date", "End Date", "Status"])
        .map_err(|e| format!("Failed to write header: {}", e))?;

    // Write each task
    for task in tasks {
        wtr.write_record([
            &task.name,
            &task.start.format("%d/%m/%Y").to_string(),
            &task.end.format("%d/%m/%Y").to_string(),
            progress_to_status(task.progress),
        ])
        .map_err(|e| format!("Failed to write task '{}': {}", task.name, e))?;
    }

    wtr.flush().map_err(|e| format!("Failed to flush CSV: {}", e))?;
    Ok(tasks.len())
}
