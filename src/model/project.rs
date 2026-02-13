use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::task::{Dependency, Task};

/// A Gantt project containing tasks, dependencies, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub tasks: Vec<Task>,
    pub dependencies: Vec<Dependency>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            tasks: Vec::new(),
            dependencies: Vec::new(),
            created: Utc::now(),
            modified: Utc::now(),
        }
    }
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Touch the modified timestamp.
    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }
}
