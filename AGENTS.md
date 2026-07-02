# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FMC-TFM is a Rust desktop application for simulating Full Matrix Capture (FMC) ultrasonic data and reconstructing images using the Total Focusing Method (TFM).

See also: [README](README.md) | [Specification](SPECIFICATION.md) | [Development](DEVELOPMENT.md)

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build (required for performance testing)
cargo run --release      # Run application
cargo test               # Run all tests
cargo test <name>        # Run specific test
cargo clippy             # Lint
cargo fmt                # Format code
```

## Architecture

Three-layer architecture:

```
UI Layer        → egui immediate-mode GUI (canvas, controls, visualization)
Core Engine     → Material model, FMC simulator, TFM reconstructor
I/O Layer       → Project files (JSON), FMC files (HDF5), image export (PNG)
```

Key data flow:
1. User draws defects on canvas → Material model stores defect list
2. FMC Simulator generates N×N×T array from material/defects → HDF5 output
3. TFM Reconstructor processes FMC data → heatmap visualization

## Domain Concepts

- **FMC (Full Matrix Capture)**: Each probe element transmits while all receive, producing N×N transmit-receive pairs
- **TFM (Total Focusing Method)**: Synthetic focusing at every pixel using time-of-flight from FMC data
- **A-scan**: Single time-domain waveform for one tx-rx pair

TFM algorithm core:
```
I(x,z) = Σᵢ Σⱼ A[i,j,t]  where t = (d(i,x,z) + d(j,x,z)) / velocity
```

## Development Workflow

Trunk-based development on `main`. Each commit must compile and run. Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `test:`.

## Performance Targets

- TFM reconstruction: <100ms for 64 elements, 300×300 grid
- UI: ≥60 FPS idle, <16ms input latency

May require SIMD, multi-threading, or GPU compute.

## Agent Instructions

- **Document changes**: When adding new tooling, CI workflows, or infrastructure, update the relevant documentation (DEVELOPMENT.md, README.md) in the same commit. Don't leave documentation for a follow-up.
