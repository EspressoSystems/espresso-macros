# zerok-macros
Generally useful procedural macros.

This crate defines a few procedural macros. It is factored out into its own crate because procedural
macros cannot be defined in the crate where they are used. Check out
[the docs](zerok-macros.docs.espressosys.com) for details.

## Usage
Add to your `Cargo.toml`:
```
zerok-macros = { git = "https://github.com/EspressoSystems/zerok-macros.git" }
```

## Viewing documentation locally
```
cargo doc --open
```
