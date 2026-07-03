# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Build and attach release binaries for Linux/Windows/macOS (d211b57)
- Add defect overlap rules for placement (5ccd6d8)
- Add project/FMC file I/O, PNG export, and synced pan/zoom (Phase 5) (cc23734)
- Add TFM heatmap visualization and fix probe coordinate frame (d51bbdc)
- Add TFM reconstruction engine (bb95a35)
- Add HDF5 FMC file export/import (3352b2c)
- Add ray-based FMC simulator (e332ebc)
- Add defect placement, selection, and control panel UI (1da504e)
- Add Project model, command pattern, and defect rendering (ba55bfd)
- Define core domain models (Material, Defect, Probe) (de0512c)
- Render material cross-section with grid, probe, and pan/zoom (f9f3014)
- Implement canvas widget with coordinate system (0204ff4)
- Add eframe with minimal window (44975ee)

### Fixed

- Disable crates.io publish in release-plz config (8c6ddc8)
- Mark workspace as a safe git directory in CI (e57cb67)
- Install gh and jq in release-plz CI workflow (e66754f)
- Make the control panel scrollable (5b18f5a)

### Other

- Add Windows cross-compile config (633726a)
- Apply rustfmt to pre-existing Phase 1/2 files (f88c673)
- Mandate TDD as core development principle (e22b81c)
- Setup TDD infrastructure with native Rust tooling (1227f91)
- Document release-plz setup and add agent instruction (217e1a2)
- Bootstrap Rust project with release-plz (0aa305d)
- Initial project documentation (9d4bedb)
