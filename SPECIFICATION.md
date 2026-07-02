# FMC-TFM Software Specification

> See also: [README](README.md) | [Development Workflow](DEVELOPMENT.md)

| Field          | Value                              |
|----------------|------------------------------------|
| Title          | FMC-TFM Desktop Application        |
| Version        | 1.0.0-draft                        |
| Status         | Draft                              |
| Author         | Quentin Boudinel                   |
| Created        | 2026-07-02                         |
| License        | GPL-3.0-or-later                   |

---

## Abstract

This document specifies the requirements for a desktop application that simulates Full Matrix Capture (FMC) ultrasonic data acquisition and performs image reconstruction using the Total Focusing Method (TFM). The application provides an interactive canvas for defining material geometry and defects, generates synthetic FMC datasets, and reconstructs images from FMC data files.

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Terminology](#2-terminology)
3. [System Overview](#3-system-overview)
4. [Technical Requirements](#4-technical-requirements)
5. [Functional Requirements](#5-functional-requirements)
6. [User Interface Requirements](#6-user-interface-requirements)
7. [Data Formats](#7-data-formats)
8. [Performance Requirements](#8-performance-requirements)
9. [Future Considerations](#9-future-considerations)
10. [References](#10-references)

---

## 1. Introduction

### 1.1 Purpose

This specification defines the functional and technical requirements for the FMC-TFM application. The application serves as a demonstration of real-time ultrasonic imaging techniques suitable for integration into portable inspection devices.

### 1.2 Scope

The application SHALL provide two primary capabilities:

1. Simulation of FMC data acquisition from a user-defined material with defects
2. Reconstruction of images from FMC data using the Total Focusing Method

### 1.3 Requirements Notation

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

---

## 2. Terminology

| Term | Definition |
|------|------------|
| FMC | Full Matrix Capture. An ultrasonic data acquisition technique where each element in an array transmits individually while all elements receive, capturing the complete matrix of transmitter-receiver combinations. |
| TFM | Total Focusing Method. A post-processing algorithm that focuses synthetically at every point in the reconstruction grid using FMC data. |
| A-scan | A single time-domain waveform representing amplitude vs. time for one transmitter-receiver pair. |
| Probe | An ultrasonic transducer array consisting of multiple elements. |
| Pitch | The center-to-center distance between adjacent probe elements. |
| Couplant | A medium (typically water or gel) that facilitates sound transmission between the probe and test material. |

---

## 3. System Overview

### 3.1 Architecture

The application SHALL be implemented as a native desktop application with the following characteristics:

- **Runtime:** Single compiled binary with no external runtime dependencies
- **Language:** Rust
- **GUI Framework:** egui (immediate-mode GUI)
- **Platform Support:** Linux, Windows, macOS

### 3.2 High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interface                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Canvas    │  │  Controls   │  │   Visualization     │  │
│  │   Editor    │  │    Panel    │  │      Panel          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Core Engine                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Material   │  │     FMC     │  │        TFM          │  │
│  │    Model    │  │  Simulator  │  │   Reconstructor     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      I/O Layer                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Project   │  │     FMC     │  │       Image         │  │
│  │    Files    │  │    Files    │  │      Export         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Technical Requirements

### 4.1 Build Requirements

| Requirement | Specification |
|-------------|---------------|
| Rust Edition | 2021 or later |
| Minimum Supported Rust Version | 1.70.0 |
| Target Platforms | x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin |

### 4.2 Dependencies

The application SHOULD minimize external dependencies. The following categories of dependencies are permitted:

- GUI rendering (egui, eframe)
- Linear algebra and numerical computation
- HDF5 file I/O
- Image encoding (PNG)
- Serialization (serde)

---

## 5. Functional Requirements

### 5.1 Material Definition

#### 5.1.1 Material Properties

The application MUST allow configuration of the following material properties:

| Property | Unit | Description |
|----------|------|-------------|
| Longitudinal Velocity | m/s | Speed of longitudinal waves in the material |
| Width | mm | Horizontal extent of the material |
| Depth | mm | Vertical extent of the material |

#### 5.1.2 Material Presets

The application MUST provide presets for common materials:

| Material | Longitudinal Velocity (m/s) |
|----------|----------------------------|
| Steel | 5920 |
| Aluminum | 6320 |
| Copper | 4700 |
| Water | 1480 |
| Acrylic (PMMA) | 2730 |

Users MAY define custom material properties.

### 5.2 Defect Definition

#### 5.2.1 Supported Defect Types

The application MUST support the following defect types:

| Type | Parameters | Description |
|------|------------|-------------|
| Point Reflector | position (x, y), amplitude | Omnidirectional point scatterer |
| Crack | position (x, y), length, orientation, amplitude | Linear defect with specular reflection |
| Void | position (x, y), radius or (semi-major, semi-minor), amplitude | Circular or elliptical cavity |
| Porosity | center position, radius, density, point size range | Cluster of small point reflectors |
| Planar Defect | position (x, y), length, orientation, amplitude | Delamination or lack of fusion |

#### 5.2.2 Defect Manipulation

The application MUST support:

- Adding defects via mouse interaction on the canvas
- Selecting and modifying existing defects
- Deleting defects
- Undo and redo operations for all defect manipulations

### 5.3 Probe Configuration

#### 5.3.1 Probe Parameters

The application MUST allow configuration of the following probe parameters:

| Parameter | Unit | Description | Default |
|-----------|------|-------------|---------|
| Number of Elements | - | Count of transducer elements | 64 |
| Pitch | mm | Center-to-center element spacing | 0.5 |
| Element Width | mm | Active width of each element | 0.4 |
| Center Frequency | MHz | Nominal operating frequency | 5.0 |

#### 5.3.2 Probe Positioning

The probe SHALL be positioned at the top surface of the material (y = 0), centered horizontally.

### 5.4 FMC Simulation

#### 5.4.1 Simulation Model

The simulation SHALL implement an idealized ray-based model:

- Direct ray paths from transmitter to defect to receiver
- Time-of-flight calculation based on material velocity
- Amplitude based on defect reflectivity and geometric spreading
- No attenuation modeling
- No noise modeling
- No beam spread modeling

#### 5.4.2 Simulation Output

The simulation SHALL generate:

- Complete FMC dataset (N×N×T array, where N = number of elements, T = time samples)
- Metadata including probe configuration and material properties

### 5.5 TFM Reconstruction

#### 5.5.1 Algorithm

The application MUST implement the Total Focusing Method:

```
For each pixel (x, z) in the reconstruction grid:
    I(x, z) = 0
    For each transmitter i:
        For each receiver j:
            t = (d(i, x, z) + d(j, x, z)) / velocity
            I(x, z) += A[i, j, t]
```

Where:
- `d(i, x, z)` is the distance from element i to point (x, z)
- `A[i, j, t]` is the amplitude from the FMC data (interpolated)

#### 5.5.2 Reconstruction Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| Grid Width | Horizontal extent of reconstruction | Material width |
| Grid Depth | Vertical extent of reconstruction | Material depth |
| Horizontal Resolution | Number of pixels horizontally | 300 |
| Vertical Resolution | Number of pixels vertically | 300 |

#### 5.5.3 Visualization

The reconstruction SHALL be displayed as a heatmap with:

- Configurable colormap
- Adjustable dynamic range (dB scale)
- Adjustable threshold/gain

### 5.6 Comparison View

The application MUST provide a side-by-side comparison view showing:

- Original canvas with defined defects
- TFM reconstruction result

Views SHOULD be synchronized for pan and zoom operations.

### 5.7 File Operations

#### 5.7.1 Project Files

The application MUST support saving and loading project files containing:

- Material configuration
- Defect definitions
- Probe configuration

Project files SHALL use a documented format (see Section 7).

#### 5.7.2 FMC File Import

The application MUST support importing FMC data files with:

- File integrity validation
- Metadata display before processing
- Error reporting for invalid files

#### 5.7.3 Image Export

The application MUST support exporting the reconstruction as a PNG image.

---

## 6. User Interface Requirements

### 6.1 Visual Design

| Aspect | Requirement |
|--------|-------------|
| Theme | Dark theme |
| Style | Minimal, industrial aesthetic |
| Typography | Monospace or technical sans-serif |
| Color Palette | Muted colors with high-contrast data visualization |

### 6.2 Layout

The application SHALL present a single-window interface containing:

1. **Canvas Area** - Material cross-section with defect visualization
2. **Control Panel** - Material, probe, and defect parameters
3. **Visualization Panel** - TFM reconstruction display
4. **Toolbar** - File operations, undo/redo, mode selection

### 6.3 Interaction

| Action | Behavior |
|--------|----------|
| Left Click | Select defect / Place new defect |
| Left Drag | Move selected defect / Draw defect shape |
| Right Click | Context menu (delete, properties) |
| Scroll | Zoom canvas |
| Middle Drag | Pan canvas |
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |
| Ctrl+S | Save project |
| Ctrl+O | Open project |

---

## 7. Data Formats

### 7.1 FMC Data Format (HDF5)

The FMC data file SHALL conform to the following HDF5 structure:

```
/
├── fmc_data                 # Dataset: float32[N, N, T]
│                            # N = number of elements, T = time samples
├── metadata/
│   ├── num_elements         # Attribute: int32
│   ├── pitch_mm             # Attribute: float32
│   ├── element_width_mm     # Attribute: float32
│   ├── center_frequency_mhz # Attribute: float32
│   ├── sample_rate_mhz      # Attribute: float32
│   ├── num_samples          # Attribute: int32
│   └── material/
│       ├── velocity_mps     # Attribute: float32
│       ├── width_mm         # Attribute: float32
│       └── depth_mm         # Attribute: float32
└── version                  # Attribute: string ("1.0")
```

### 7.2 Project File Format

Project files SHALL use JSON with the following schema:

```json
{
  "version": "1.0",
  "material": {
    "velocity_mps": 5920.0,
    "width_mm": 100.0,
    "depth_mm": 50.0,
    "preset": "steel"
  },
  "probe": {
    "num_elements": 64,
    "pitch_mm": 0.5,
    "element_width_mm": 0.4,
    "center_frequency_mhz": 5.0
  },
  "defects": [
    {
      "type": "point",
      "x_mm": 25.0,
      "y_mm": 30.0,
      "amplitude": 1.0
    }
  ]
}
```

---

## 8. Performance Requirements

### 8.1 TFM Reconstruction

| Metric | Requirement |
|--------|-------------|
| Target Reconstruction Time | < 100 ms |
| Reference Configuration | 64 elements, 300×300 grid |

### 8.2 User Interface

| Metric | Requirement |
|--------|-------------|
| Frame Rate | ≥ 60 FPS during idle |
| Input Latency | < 16 ms |

### 8.3 Optimization Strategies

The implementation MAY employ:

- SIMD vectorization
- Multi-threading
- GPU compute shaders (if available)
- Lookup tables for distance calculations

---

## 9. Future Considerations

The following features are explicitly out of scope for version 1.0 but MAY be considered for future versions:

| Feature | Description |
|---------|-------------|
| Multi-probe Support | Multiple probes on different surfaces |
| Wave Animation | Visual demonstration of wave propagation |
| Realistic Simulation | Attenuation, noise, beam spread modeling |
| Theme Toggle | Light/dark theme selection |
| Shear Wave Support | Mode conversion and shear wave propagation |
| SAFT | Synthetic Aperture Focusing Technique as alternative to TFM |

---

## 10. References

1. Holmes, C., Drinkwater, B. W., & Wilcox, P. D. (2005). Post-processing of the full matrix of ultrasonic transmit–receive array data for non-destructive evaluation. *NDT & E International*, 38(8), 701-711.

2. RFC 2119 - Key words for use in RFCs to Indicate Requirement Levels. https://www.rfc-editor.org/rfc/rfc2119

3. HDF5 File Format Specification. https://www.hdfgroup.org/

---

## Appendix A: Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.0.0-draft | 2026-07-02 | Initial draft |
