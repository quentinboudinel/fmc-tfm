use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::Project;

/// Version written to (and checked when reading) the project file's
/// top-level `version` field, per SPECIFICATION.md 7.2.
pub const PROJECT_FORMAT_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    version: String,
    #[serde(flatten)]
    project: Project,
}

#[derive(Debug)]
pub enum ProjectFileError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for ProjectFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectFileError::Io(e) => write!(f, "I/O error: {e}"),
            ProjectFileError::Json(e) => write!(f, "invalid project file: {e}"),
        }
    }
}

impl std::error::Error for ProjectFileError {}

impl From<std::io::Error> for ProjectFileError {
    fn from(e: std::io::Error) -> Self {
        ProjectFileError::Io(e)
    }
}

impl From<serde_json::Error> for ProjectFileError {
    fn from(e: serde_json::Error) -> Self {
        ProjectFileError::Json(e)
    }
}

pub fn save_project<P: AsRef<Path>>(path: P, project: &Project) -> Result<(), ProjectFileError> {
    let file = ProjectFile {
        version: PROJECT_FORMAT_VERSION.to_string(),
        project: project.clone(),
    };
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_project<P: AsRef<Path>>(path: P) -> Result<Project, ProjectFileError> {
    let json = std::fs::read_to_string(path)?;
    let file: ProjectFile = serde_json::from_str(&json)?;
    Ok(file.project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Defect, Material, PointReflector, Probe};

    fn sample_project() -> Project {
        Project {
            material: Material::new(5900.0, 80.0, 40.0),
            probe: Probe::default(),
            defects: vec![Defect::PointReflector(PointReflector {
                x: 20.0,
                y: 10.0,
                amplitude: 0.9,
            })],
        }
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        tempfile::Builder::new()
            .prefix(name)
            .suffix(".json")
            .tempfile()
            .unwrap()
            .into_temp_path()
            .to_path_buf()
    }

    #[test]
    fn roundtrip_preserves_project() {
        let path = temp_path("project_roundtrip");
        let project = sample_project();

        save_project(&path, &project).unwrap();
        let loaded = load_project(&path).unwrap();

        assert_eq!(loaded.material, project.material);
        assert_eq!(loaded.probe, project.probe);
        assert_eq!(loaded.defects, project.defects);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn saved_file_contains_version_field() {
        let path = temp_path("project_version");
        save_project(&path, &sample_project()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(json["version"], PROJECT_FORMAT_VERSION);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_rejects_invalid_json() {
        let path = temp_path("project_invalid");
        std::fs::write(&path, "not valid json").unwrap();

        let result = load_project(&path);
        assert!(result.is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_rejects_missing_file() {
        let result = load_project("/nonexistent/path/project.json");
        assert!(result.is_err());
    }
}
