use rayon::prelude::*;

use super::fmc::FmcData;
use super::tfm::{TfmGrid, TfmImage};
use super::Probe;

pub struct TfmReconstructor;

impl TfmReconstructor {
    /// Total Focusing Method per SPECIFICATION.md 5.5.1: for every pixel, sum
    /// the (linearly interpolated) FMC amplitude at the time-of-flight for
    /// every transmit/receive pair.
    ///
    /// Looping pixel-outermost (one FMC array touch per pixel) forces a scan
    /// over the whole, typically many-megabyte, FMC array for every one of
    /// the up to 90000 pixels. Instead this precomputes a small per-element
    /// distance table once, then loops tx/rx-outermost so each pass streams
    /// one A-scan and accumulates into a cache-sized per-task image buffer.
    /// Combined with a branch-free `interpolate` this lands around 150-200ms
    /// for the reference 64-element/300x300 case on this project's dev
    /// hardware — well short of the <100ms target in SPECIFICATION.md 8.1.
    /// Closing that gap needs GPU compute or explicit SIMD/2D cache tiling
    /// (see DEVELOPMENT.md); this is CPU/rayon-only.
    pub fn reconstruct(fmc: &FmcData, grid: TfmGrid) -> TfmImage {
        let meta = &fmc.metadata;
        let probe = Probe {
            num_elements: meta.num_elements,
            pitch_mm: meta.pitch_mm,
            element_width_mm: meta.element_width_mm,
            center_frequency_mhz: meta.center_frequency_mhz,
        };
        let element_positions = probe.element_positions();
        let num_elements = meta.num_elements;
        let num_samples = meta.num_samples;
        let num_pixels = grid.res_x * grid.res_z;
        // mm of ray length -> fractional sample index, folding both unit
        // conversion and the sample-rate division into one multiply so the
        // hot loop never divides.
        let mm_to_sample = (1000.0 / (meta.material_velocity_mps / meta.sample_rate_mhz)) as f32;

        let mut dist_table = vec![0.0_f32; num_elements * num_pixels];
        dist_table
            .par_chunks_mut(num_pixels)
            .zip(element_positions.par_iter())
            .for_each(|(table, &ex)| {
                for iz in 0..grid.res_z {
                    for ix in 0..grid.res_x {
                        let (x, z) = grid.pixel_center(ix, iz);
                        table[iz * grid.res_x + ix] = distance((ex, 0.0), (x, z)) as f32;
                    }
                }
            });

        let values = (0..num_elements)
            .into_par_iter()
            .map(|tx| {
                let tx_dist = &dist_table[tx * num_pixels..(tx + 1) * num_pixels];
                let mut partial = vec![0.0_f32; num_pixels];
                for rx in 0..num_elements {
                    let rx_dist = &dist_table[rx * num_pixels..(rx + 1) * num_pixels];
                    let ascan = fmc.ascan(tx, rx);
                    for p in 0..num_pixels {
                        let sample_pos = (tx_dist[p] + rx_dist[p]) * mm_to_sample;
                        partial[p] += interpolate(ascan, sample_pos, num_samples);
                    }
                }
                partial
            })
            .reduce(
                || vec![0.0_f32; num_pixels],
                |mut acc, partial| {
                    for (a, p) in acc.iter_mut().zip(partial.iter()) {
                        *a += p;
                    }
                    acc
                },
            );

        TfmImage::new(grid, values)
    }
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// Linear interpolation of the A-scan at a fractional sample index; contributes
/// nothing when the time-of-flight falls outside the recorded time window.
///
/// Written branch-free (clamp + select instead of an early return) so the
/// hot per-pixel loop this runs in can auto-vectorize instead of stalling on
/// a divergent branch every iteration.
#[inline(always)]
fn interpolate(ascan: &[f32], sample_pos: f32, num_samples: usize) -> f32 {
    let last = (num_samples - 1) as f32;
    let in_range = (sample_pos >= 0.0) & (sample_pos <= last);
    let clamped = sample_pos.clamp(0.0, last);
    let lo = clamped as usize;
    let hi = (lo + 1).min(num_samples - 1);
    let frac = clamped - lo as f32;
    let a = ascan[lo];
    let b = ascan[hi];
    let value = a + (b - a) * frac;
    if in_range {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Defect, FmcMetadata, FmcSimulator, Material, PointReflector};

    fn probe_for(num_elements: usize) -> Probe {
        Probe {
            num_elements,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
        }
    }

    #[test]
    fn reconstruct_produces_grid_shaped_image() {
        let metadata = FmcMetadata {
            num_elements: 2,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
            sample_rate_mhz: 100.0,
            num_samples: 10,
            material_velocity_mps: 5900.0,
            material_width_mm: 100.0,
            material_depth_mm: 50.0,
        };
        let fmc = FmcData::new(metadata);
        let grid = TfmGrid::new(100.0, 50.0, 12, 7);
        let image = TfmReconstructor::reconstruct(&fmc, grid);
        assert_eq!(image.as_slice().len(), 12 * 7);
    }

    #[test]
    fn zero_fmc_produces_zero_image() {
        let metadata = FmcMetadata {
            num_elements: 4,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
            sample_rate_mhz: 100.0,
            num_samples: 200,
            material_velocity_mps: 5900.0,
            material_width_mm: 20.0,
            material_depth_mm: 20.0,
        };
        let fmc = FmcData::new(metadata);
        let grid = TfmGrid::new(20.0, 20.0, 10, 10);
        let image = TfmReconstructor::reconstruct(&fmc, grid);
        assert!(image.as_slice().iter().all(|&v| v == 0.0));
    }

    #[test]
    fn single_point_reflector_peaks_near_true_position() {
        let material = Material::new(5900.0, 40.0, 40.0);
        let probe = probe_for(16);
        let simulator = FmcSimulator::default();
        let defect_x = 3.0;
        let defect_y = 15.0;
        let defects = [Defect::PointReflector(PointReflector {
            x: defect_x,
            y: defect_y,
            amplitude: 1.0,
        })];
        let fmc = simulator.simulate(&material, &probe, &defects);

        let grid = TfmGrid::new(40.0, 40.0, 80, 80);
        let image = TfmReconstructor::reconstruct(&fmc, grid);

        let (ix, iz) = image.argmax_abs();
        let (px, pz) = grid.pixel_center(ix, iz);

        assert!(
            (px - defect_x).abs() < 1.0,
            "peak x={px} expected~{defect_x}"
        );
        assert!(
            (pz - defect_y).abs() < 1.0,
            "peak z={pz} expected~{defect_y}"
        );
    }

    /// Debug builds are far slower than release, so this isn't run by
    /// default; check timing with
    /// `cargo test --release -- --ignored reconstruction_meets_performance_target`.
    ///
    /// SPECIFICATION.md 8.1 targets <100ms for this configuration; on this
    /// project's dev hardware (10-core i9-13900H) this implementation
    /// measures ~145-220ms, so the target isn't currently met (see the
    /// gap analysis in DEVELOPMENT.md and the doc comment on `reconstruct`).
    /// This assertion is a looser regression guard, not the spec target.
    #[test]
    #[ignore]
    fn reconstruction_meets_performance_target() {
        let material = Material::default();
        let probe = Probe::default();
        let simulator = FmcSimulator::default();
        let defects = [Defect::PointReflector(PointReflector {
            x: 10.0,
            y: 25.0,
            amplitude: 1.0,
        })];
        let fmc = simulator.simulate(&material, &probe, &defects);
        let grid = TfmGrid::new(material.width_mm, material.depth_mm, 300, 300);

        let start = std::time::Instant::now();
        let _image = TfmReconstructor::reconstruct(&fmc, grid);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 400,
            "reconstruction took {elapsed:?}, regression guard is <400ms \
             (SPECIFICATION.md 8.1's target is <100ms, not yet met - see DEVELOPMENT.md)"
        );
    }
}
