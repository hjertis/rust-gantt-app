use std::path::PathBuf;

use chrono::NaiveDate;

use crate::model::Task;
use crate::ui::theme;

/// Map a status string to a progress value (0.0 – 1.0).
fn status_to_progress(status: &str) -> f32 {
    match status.trim().to_lowercase().as_str() {
        "finished" | "done" | "complete" | "completed" => 1.0,
        "in progress" | "in-progress" | "active" | "started" => 0.5,
        "released" | "planned" => 0.25,
        "firm planned" | "firm-planned" | "not started" | "not-started" | "new" => 0.0,
        _ => 0.0,
    }
}

/// Try parsing a date string with several common formats.
fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    for fmt in &["%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%d-%m-%Y", "%d.%m.%Y", "%Y/%m/%d", "%m-%d-%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

/// Detect delimiter by checking the first line for common separators.
fn detect_delimiter(first_line: &str) -> u8 {
    let semicolons = first_line.matches(';').count();
    let commas = first_line.matches(',').count();
    let tabs = first_line.matches('\t').count();

    if semicolons >= commas && semicolons >= tabs {
        b';'
    } else if tabs >= commas {
        b'\t'
    } else {
        b','
    }
}

/// Normalize a header string to a canonical column key.
fn normalize_header(h: &str) -> String {
    h.trim()
        .to_lowercase()
        .replace([' ', '-', '_'], "")
}

/// Map a normalized header to our column index:
///   0 = name, 1 = start, 2 = end, 3 = status
fn header_to_col(normalized: &str) -> Option<usize> {
    match normalized {
        "name" | "task" | "tasklabel" | "taskname" | "label" | "title"
        | "description" | "activity" => Some(0),

        "start" | "startdate" | "from" | "begin" | "begindate" => Some(1),

        "end" | "enddate" | "to" | "finish" | "finishdate" | "due" | "duedate" => Some(2),

        "status" | "state" | "progress" | "stage" | "phase" => Some(3),

        _ => None,
    }
}

/// Import tasks from a CSV file.
///
/// Auto-detects delimiter (comma, semicolon, tab).
/// Matches column headers flexibly (e.g. "Task Label", "Start Date", etc.).
/// Returns `(tasks, skipped_count)` on success.
pub fn import_csv(path: &PathBuf) -> Result<(Vec<Task>, usize), String> {
    // Read the whole file to detect delimiter from the first line
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let first_line = content.lines().next().unwrap_or("");
    let delimiter = detect_delimiter(first_line);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    // Parse headers and map them to column indices
    let headers = reader
        .headers()
        .map_err(|e| format!("Failed to read CSV headers: {}", e))?
        .clone();

    let col_map: Vec<Option<usize>> = headers
        .iter()
        .map(|h| header_to_col(&normalize_header(h)))
        .collect();

    // Verify we have at least name, start, end
    let has_name = col_map.iter().any(|c| *c == Some(0));
    let has_start = col_map.iter().any(|c| *c == Some(1));
    let has_end = col_map.iter().any(|c| *c == Some(2));

    if !has_name || !has_start || !has_end {
        let found: Vec<&str> = headers.iter().collect();
        return Err(format!(
            "CSV is missing required columns. Found headers: {:?}. \
             Need columns for: task name, start date, end date.",
            found
        ));
    }

    let colors = theme::task_palette();
    let mut tasks = Vec::new();
    let mut skipped = 0usize;

    for (i, result) in reader.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping CSV row {}: {}", i + 2, e);
                skipped += 1;
                continue;
            }
        };

        // Extract fields by mapped column positions
        let mut name_val = None;
        let mut start_val = None;
        let mut end_val = None;
        let mut status_val = None;

        for (col_idx, field) in record.iter().enumerate() {
            if col_idx < col_map.len() {
                match col_map[col_idx] {
                    Some(0) => name_val = Some(field.trim().to_string()),
                    Some(1) => start_val = Some(field.trim().to_string()),
                    Some(2) => end_val = Some(field.trim().to_string()),
                    Some(3) => status_val = Some(field.trim().to_string()),
                    _ => {}
                }
            }
        }

        let name = match name_val {
            Some(n) if !n.is_empty() => n,
            _ => { skipped += 1; continue; }
        };

        let start = match start_val.as_deref().and_then(parse_date) {
            Some(d) => d,
            None => {
                eprintln!("Skipping row {}: invalid start date '{}'", i + 2, start_val.as_deref().unwrap_or(""));
                skipped += 1;
                continue;
            }
        };

        let end = match end_val.as_deref().and_then(parse_date) {
            Some(d) => d,
            None => {
                eprintln!("Skipping row {}: invalid end date '{}'", i + 2, end_val.as_deref().unwrap_or(""));
                skipped += 1;
                continue;
            }
        };

        let progress = status_val
            .as_deref()
            .map(status_to_progress)
            .unwrap_or(0.0);

        let mut task = Task::new(name, start, end.max(start));
        task.progress = progress;
        task.color = colors[tasks.len() % colors.len()];
        tasks.push(task);
    }

    if tasks.is_empty() && skipped > 0 {
        return Err(format!(
            "No valid tasks found in CSV ({} rows skipped)",
            skipped
        ));
    }
    if tasks.is_empty() {
        return Err("CSV file is empty or has no data rows".to_string());
    }

    Ok((tasks, skipped))
}
