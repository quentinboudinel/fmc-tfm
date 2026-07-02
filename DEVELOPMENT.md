# Development Workflow

> See also: [README](README.md) | [Specification](SPECIFICATION.md)

## Philosophy

This project follows a **trunk-based development** approach with small, frequent commits directly to `main`. Each commit represents a working state of the application.

## Principles

1. **Small iterations** — Each commit introduces a single logical change
2. **Always shippable** — `main` should always compile and run
3. **Incremental delivery** — Features are built in thin vertical slices
4. **Refactor as needed** — Improve structure when patterns emerge, not preemptively

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

### Phase 4: TFM Reconstruction
Implement the Total Focusing Method algorithm with real-time performance and heatmap visualization.

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
