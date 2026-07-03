use super::fmc::{FmcData, FmcMetadata};
use super::{Defect, Material, Probe};

/// Number of pulse periods captured within the Gaussian envelope's half-width.
const PULSE_CYCLES: f64 = 3.0;
/// Minimum ray length used in amplitude/TOF calculations, avoiding a singularity
/// when a scatterer sits exactly on the probe surface.
const MIN_DISTANCE_MM: f64 = 0.1;

pub struct FmcSimulator {
    pub sample_rate_mhz: f64,
}

impl Default for FmcSimulator {
    fn default() -> Self {
        Self {
            sample_rate_mhz: 100.0,
        }
    }
}

impl FmcSimulator {
    pub fn simulate(&self, material: &Material, probe: &Probe, defects: &[Defect]) -> FmcData {
        let element_positions = probe.element_positions();
        let num_elements = probe.num_elements;
        let num_samples = self.required_samples(material, probe);

        let metadata = FmcMetadata {
            num_elements,
            pitch_mm: probe.pitch_mm,
            element_width_mm: probe.element_width_mm,
            center_frequency_mhz: probe.center_frequency_mhz,
            sample_rate_mhz: self.sample_rate_mhz,
            num_samples,
            material_velocity_mps: material.velocity_mps,
            material_width_mm: material.width_mm,
            material_depth_mm: material.depth_mm,
        };
        let mut fmc = FmcData::new(metadata);

        let scatterers: Vec<(f64, f64, f64)> =
            defects.iter().flat_map(Defect::scatter_points).collect();
        if scatterers.is_empty() {
            return fmc;
        }

        let dt_us = 1.0 / self.sample_rate_mhz;
        let period_us = 1.0 / probe.center_frequency_mhz;
        let sigma_us = period_us / 2.0;
        let half_window_samples = ((PULSE_CYCLES * period_us) / dt_us).ceil() as isize;

        for (tx, &tx_x) in element_positions.iter().enumerate() {
            let tx_pos = (tx_x, 0.0);
            for (rx, &rx_x) in element_positions.iter().enumerate() {
                let rx_pos = (rx_x, 0.0);
                for &(sx, sy, amp) in &scatterers {
                    let r1 = distance(tx_pos, (sx, sy)).max(MIN_DISTANCE_MM);
                    let r2 = distance((sx, sy), rx_pos).max(MIN_DISTANCE_MM);
                    let tof_us = (r1 + r2) * 1000.0 / material.velocity_mps;
                    let amplitude = amp / (r1.sqrt() * r2.sqrt());
                    add_pulse(
                        &mut fmc,
                        tx,
                        rx,
                        tof_us,
                        amplitude,
                        probe.center_frequency_mhz,
                        dt_us,
                        sigma_us,
                        half_window_samples,
                    );
                }
            }
        }
        fmc
    }

    /// Sizes the time axis so the latest possible echo (round trip to the far
    /// bottom corners of the material, plus the pulse's own tail) still fits.
    fn required_samples(&self, material: &Material, probe: &Probe) -> usize {
        let element_positions = probe.element_positions();
        let corners = [
            (-material.width_mm / 2.0, material.depth_mm),
            (material.width_mm / 2.0, material.depth_mm),
        ];
        let max_r = element_positions
            .iter()
            .flat_map(|&ex| {
                corners
                    .iter()
                    .map(move |&(cx, cy)| distance((ex, 0.0), (cx, cy)))
            })
            .fold(0.0_f64, f64::max);
        let max_round_trip_mm = 2.0 * max_r;
        let max_time_us = max_round_trip_mm * 1000.0 / material.velocity_mps;
        let dt_us = 1.0 / self.sample_rate_mhz;
        let pulse_tail_us = PULSE_CYCLES / probe.center_frequency_mhz;
        (((max_time_us + pulse_tail_us) / dt_us).ceil() as usize).max(2)
    }
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

#[allow(clippy::too_many_arguments)]
fn add_pulse(
    fmc: &mut FmcData,
    tx: usize,
    rx: usize,
    tof_us: f64,
    amplitude: f64,
    freq_mhz: f64,
    dt_us: f64,
    sigma_us: f64,
    half_window_samples: isize,
) {
    let num_samples = fmc.metadata.num_samples as isize;
    let center_sample = (tof_us / dt_us).round() as isize;
    for ds in -half_window_samples..=half_window_samples {
        let idx = center_sample + ds;
        if idx < 0 || idx >= num_samples {
            continue;
        }
        let t_us = idx as f64 * dt_us;
        let rel = t_us - tof_us;
        let envelope = (-(rel * rel) / (2.0 * sigma_us * sigma_us)).exp();
        let carrier = (2.0 * std::f64::consts::PI * freq_mhz * rel).cos();
        fmc.add(
            tx,
            rx,
            idx as usize,
            (amplitude * envelope * carrier) as f32,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PointReflector;

    fn single_element_probe() -> Probe {
        Probe {
            num_elements: 1,
            pitch_mm: 1.0,
            element_width_mm: 0.8,
            center_frequency_mhz: 5.0,
        }
    }

    fn point_defect(x: f64, y: f64, amplitude: f64) -> Defect {
        Defect::PointReflector(PointReflector { x, y, amplitude })
    }

    fn argmax_abs(ascan: &[f32]) -> usize {
        ascan
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap()
    }

    #[test]
    fn no_defects_produces_all_zero_data() {
        let material = Material::new(5000.0, 100.0, 50.0);
        let probe = single_element_probe();
        let sim = FmcSimulator::default();
        let fmc = sim.simulate(&material, &probe, &[]);
        assert!(fmc.as_slice().iter().all(|&v| v == 0.0));
    }

    #[test]
    fn single_point_reflector_peak_at_expected_sample() {
        let material = Material::new(5000.0, 100.0, 50.0);
        let probe = single_element_probe();
        let sim = FmcSimulator::default();
        let defects = [point_defect(0.0, 20.0, 1.0)];
        let fmc = sim.simulate(&material, &probe, &defects);

        // round trip = 40mm at 5000 m/s => 8.0us => sample 800 at 100 MHz
        let expected_sample = 800usize;
        let ascan = fmc.ascan(0, 0);
        assert!(expected_sample < ascan.len());
        assert_eq!(argmax_abs(ascan), expected_sample);
        assert!(ascan[expected_sample] > 0.0);
    }

    #[test]
    fn amplitude_decreases_with_distance() {
        let material = Material::new(5000.0, 100.0, 50.0);
        let probe = single_element_probe();
        let sim = FmcSimulator::default();

        let near = sim.simulate(&material, &probe, &[point_defect(0.0, 10.0, 1.0)]);
        let far = sim.simulate(&material, &probe, &[point_defect(0.0, 40.0, 1.0)]);

        let near_peak = near.ascan(0, 0).iter().cloned().fold(0.0_f32, f32::max);
        let far_peak = far.ascan(0, 0).iter().cloned().fold(0.0_f32, f32::max);

        assert!(near_peak > far_peak, "near={near_peak} far={far_peak}");
    }

    #[test]
    fn multiple_defects_produce_distinct_peaks() {
        let material = Material::new(5000.0, 100.0, 50.0);
        let probe = single_element_probe();
        let sim = FmcSimulator::default();
        let defects = [point_defect(0.0, 10.0, 1.0), point_defect(0.0, 40.0, 1.0)];
        let fmc = sim.simulate(&material, &probe, &defects);
        let ascan = fmc.ascan(0, 0);

        // round trips: 20mm->4.0us->sample 400 ; 80mm->16.0us->sample 1600
        assert!(ascan[400] > 0.0);
        assert!(ascan[1600] > 0.0);
    }

    #[test]
    fn generated_samples_are_not_truncated() {
        let material = Material::default();
        let probe = Probe::default();
        let sim = FmcSimulator::default();
        let defect = point_defect(0.0, material.depth_mm - 1.0, 1.0);
        let fmc = sim.simulate(&material, &probe, &[defect]);

        let energy: f32 = fmc.as_slice().iter().map(|v| v.abs()).sum();
        assert!(
            energy > 0.0,
            "pulses should not be fully clipped out of range"
        );
    }

    #[test]
    fn ascan_is_reciprocal_between_tx_and_rx() {
        let material = Material::new(5000.0, 100.0, 50.0);
        let probe = Probe {
            num_elements: 4,
            pitch_mm: 1.0,
            element_width_mm: 0.8,
            center_frequency_mhz: 5.0,
        };
        let sim = FmcSimulator::default();
        let defects = [point_defect(2.0, 15.0, 1.0), point_defect(-3.0, 25.0, 0.7)];
        let fmc = sim.simulate(&material, &probe, &defects);

        assert_eq!(fmc.ascan(1, 3), fmc.ascan(3, 1));
        assert_eq!(fmc.ascan(0, 2), fmc.ascan(2, 0));
    }
}
