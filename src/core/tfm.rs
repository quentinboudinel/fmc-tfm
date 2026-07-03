use super::fmc::FmcData;

/// Default reconstruction resolution per SPECIFICATION.md 5.5.2.
pub const DEFAULT_RESOLUTION: usize = 300;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TfmGrid {
    pub width_mm: f64,
    pub depth_mm: f64,
    pub res_x: usize,
    pub res_z: usize,
}

impl TfmGrid {
    pub fn new(width_mm: f64, depth_mm: f64, res_x: usize, res_z: usize) -> Self {
        Self {
            width_mm,
            depth_mm,
            res_x,
            res_z,
        }
    }

    /// Grid defaulted to the material's extent at the default resolution,
    /// per SPECIFICATION.md 5.5.2.
    pub fn from_fmc(fmc: &FmcData) -> Self {
        Self {
            width_mm: fmc.metadata.material_width_mm,
            depth_mm: fmc.metadata.material_depth_mm,
            res_x: DEFAULT_RESOLUTION,
            res_z: DEFAULT_RESOLUTION,
        }
    }

    /// World-space (x, z) coordinate of a pixel's center; x centered on 0,
    /// z measured down from the probe surface, matching the defect/probe
    /// coordinate convention used by the simulator.
    pub fn pixel_center(&self, ix: usize, iz: usize) -> (f64, f64) {
        let dx = self.width_mm / self.res_x as f64;
        let dz = self.depth_mm / self.res_z as f64;
        let x = -self.width_mm / 2.0 + (ix as f64 + 0.5) * dx;
        let z = (iz as f64 + 0.5) * dz;
        (x, z)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TfmImage {
    pub grid: TfmGrid,
    values: Vec<f32>,
}

impl TfmImage {
    pub fn new(grid: TfmGrid, values: Vec<f32>) -> Self {
        assert_eq!(
            values.len(),
            grid.res_x * grid.res_z,
            "TfmImage data length does not match grid resolution"
        );
        Self { grid, values }
    }

    pub fn get(&self, ix: usize, iz: usize) -> f32 {
        self.values[iz * self.grid.res_x + ix]
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }

    pub fn max_abs(&self) -> f32 {
        self.values.iter().fold(0.0_f32, |m, v| m.max(v.abs()))
    }

    /// (ix, iz) of the pixel with the greatest absolute value.
    pub fn argmax_abs(&self) -> (usize, usize) {
        let idx = self
            .values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        (idx % self.grid.res_x, idx / self.grid.res_x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::FmcMetadata;

    fn test_fmc() -> FmcData {
        let metadata = FmcMetadata {
            num_elements: 2,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
            sample_rate_mhz: 100.0,
            num_samples: 4,
            material_velocity_mps: 5900.0,
            material_width_mm: 100.0,
            material_depth_mm: 50.0,
        };
        FmcData::from_raw(metadata, vec![0.0; 2 * 2 * 4])
    }

    #[test]
    fn pixel_center_of_first_and_last_pixel() {
        let grid = TfmGrid::new(100.0, 50.0, 10, 5);
        let (x0, z0) = grid.pixel_center(0, 0);
        assert!((x0 - (-45.0)).abs() < 1e-9);
        assert!((z0 - 5.0).abs() < 1e-9);

        let (x9, z4) = grid.pixel_center(9, 4);
        assert!((x9 - 45.0).abs() < 1e-9);
        assert!((z4 - 45.0).abs() < 1e-9);
    }

    #[test]
    fn from_fmc_uses_material_dims_and_default_resolution() {
        let fmc = test_fmc();
        let grid = TfmGrid::from_fmc(&fmc);
        assert_eq!(grid.width_mm, 100.0);
        assert_eq!(grid.depth_mm, 50.0);
        assert_eq!(grid.res_x, DEFAULT_RESOLUTION);
        assert_eq!(grid.res_z, DEFAULT_RESOLUTION);
    }

    #[test]
    fn image_get_matches_row_major_layout() {
        let grid = TfmGrid::new(10.0, 10.0, 3, 2);
        // row-major by iz: [iz=0: 3 values][iz=1: 3 values]
        let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let image = TfmImage::new(grid, values);
        assert_eq!(image.get(0, 0), 0.0);
        assert_eq!(image.get(2, 0), 2.0);
        assert_eq!(image.get(0, 1), 3.0);
        assert_eq!(image.get(2, 1), 5.0);
    }

    #[test]
    fn max_abs_and_argmax_abs() {
        let grid = TfmGrid::new(10.0, 10.0, 3, 2);
        let values = vec![0.0, -1.0, 2.0, -7.0, 4.0, 5.0];
        let image = TfmImage::new(grid, values);
        assert_eq!(image.max_abs(), 7.0);
        assert_eq!(image.argmax_abs(), (0, 1));
    }

    #[test]
    fn as_slice_returns_full_data() {
        let grid = TfmGrid::new(10.0, 10.0, 3, 2);
        let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let image = TfmImage::new(grid, values.clone());
        assert_eq!(image.as_slice(), values.as_slice());
    }
}
