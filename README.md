# espresso-macros
Generally useful procedural macros.

This crate defines a few procedural macros. It is factored out into its own crate because procedural
macros cannot be defined in the crate where they are used. Check out
[the docs](espresso-macros.docs.espressosys.com) for details.

## Usage
Add to your `Cargo.toml`:
```
espresso-macros = { git = "https://github.com/EspressoSystems/espresso-macros.git" }
```

## Viewing documentation locally
```
cargo doc --open
```
