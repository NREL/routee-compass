# instructions.md

This file provides guidance to AI agents when working with code in this repository.

## Project Overview

**RouteE Compass** is an energy-aware routing engine for the RouteE ecosystem with:
- Dynamic and extensible search objectives blending distance, time, cost, and energy (via RouteE Powertrain)
- Core engine written in Rust for high performance and parallel query execution
- Rust, HTTP, and Python APIs for integration into research pipelines and other software

## Repository Structure

```
docs/                         # Jupyter Book documentation
python/                       # Python wrapper code
  nrel/routee/compass/        # Main Python package source
rust/                         # Core Rust implementation
  routee-compass/             # Main application and binary crate
  routee-compass-core/        # Core shared logic for application 
  routee-compass-macros/      # Macros for the project 
  routee-compass-powertrain/  # Extension to include RouteE Powertrain Energy 
  routee-compass-py/          # Rust-Python bindings using PyO3
```

## Architecture Overview

### Plugin System via `inventory` Crate
RouteE Compass uses a compile-time plugin registration system:
- Plugins register themselves using `inventory::submit!` macro at compile time
- See `rust/routee-compass/src/app/compass/compass_builder_inventory.rs` for core plugin registration
- Builder types: `TraversalModelBuilder`, `ConstraintModelBuilder`, `LabelModelBuilder`, `InputPluginBuilder`, `OutputPluginBuilder`
- Example plugin registration:
```rust
inventory::submit! {
    BuilderRegistration(|builder| {
        builder.add_traversal_model("distance".to_string(), Rc::new(DistanceTraversalBuilder {}));
        // ... more registrations
    })
}
```

### Builder → Service → Model Pattern
Core architecture follows a three-stage pattern:
1. **Builder**: Reads config TOML, constructs service from config
2. **Service**: Immutable service object, typically `Arc<dyn XxxService>`
3. **Model**: Per-query stateful instance created by service

Example: `TraversalModelBuilder::build()` → `TraversalModelService` → `TraversalModel` instance per query

### Configuration System
- Apps configured via TOML files (see `docs/config.md`)
- Main sections: `[system]`, `[algorithm]`, `[graph]`, `[mapping]`, `[search.traversal]`, `[search.constraint]`
- Traversal models can be `combined` with multiple `[[search.traversal.models]]` entries
- Config parsing uses `config` crate with custom extensions in `routee-compass-core/src/config/`

### Query Format
JSON queries with structure:
```json
{
  "origin_x": -104.969307,
  "origin_y": 39.749512,
  "destination_x": -104.975360,
  "destination_y": 39.752155
}
```

## Development Workflows

### Rust Development
```bash
cd rust/

# Build
cargo build --release

# Run tests (from rust/ directory)
cargo test --workspace

# Format check
cargo fmt --all -- --check

# Run benchmarks
cargo bench

# Build docs
cargo doc --no-deps --open
```

### Python Development
```bash
# Setup (requires conda environment)
conda activate routee-compass
maturin develop  # from repo root

# Run tests
pytest python/tests

# Install with dev dependencies
pip install -e ".[dev]"
```

### Documentation
```bash
# Build Jupyter Book docs
python docs/examples/_convert_examples_to_notebooks.py
jupyter-book build docs/
# View at docs/_build/html/index.html
```

## Code Style & Conventions

### Rust-Specific Rules (see `docs/developers/rust_code_style.md`)
- **NEVER use** `.unwrap()`, `panic!()`, or `.expect()` except in tests
- Prefer returning `Result<T, E>` or `Option<T>` for error handling
- Use `thiserror` crate for error types, NOT `anyhow` (except simple CLI scripts)
- Prefer `&T` (reference) over `T` (value) in function parameters (99% of cases)
- Prefer `Box<[T]>` over `Vec<T>` for large, immutable collections
- Write tests in `#[cfg(test)] mod test {}` at bottom of files
- Use Rustdoc comments (`///`) for public APIs - these generate docs.rs documentation

### Error Handling Pattern
```rust
// Good - returns Result
fn process_data(data: &Data) -> Result<Output, MyError> {
    // ...
}

// Bad - panics on error
fn process_data(data: &Data) -> Output {
    // ... .unwrap() calls ...
}
```

### Testing Patterns
- Unit tests in `#[cfg(test)]` modules at file bottom
- Integration tests use test fixtures in crate test directories
- Python tests in `python/tests/` using pytest

## Key Integration Points

### Python ↔ Rust Bridge
- PyO3 bindings in `rust/routee-compass-py/`
- `CompassAppWrapper` wraps Rust `CompassApp` for Python
- Python package structure: `nrel.routee.compass`
- Module name in pyproject.toml: `nrel.routee.compass.routee_compass_py`

### RouteE Powertrain Integration
- Energy modeling via `routee-compass-powertrain` crate
- Registers "energy" traversal model builder
- Uses smartcore for ML models and ONNX for neural networks

## Package Management

### Rust
- Workspace-level dependencies in `rust/Cargo.toml`
- Member crates share dependency versions via `[workspace.dependencies]`
- Key dependencies: `serde`, `rayon` (parallelism), `geo` (geometry), `smartcore` (ML)

### Python
- Built with `maturin` (Rust→Python build tool)
- Package name: `nrel.routee.compass` (note: underscores in package, dots in import)
- Optional dependencies: `[dev]`, `[osm]`, `[all]`
- Uses `pixi` for conda environment management

## Common Pitfalls

1. **Working directory for Rust tests**: Must run `cargo test` from `rust/` directory
2. **Python development build**: Run `maturin develop` from repo root, not `rust/` directory
3. **Config file paths**: Relative paths in TOML configs are relative to config file location
4. **Python import**: Use `from nrel.routee.compass import CompassApp`, not `import nrel_routee_compass`
5. **Conda activation**: Always activate `routee-compass` conda env before Python work
