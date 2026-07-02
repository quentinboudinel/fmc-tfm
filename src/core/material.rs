use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialPreset {
    Steel,
    Aluminum,
    Copper,
    Water,
    Acrylic,
}

impl MaterialPreset {
    pub fn velocity_mps(self) -> f64 {
        match self {
            Self::Steel => 5900.0,
            Self::Aluminum => 6300.0,
            Self::Copper => 4700.0,
            Self::Water => 1480.0,
            Self::Acrylic => 2700.0,
        }
    }

    pub fn all() -> &'static [MaterialPreset] {
        &[
            Self::Steel,
            Self::Aluminum,
            Self::Copper,
            Self::Water,
            Self::Acrylic,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub velocity_mps: f64,
    pub width_mm: f64,
    pub depth_mm: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self::from_preset(MaterialPreset::Steel, 100.0, 50.0)
    }
}

impl Material {
    pub fn new(velocity_mps: f64, width_mm: f64, depth_mm: f64) -> Self {
        Self {
            velocity_mps,
            width_mm,
            depth_mm,
        }
    }

    pub fn from_preset(preset: MaterialPreset, width_mm: f64, depth_mm: f64) -> Self {
        Self {
            velocity_mps: preset.velocity_mps(),
            width_mm,
            depth_mm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_velocities_are_correct() {
        assert_eq!(MaterialPreset::Steel.velocity_mps(), 5900.0);
        assert_eq!(MaterialPreset::Aluminum.velocity_mps(), 6300.0);
        assert_eq!(MaterialPreset::Copper.velocity_mps(), 4700.0);
        assert_eq!(MaterialPreset::Water.velocity_mps(), 1480.0);
        assert_eq!(MaterialPreset::Acrylic.velocity_mps(), 2700.0);
    }

    #[test]
    fn from_preset_sets_velocity() {
        let material = Material::from_preset(MaterialPreset::Aluminum, 80.0, 40.0);
        assert_eq!(material.velocity_mps, 6300.0);
        assert_eq!(material.width_mm, 80.0);
        assert_eq!(material.depth_mm, 40.0);
    }

    #[test]
    fn default_is_steel() {
        let material = Material::default();
        assert_eq!(material.velocity_mps, 5900.0);
        assert_eq!(material.width_mm, 100.0);
        assert_eq!(material.depth_mm, 50.0);
    }

    #[test]
    fn serialization_roundtrip() {
        let material = Material::from_preset(MaterialPreset::Copper, 120.0, 60.0);
        let json = serde_json::to_string(&material).unwrap();
        let deserialized: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(material, deserialized);
    }

    #[test]
    fn preset_serialization_roundtrip() {
        let preset = MaterialPreset::Water;
        let json = serde_json::to_string(&preset).unwrap();
        let deserialized: MaterialPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(preset, deserialized);
    }
}
