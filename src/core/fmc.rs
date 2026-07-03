use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FmcMetadata {
    pub num_elements: usize,
    pub pitch_mm: f64,
    pub element_width_mm: f64,
    pub center_frequency_mhz: f64,
    pub sample_rate_mhz: f64,
    pub num_samples: usize,
    pub material_velocity_mps: f64,
    pub material_width_mm: f64,
    pub material_depth_mm: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FmcData {
    pub metadata: FmcMetadata,
    data: Vec<f32>,
}

impl FmcData {
    pub fn new(metadata: FmcMetadata) -> Self {
        let len = metadata.num_elements * metadata.num_elements * metadata.num_samples;
        Self {
            metadata,
            data: vec![0.0; len],
        }
    }

    pub fn from_raw(metadata: FmcMetadata, data: Vec<f32>) -> Self {
        let expected = metadata.num_elements * metadata.num_elements * metadata.num_samples;
        assert_eq!(
            data.len(),
            expected,
            "raw FMC data length does not match metadata dimensions"
        );
        Self { metadata, data }
    }

    fn index(&self, tx: usize, rx: usize, t: usize) -> usize {
        (tx * self.metadata.num_elements + rx) * self.metadata.num_samples + t
    }

    pub fn get(&self, tx: usize, rx: usize, t: usize) -> f32 {
        self.data[self.index(tx, rx, t)]
    }

    pub fn add(&mut self, tx: usize, rx: usize, t: usize, value: f32) {
        let idx = self.index(tx, rx, t);
        self.data[idx] += value;
    }

    pub fn ascan(&self, tx: usize, rx: usize) -> &[f32] {
        let start = self.index(tx, rx, 0);
        &self.data[start..start + self.metadata.num_samples]
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_metadata() -> FmcMetadata {
        FmcMetadata {
            num_elements: 4,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
            sample_rate_mhz: 100.0,
            num_samples: 10,
            material_velocity_mps: 5900.0,
            material_width_mm: 100.0,
            material_depth_mm: 50.0,
        }
    }

    #[test]
    fn new_is_zero_filled() {
        let fmc = FmcData::new(test_metadata());
        for tx in 0..4 {
            for rx in 0..4 {
                for t in 0..10 {
                    assert_eq!(fmc.get(tx, rx, t), 0.0);
                }
            }
        }
    }

    #[test]
    fn add_accumulates_value() {
        let mut fmc = FmcData::new(test_metadata());
        fmc.add(1, 2, 3, 0.5);
        fmc.add(1, 2, 3, 0.25);
        assert!((fmc.get(1, 2, 3) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn add_only_affects_target_cell() {
        let mut fmc = FmcData::new(test_metadata());
        fmc.add(1, 2, 3, 1.0);
        assert_eq!(fmc.get(0, 2, 3), 0.0);
        assert_eq!(fmc.get(1, 1, 3), 0.0);
        assert_eq!(fmc.get(1, 2, 2), 0.0);
    }

    #[test]
    fn ascan_returns_correct_slice() {
        let mut fmc = FmcData::new(test_metadata());
        fmc.add(2, 3, 0, 1.0);
        fmc.add(2, 3, 5, 2.0);
        fmc.add(2, 3, 9, 3.0);
        let ascan = fmc.ascan(2, 3);
        assert_eq!(ascan.len(), 10);
        assert_eq!(ascan[0], 1.0);
        assert_eq!(ascan[5], 2.0);
        assert_eq!(ascan[9], 3.0);
    }

    #[test]
    fn as_slice_len_matches_dimensions() {
        let fmc = FmcData::new(test_metadata());
        assert_eq!(fmc.as_slice().len(), 4 * 4 * 10);
    }

    #[test]
    fn from_raw_roundtrips_through_get() {
        let metadata = test_metadata();
        let len = metadata.num_elements * metadata.num_elements * metadata.num_samples;
        let raw: Vec<f32> = (0..len).map(|i| i as f32).collect();
        let fmc = FmcData::from_raw(metadata, raw);
        assert_eq!(fmc.get(0, 0, 0), 0.0);
        assert_eq!(fmc.get(0, 0, 1), 1.0);
        assert_eq!(fmc.get(0, 1, 0), 10.0);
    }

    #[test]
    fn metadata_serialization_roundtrip() {
        let metadata = test_metadata();
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: FmcMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata, parsed);
    }
}
