use serde::{Deserialize, Serialize};

use super::{Defect, Material, Probe};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Project {
    pub material: Material,
    pub probe: Probe,
    pub defects: Vec<Defect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_project_has_empty_defects() {
        let project = Project::default();
        assert!(project.defects.is_empty());
    }

    #[test]
    fn serialization_roundtrip() {
        let project = Project::default();
        let json = serde_json::to_string(&project).unwrap();
        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.defects.len(), 0);
    }
}
