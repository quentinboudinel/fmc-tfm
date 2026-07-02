use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Probe {
    pub num_elements: usize,
    pub pitch_mm: f64,
    pub element_width_mm: f64,
    pub center_frequency_mhz: f64,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            num_elements: 64,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
        }
    }
}

impl Probe {
    pub fn element_positions(&self) -> Vec<f64> {
        let total = self.total_width();
        let start = -total / 2.0 + self.pitch_mm / 2.0;
        (0..self.num_elements)
            .map(|i| start + i as f64 * self.pitch_mm)
            .collect()
    }

    pub fn total_width(&self) -> f64 {
        self.num_elements as f64 * self.pitch_mm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let probe = Probe::default();
        assert_eq!(probe.num_elements, 64);
        assert!((probe.pitch_mm - 0.5).abs() < 1e-9);
        assert!((probe.element_width_mm - 0.4).abs() < 1e-9);
        assert!((probe.center_frequency_mhz - 5.0).abs() < 1e-9);
    }

    #[test]
    fn element_positions_count() {
        let probe = Probe::default();
        let positions = probe.element_positions();
        assert_eq!(positions.len(), probe.num_elements);
    }

    #[test]
    fn element_positions_spacing() {
        let probe = Probe::default();
        let positions = probe.element_positions();
        for i in 1..positions.len() {
            let spacing = positions[i] - positions[i - 1];
            assert!((spacing - probe.pitch_mm).abs() < 1e-9);
        }
    }

    #[test]
    fn element_positions_centered() {
        let probe = Probe::default();
        let positions = probe.element_positions();
        let sum: f64 = positions.iter().sum();
        assert!(sum.abs() < 1e-9, "positions should sum to ~0 when centered");
    }

    #[test]
    fn total_width_calculation() {
        let probe = Probe::default();
        let expected = 64.0 * 0.5;
        assert!((probe.total_width() - expected).abs() < 1e-9);
    }

    #[test]
    fn serialization_roundtrip() {
        let probe = Probe::default();
        let json = serde_json::to_string(&probe).unwrap();
        let parsed: Probe = serde_json::from_str(&json).unwrap();
        assert_eq!(probe, parsed);
    }
}
