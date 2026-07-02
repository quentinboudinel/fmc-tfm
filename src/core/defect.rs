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
}
