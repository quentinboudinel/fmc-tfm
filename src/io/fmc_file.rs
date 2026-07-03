use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use hdf5_pure::{AttrValue, File, FileBuilder};

use crate::core::{FmcData, FmcMetadata};

/// Version written to the `version` root attribute; read back and checked on import.
pub const FORMAT_VERSION: &str = "1.0";

#[derive(Debug)]
pub enum FmcFileError {
    Hdf5(hdf5_pure::Error),
    MissingAttribute(&'static str),
    UnsupportedVersion(String),
    ShapeMismatch {
        expected: [u64; 3],
        actual: Vec<u64>,
    },
}

impl fmt::Display for FmcFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FmcFileError::Hdf5(e) => write!(f, "HDF5 error: {e}"),
            FmcFileError::MissingAttribute(name) => {
                write!(f, "missing or malformed metadata attribute: {name}")
            }
            FmcFileError::UnsupportedVersion(v) => write!(f, "unsupported FMC file version: {v}"),
            FmcFileError::ShapeMismatch { expected, actual } => write!(
                f,
                "fmc_data shape mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for FmcFileError {}

impl From<hdf5_pure::Error> for FmcFileError {
    fn from(e: hdf5_pure::Error) -> Self {
        FmcFileError::Hdf5(e)
    }
}

pub fn write_fmc_file<P: AsRef<Path>>(path: P, fmc: &FmcData) -> Result<(), FmcFileError> {
    let meta = &fmc.metadata;
    let n = meta.num_elements as u64;
    let t = meta.num_samples as u64;

    let mut builder = FileBuilder::new();
    builder
        .create_dataset("fmc_data")
        .with_f32_data(fmc.as_slice())
        .with_shape(&[n, n, t]);
    builder.set_attr("version", AttrValue::String(FORMAT_VERSION.to_string()));

    let mut metadata_group = builder.create_group("metadata");
    metadata_group.set_attr("num_elements", AttrValue::I32(meta.num_elements as i32));
    metadata_group.set_attr("pitch_mm", AttrValue::F64(meta.pitch_mm));
    metadata_group.set_attr("element_width_mm", AttrValue::F64(meta.element_width_mm));
    metadata_group.set_attr(
        "center_frequency_mhz",
        AttrValue::F64(meta.center_frequency_mhz),
    );
    metadata_group.set_attr("sample_rate_mhz", AttrValue::F64(meta.sample_rate_mhz));
    metadata_group.set_attr("num_samples", AttrValue::I32(meta.num_samples as i32));

    let mut material_group = metadata_group.create_group("material");
    material_group.set_attr("velocity_mps", AttrValue::F64(meta.material_velocity_mps));
    material_group.set_attr("width_mm", AttrValue::F64(meta.material_width_mm));
    material_group.set_attr("depth_mm", AttrValue::F64(meta.material_depth_mm));
    metadata_group.add_group(material_group.finish());
    builder.add_group(metadata_group.finish());

    builder.write(path)?;
    Ok(())
}

pub fn read_fmc_file<P: AsRef<Path>>(path: P) -> Result<FmcData, FmcFileError> {
    let file = File::open(path)?;

    let root_attrs = file.root().attrs()?;
    let version = match root_attrs.get("version") {
        Some(AttrValue::String(v)) | Some(AttrValue::AsciiString(v)) => v.clone(),
        _ => return Err(FmcFileError::MissingAttribute("version")),
    };
    if version != FORMAT_VERSION {
        return Err(FmcFileError::UnsupportedVersion(version));
    }

    let metadata_attrs = file.group("metadata")?.attrs()?;
    let num_elements = get_i32(&metadata_attrs, "num_elements")? as usize;
    let pitch_mm = get_f64(&metadata_attrs, "pitch_mm")?;
    let element_width_mm = get_f64(&metadata_attrs, "element_width_mm")?;
    let center_frequency_mhz = get_f64(&metadata_attrs, "center_frequency_mhz")?;
    let sample_rate_mhz = get_f64(&metadata_attrs, "sample_rate_mhz")?;
    let num_samples = get_i32(&metadata_attrs, "num_samples")? as usize;

    let material_attrs = file.group("metadata/material")?.attrs()?;
    let material_velocity_mps = get_f64(&material_attrs, "velocity_mps")?;
    let material_width_mm = get_f64(&material_attrs, "width_mm")?;
    let material_depth_mm = get_f64(&material_attrs, "depth_mm")?;

    let metadata = FmcMetadata {
        num_elements,
        pitch_mm,
        element_width_mm,
        center_frequency_mhz,
        sample_rate_mhz,
        num_samples,
        material_velocity_mps,
        material_width_mm,
        material_depth_mm,
    };

    let dataset = file.dataset("fmc_data")?;
    let shape = dataset.shape()?;
    let expected = [num_elements as u64, num_elements as u64, num_samples as u64];
    if shape != expected {
        return Err(FmcFileError::ShapeMismatch {
            expected,
            actual: shape,
        });
    }
    let data = dataset.read_f32()?;

    Ok(FmcData::from_raw(metadata, data))
}

fn get_i32(attrs: &HashMap<String, AttrValue>, name: &'static str) -> Result<i32, FmcFileError> {
    match attrs.get(name) {
        Some(AttrValue::I32(v)) => Ok(*v),
        // hdf5-pure normalizes integer attributes to I64 on read regardless
        // of the width they were written with.
        Some(AttrValue::I64(v)) => Ok(*v as i32),
        _ => Err(FmcFileError::MissingAttribute(name)),
    }
}

fn get_f64(attrs: &HashMap<String, AttrValue>, name: &'static str) -> Result<f64, FmcFileError> {
    match attrs.get(name) {
        Some(AttrValue::F64(v)) => Ok(*v),
        _ => Err(FmcFileError::MissingAttribute(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::FmcMetadata;

    fn sample_metadata() -> FmcMetadata {
        FmcMetadata {
            num_elements: 2,
            pitch_mm: 0.5,
            element_width_mm: 0.4,
            center_frequency_mhz: 5.0,
            sample_rate_mhz: 100.0,
            num_samples: 3,
            material_velocity_mps: 5900.0,
            material_width_mm: 100.0,
            material_depth_mm: 50.0,
        }
    }

    fn sample_fmc() -> FmcData {
        let metadata = sample_metadata();
        let len = metadata.num_elements * metadata.num_elements * metadata.num_samples;
        let raw: Vec<f32> = (0..len).map(|i| i as f32 * 0.1).collect();
        FmcData::from_raw(metadata, raw)
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        tempfile::Builder::new()
            .prefix(name)
            .suffix(".h5")
            .tempfile()
            .unwrap()
            .into_temp_path()
            .to_path_buf()
    }

    #[test]
    fn roundtrip_preserves_metadata_and_data() {
        let path = temp_path("fmc_roundtrip");
        let fmc = sample_fmc();

        write_fmc_file(&path, &fmc).unwrap();
        let loaded = read_fmc_file(&path).unwrap();

        assert_eq!(loaded.metadata, fmc.metadata);
        assert_eq!(loaded.as_slice(), fmc.as_slice());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn write_sets_version_attribute() {
        let path = temp_path("fmc_version");
        write_fmc_file(&path, &sample_fmc()).unwrap();

        let file = File::open(&path).unwrap();
        let attrs = file.root().attrs().unwrap();
        assert_eq!(
            attrs.get("version"),
            Some(&AttrValue::String(FORMAT_VERSION.to_string()))
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_rejects_wrong_version() {
        let path = temp_path("fmc_bad_version");
        let mut builder = FileBuilder::new();
        builder.set_attr("version", AttrValue::String("9.9".to_string()));
        builder
            .create_dataset("fmc_data")
            .with_f32_data(&[0.0; 4])
            .with_shape(&[1, 1, 4]);
        builder.write(&path).unwrap();

        let err = read_fmc_file(&path).unwrap_err();
        assert!(matches!(err, FmcFileError::UnsupportedVersion(v) if v == "9.9"));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_rejects_missing_metadata() {
        let path = temp_path("fmc_no_metadata");
        let mut builder = FileBuilder::new();
        builder.set_attr("version", AttrValue::String(FORMAT_VERSION.to_string()));
        builder
            .create_dataset("fmc_data")
            .with_f32_data(&[0.0; 4])
            .with_shape(&[1, 1, 4]);
        builder.write(&path).unwrap();

        let err = read_fmc_file(&path);
        assert!(err.is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_rejects_shape_mismatch() {
        let path = temp_path("fmc_shape_mismatch");
        let fmc = sample_fmc();
        write_fmc_file(&path, &fmc).unwrap();

        // Corrupt the file by rewriting fmc_data with a different shape but
        // leaving metadata claiming the original dimensions.
        let mut builder = FileBuilder::new();
        builder.set_attr("version", AttrValue::String(FORMAT_VERSION.to_string()));
        builder
            .create_dataset("fmc_data")
            .with_f32_data(&[0.0; 8])
            .with_shape(&[2, 2, 2]);
        let mut metadata_group = builder.create_group("metadata");
        metadata_group.set_attr("num_elements", AttrValue::I32(2));
        metadata_group.set_attr("pitch_mm", AttrValue::F64(0.5));
        metadata_group.set_attr("element_width_mm", AttrValue::F64(0.4));
        metadata_group.set_attr("center_frequency_mhz", AttrValue::F64(5.0));
        metadata_group.set_attr("sample_rate_mhz", AttrValue::F64(100.0));
        metadata_group.set_attr("num_samples", AttrValue::I32(3));
        let mut material_group = metadata_group.create_group("material");
        material_group.set_attr("velocity_mps", AttrValue::F64(5900.0));
        material_group.set_attr("width_mm", AttrValue::F64(100.0));
        material_group.set_attr("depth_mm", AttrValue::F64(50.0));
        metadata_group.add_group(material_group.finish());
        builder.add_group(metadata_group.finish());
        builder.write(&path).unwrap();

        let err = read_fmc_file(&path).unwrap_err();
        assert!(matches!(err, FmcFileError::ShapeMismatch { .. }));

        std::fs::remove_file(&path).ok();
    }
}
