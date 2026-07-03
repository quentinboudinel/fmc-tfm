# Development Workflow

> See also: [README](README.md) | [Specification](SPECIFICATION.md)

## Philosophy

This project follows a **trunk-based development** approach with small, frequent commits directly to `main`. Each commit represents a working state of the application.

## Principles

1. **Test-Driven Development** — Write tests first, then implementation. Red → Green → Refactor
2. **Small iterations** — Each commit introduces a single logical change
3. **Always shippable** — `main` should always compile and run
4. **Incremental delivery** — Features are built in thin vertical slices
5. **Refactor as needed** — Improve structure when patterns emerge, not preemptively

## Commit Guidelines

- Commit messages follow conventional format: `type: description`
- Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`
- Each commit should be atomic and self-contained
- No work-in-progress commits on `main`

## Automated Releases

This project uses [release-plz](https://release-plz.dev/) for automated versioning and releases based on conventional commits:

- **Version bumps**: `feat:` → minor, `fix:` → patch, `BREAKING CHANGE` → major
- **Changelog**: Auto-generated from commit messages
- **GitHub releases**: Created automatically with git tags

Configuration:
- `.github/workflows/release-plz.yml` — CI workflow (runs in `rust:slim` container)
- `release-plz.toml` — Release and changelog settings

## Development Phases

### Phase 1: Foundation
Establish the basic application structure with egui, including window setup, canvas rendering, and basic interaction patterns.

### Phase 2: Material & Defects
Implement the material model, defect types, and canvas editing capabilities including undo/redo.

### Phase 3: FMC Simulation
Build the ray-based FMC simulation engine and HDF5 file output.

HDF5 I/O uses the [`hdf5-pure`](https://crates.io/crates/hdf5-pure) crate rather than the
`hdf5`/`hdf5-metno` bindings: it's a pure-Rust reader/writer with no dependency on the system
`libhdf5` C library, so builds don't need it installed or vendored. It normalizes integer
attributes to `I64` on read regardless of the width they were written with (see
`src/io/fmc_file.rs`), and scalar float metadata is stored as `F64` since the crate has no `F32`
attribute variant — only the `fmc_data` dataset itself is stored as `float32`, per the spec.

### Phase 4: TFM Reconstruction
Implement the Total Focusing Method algorithm with real-time performance and heatmap visualization.

`TfmReconstructor` (`src/core/reconstructor.rs`) parallelizes with `rayon`, precomputes a
per-element distance lookup table (avoiding a sqrt per tx/rx pair), loops tx/rx-outermost so
each A-scan is read from the FMC array exactly once instead of once per pixel, and uses a
branch-free interpolation kernel so the hot loop can auto-vectorize.

**Known gap:** SPECIFICATION.md 8.1 targets <100ms for 64 elements at a 300x300 grid. On this
project's dev hardware (10-core i9-13900H) the current CPU/rayon-only implementation measures
~145-220ms — closer, but short of the target. The remaining gap is dominated by the FMC array
(tens of MB, exceeds L3) and the per-element distance table both needing to be streamed through
for every tx/rx pair; closing it fully would need either GPU compute (wgpu compute shader) or a
proper 2D cache-blocked tiling of the pixel and tx/rx dimensions together, both larger efforts
than implemented so far. The `#[ignore]`d `reconstruction_meets_performance_target` test in
`reconstructor.rs` checks a looser regression-guard bound (<400ms) rather than the spec's <100ms.

### Phase 5: Polish
Add comparison view, file operations, image export, and documentation.

## Build & Run

```bash
cargo run --release
```

## Testing

Rust's native test framework with `cargo test`. For TDD, use `cargo-watch` to auto-run tests on save:

```bash
cargo test                     # Run all tests
cargo test <name>              # Run tests matching name
cargo watch -x test            # TDD: auto-run tests on file change
cargo watch -x "test <name>"   # TDD: watch specific test
```

Install cargo-watch: `cargo install cargo-watch`

### Test Organization

- **Unit tests**: In `src/*.rs` files, inside `#[cfg(test)]` modules
- **Integration tests**: In `tests/` directory, each file is a separate test crate
