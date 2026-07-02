# fmc-tfm

A desktop application for simulating and visualizing ultrasonic imaging using Full Matrix Capture (FMC) and Total Focusing Method (TFM).

## What is this?

This application lets you:

1. **Draw defects** on a virtual material (cracks, voids, inclusions)
2. **Simulate** ultrasonic data acquisition using Full Matrix Capture
3. **Reconstruct** images using the Total Focusing Method
4. **Compare** the original defects with the reconstructed image

Built as a portfolio project to demonstrate real-time NDT (Non-Destructive Testing) imaging techniques.

## Background

### Ultrasonic Array Imaging

Ultrasonic testing uses high-frequency sound waves to detect internal flaws in materials. A **phased array probe** contains multiple transducer elements that can transmit and receive ultrasound independently.

### Full Matrix Capture (FMC)

Traditional phased array imaging fires all elements together with calculated delays. FMC takes a different approach:

```
For each element i (transmitter):
    Fire element i alone
    Record signals on ALL elements simultaneously
```

This produces an N×N matrix of waveforms (N = number of elements), capturing every possible transmit-receive combination. More data means better reconstruction possibilities.

### Total Focusing Method (TFM)

TFM is a post-processing algorithm that synthetically focuses at every point in the image:

```
For each pixel (x, z):
    For each transmitter i:
        For each receiver j:
            Calculate time-of-flight: t = (d(i→pixel) + d(pixel→j)) / velocity
            Add amplitude at time t to pixel value
```

The result is an image where every point is in perfect focus — something impossible with conventional imaging.

### Why does this matter?

FMC/TFM provides:
- **Better resolution** than conventional techniques
- **Full focusing** at all depths simultaneously
- **Post-acquisition flexibility** — change focus, angle, or region of interest without re-scanning
- **Quantitative sizing** of defects

These techniques are used in aerospace, nuclear, oil & gas, and other industries where finding small defects is critical.

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) 1.70.0 or later

### Build

```bash
git clone https://github.com/yourusername/fmc-tfm.git
cd fmc-tfm
cargo build --release
```

### Run

```bash
cargo run --release
```

## Usage

1. **Configure material** — Select a preset (steel, aluminum, etc.) or set custom properties
2. **Add defects** — Click on the canvas to place point reflectors, cracks, or voids
3. **Configure probe** — Set number of elements, pitch, and frequency
4. **Simulate** — Generate FMC data and export to HDF5
5. **Reconstruct** — Load FMC file and view TFM reconstruction
6. **Compare** — View original defects side-by-side with reconstruction

## Documentation

| Document | Description |
|----------|-------------|
| [SPECIFICATION.md](SPECIFICATION.md) | Formal requirements specification (RFC-style) |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Development workflow and methodology |
| [AGENTS.md](AGENTS.md) | Quick reference for AI coding assistants |

## References

- Holmes, C., Drinkwater, B. W., & Wilcox, P. D. (2005). *Post-processing of the full matrix of ultrasonic transmit–receive array data for non-destructive evaluation.* NDT & E International, 38(8), 701-711.
- [Introduction to Phased Array Ultrasonic Testing](https://www.olympus-ims.com/en/ndt-tutorials/flaw-detection/phased-array/)

## License

[GPL-3.0-or-later](LICENSE)
