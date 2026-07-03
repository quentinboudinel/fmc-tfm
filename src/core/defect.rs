use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointReflector {
    pub x: f64,
    pub y: f64,
    pub amplitude: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crack {
    pub x: f64,
    pub y: f64,
    pub length: f64,
    pub angle: f64,
    pub amplitude: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Void {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub amplitude: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Porosity {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub density: f64,
    pub amplitude: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanarDefect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub angle: f64,
    pub amplitude: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Defect {
    PointReflector(PointReflector),
    Crack(Crack),
    Void(Void),
    Porosity(Porosity),
    PlanarDefect(PlanarDefect),
}

impl Defect {
    pub fn position(&self) -> (f64, f64) {
        match self {
            Defect::PointReflector(d) => (d.x, d.y),
            Defect::Crack(d) => (d.x, d.y),
            Defect::Void(d) => (d.x, d.y),
            Defect::Porosity(d) => (d.x, d.y),
            Defect::PlanarDefect(d) => (d.x, d.y),
        }
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        match self {
            Defect::PointReflector(d) => {
                d.x = x;
                d.y = y;
            }
            Defect::Crack(d) => {
                d.x = x;
                d.y = y;
            }
            Defect::Void(d) => {
                d.x = x;
                d.y = y;
            }
            Defect::Porosity(d) => {
                d.x = x;
                d.y = y;
            }
            Defect::PlanarDefect(d) => {
                d.x = x;
                d.y = y;
            }
        }
    }

    pub fn amplitude(&self) -> f64 {
        match self {
            Defect::PointReflector(d) => d.amplitude,
            Defect::Crack(d) => d.amplitude,
            Defect::Void(d) => d.amplitude,
            Defect::Porosity(d) => d.amplitude,
            Defect::PlanarDefect(d) => d.amplitude,
        }
    }

    /// Point-scatterer approximation (x, y, amplitude) used by the ray-based
    /// FMC simulator, which only knows how to bounce a ray off a point.
    pub fn scatter_points(&self) -> Vec<(f64, f64, f64)> {
        match self {
            Defect::PointReflector(d) => vec![(d.x, d.y, d.amplitude)],
            // Idealized: reflection comes from the face of the cavity closest to the surface.
            Defect::Void(d) => vec![(d.x, d.y - d.radius, d.amplitude)],
            Defect::Crack(d) => line_scatter_points(d.x, d.y, d.length, d.angle, d.amplitude),
            Defect::PlanarDefect(d) => line_scatter_points(d.x, d.y, d.width, d.angle, d.amplitude),
            Defect::Porosity(d) => {
                porosity_scatter_points(d.x, d.y, d.radius, d.density, d.amplitude)
            }
        }
    }
}

/// Spacing between sample scatterers along a linear defect (crack/planar), in mm.
const LINE_SCATTER_SPACING_MM: f64 = 0.5;
const LINE_SCATTER_MIN_POINTS: usize = 2;
const LINE_SCATTER_MAX_POINTS: usize = 21;

fn line_scatter_points(
    cx: f64,
    cy: f64,
    length: f64,
    angle: f64,
    amplitude: f64,
) -> Vec<(f64, f64, f64)> {
    let num_points = ((length / LINE_SCATTER_SPACING_MM).round() as usize)
        .clamp(LINE_SCATTER_MIN_POINTS, LINE_SCATTER_MAX_POINTS);
    let (sin_a, cos_a) = angle.sin_cos();
    (0..num_points)
        .map(|i| {
            let t = i as f64 / (num_points - 1) as f64 - 0.5;
            let offset = t * length;
            (cx + offset * cos_a, cy + offset * sin_a, amplitude)
        })
        .collect()
}

/// Maximum number of sub-scatterers for a fully dense porosity cluster.
const POROSITY_MAX_POINTS: usize = 40;

fn porosity_scatter_points(
    cx: f64,
    cy: f64,
    radius: f64,
    density: f64,
    amplitude: f64,
) -> Vec<(f64, f64, f64)> {
    let num_points =
        ((density.clamp(0.0, 1.0) * POROSITY_MAX_POINTS as f64).round() as usize).max(1);
    // Vogel/sunflower spiral: deterministic, well-distributed points within a disc.
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    (0..num_points)
        .map(|i| {
            let r = radius * ((i as f64 + 0.5) / num_points as f64).sqrt();
            let theta = i as f64 * golden_angle;
            (cx + r * theta.cos(), cy + r * theta.sin(), amplitude)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_reflector_position() {
        let defect = Defect::PointReflector(PointReflector {
            x: 10.0,
            y: 20.0,
            amplitude: 1.0,
        });
        assert_eq!(defect.position(), (10.0, 20.0));
        assert_eq!(defect.amplitude(), 1.0);
    }

    #[test]
    fn crack_position() {
        let defect = Defect::Crack(Crack {
            x: 5.0,
            y: 15.0,
            length: 10.0,
            angle: 0.5,
            amplitude: 0.8,
        });
        assert_eq!(defect.position(), (5.0, 15.0));
        assert_eq!(defect.amplitude(), 0.8);
    }

    #[test]
    fn void_position() {
        let defect = Defect::Void(Void {
            x: 30.0,
            y: 25.0,
            radius: 5.0,
            amplitude: 0.9,
        });
        assert_eq!(defect.position(), (30.0, 25.0));
    }

    #[test]
    fn porosity_position() {
        let defect = Defect::Porosity(Porosity {
            x: 40.0,
            y: 35.0,
            radius: 8.0,
            density: 0.5,
            amplitude: 0.7,
        });
        assert_eq!(defect.position(), (40.0, 35.0));
    }

    #[test]
    fn planar_defect_position() {
        let defect = Defect::PlanarDefect(PlanarDefect {
            x: 50.0,
            y: 45.0,
            width: 12.0,
            angle: 1.57,
            amplitude: 0.95,
        });
        assert_eq!(defect.position(), (50.0, 45.0));
    }

    #[test]
    fn set_position_updates_coordinates() {
        let mut defect = Defect::PointReflector(PointReflector {
            x: 0.0,
            y: 0.0,
            amplitude: 1.0,
        });
        defect.set_position(100.0, 200.0);
        assert_eq!(defect.position(), (100.0, 200.0));
    }

    #[test]
    fn serialization_roundtrip() {
        let defect = Defect::Crack(Crack {
            x: 5.0,
            y: 15.0,
            length: 10.0,
            angle: 0.5,
            amplitude: 0.8,
        });
        let json = serde_json::to_string(&defect).unwrap();
        let parsed: Defect = serde_json::from_str(&json).unwrap();
        assert_eq!(defect, parsed);
    }

    #[test]
    fn point_reflector_scatter_points_is_single_point() {
        let defect = Defect::PointReflector(PointReflector {
            x: 10.0,
            y: 20.0,
            amplitude: 0.9,
        });
        let points = defect.scatter_points();
        assert_eq!(points, vec![(10.0, 20.0, 0.9)]);
    }

    #[test]
    fn void_scatter_point_is_at_top_of_cavity() {
        let defect = Defect::Void(Void {
            x: 30.0,
            y: 25.0,
            radius: 5.0,
            amplitude: 0.7,
        });
        let points = defect.scatter_points();
        assert_eq!(points, vec![(30.0, 20.0, 0.7)]);
    }

    #[test]
    fn crack_scatter_points_lie_on_the_line() {
        let defect = Defect::Crack(Crack {
            x: 0.0,
            y: 0.0,
            length: 10.0,
            angle: 0.0,
            amplitude: 1.0,
        });
        let points = defect.scatter_points();
        assert!(points.len() >= 2);
        for &(x, y, amp) in &points {
            assert!((-5.0..=5.0).contains(&x), "x={x} out of range");
            assert!(y.abs() < 1e-9, "y should be ~0 for a horizontal crack");
            assert_eq!(amp, 1.0);
        }
        let min_x = points.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let max_x = points.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        assert!((min_x - (-5.0)).abs() < 1e-6);
        assert!((max_x - 5.0).abs() < 1e-6);
    }

    #[test]
    fn planar_defect_scatter_points_lie_on_the_line() {
        let defect = Defect::PlanarDefect(PlanarDefect {
            x: 50.0,
            y: 40.0,
            width: 8.0,
            angle: std::f64::consts::FRAC_PI_2,
            amplitude: 0.5,
        });
        let points = defect.scatter_points();
        assert!(points.len() >= 2);
        for &(x, _y, amp) in &points {
            assert!(
                (x - 50.0).abs() < 1e-6,
                "x should stay ~50 for vertical line"
            );
            assert_eq!(amp, 0.5);
        }
        let min_y = points.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let max_y = points.iter().map(|p| p.1).fold(f64::MIN, f64::max);
        assert!((min_y - 36.0).abs() < 1e-6);
        assert!((max_y - 44.0).abs() < 1e-6);
    }

    #[test]
    fn porosity_scatter_points_are_within_radius() {
        let defect = Defect::Porosity(Porosity {
            x: 40.0,
            y: 35.0,
            radius: 8.0,
            density: 0.5,
            amplitude: 0.6,
        });
        let points = defect.scatter_points();
        assert!(!points.is_empty());
        for &(x, y, amp) in &points {
            let dist = ((x - 40.0).powi(2) + (y - 35.0).powi(2)).sqrt();
            assert!(
                dist <= 8.0 + 1e-9,
                "point at distance {dist} exceeds radius"
            );
            assert_eq!(amp, 0.6);
        }
    }

    #[test]
    fn porosity_scatter_point_count_scales_with_density() {
        let low = Defect::Porosity(Porosity {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            density: 0.1,
            amplitude: 1.0,
        });
        let high = Defect::Porosity(Porosity {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
            density: 0.9,
            amplitude: 1.0,
        });
        assert!(low.scatter_points().len() < high.scatter_points().len());
    }
}
