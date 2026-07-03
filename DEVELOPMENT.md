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
- `.github/workflows/release-plz.yml` — CI workflow (runs in `rust:slim` container). The `rust:slim`
  image ships neither the GitHub CLI nor `jq`, both of which `release-plz/action`'s internal
  git-config step shells out to for commit-author identity — every run failed with
  `gh: command not found` until the workflow installed them explicitly (see the "Install git,
  GitHub CLI, and jq" step).
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

**Coordinate convention:** material/defects/`TfmGrid` all use x in `[0, material.width_mm]` and
z (depth) in `[0, material.depth_mm]`, matching how `Canvas` draws and places them — material's
top-left corner is the origin. `Probe::element_positions()` itself is centered on 0 (its own
local frame), so anything placing a probe into the material frame (`FmcSimulator`,
`TfmReconstructor`) must go through `Probe::absolute_element_positions(material_width_mm)`,
which shifts by `+material_width_mm / 2.0` per SPECIFICATION.md 5.3.2 ("centered horizontally").
Forgetting that shift was a real bug caught by visually comparing a placed defect against its
heatmap reconstruction — the two need to overlay at the same normalized position within their
respective panels.

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

- **Project files** (`src/io/project_file.rs`): JSON with a `version` envelope around `Project`,
  wired to Ctrl+S/Ctrl+O and Save/Open Project buttons.
- **FMC file export/import** wired into the control panel: Export writes the last simulated
  `FmcData`; Import reads a file, shows its metadata (SPECIFICATION.md 5.7.2), and only
  reconstructs when the user clicks "Reconstruct" — errors (bad version, wrong shape, missing
  file) are shown as a status message instead of panicking.
- **PNG export** (`ui::export_png` in `src/ui/heatmap.rs`) reuses the heatmap's own
  normalize/colormap pipeline (`render_pixels`), so the exported image matches what's on screen,
  via the `image` crate.
- **File dialogs** use `rfd` with its default (`xdg-portal`) backend — no GTK dependency needed.
- **Comparison view pan/zoom sync** (SPECIFICATION.md 5.6): `Heatmap` doesn't own its own
  zoom/pan; `App` passes `&mut self.canvas.zoom` / `&mut self.canvas.pan` into both `Canvas` and
  `Heatmap`, so dragging or scrolling either panel updates the same shared state. This
  deliberately duplicates `Canvas`'s small `pixels_per_mm`/`world_to_screen`/`screen_to_world`
  formulas as free functions in `heatmap.rs` (parameterized instead of tied to a `Canvas`)
  rather than refactoring `Canvas`'s already-tested API.

## Build & Run

```bash
cargo run --release
```

### Cross-compiling for Windows

`.cargo/config.toml` sets up the `x86_64-pc-windows-gnu` target to link with MinGW and statically
link the C/C++ runtime (`target-feature=+crt-static`), so the resulting `.exe` doesn't need
`libgcc`/`libstdc++`/`libwinpthread` DLLs alongside it. One-time setup on a Debian/Ubuntu-like
host:

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt-get install -y mingw-w64
```

Then:

```bash
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/fmc-tfm.exe
```

This has been verified to *build* cleanly (including the Windows-specific backends for `rfd`,
`arboard`, and `accesskit`), but not run-tested on real Windows/Wine — smoke-test the `.exe`
before distributing it.

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
