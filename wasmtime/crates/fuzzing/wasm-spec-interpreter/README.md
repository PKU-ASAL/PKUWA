wasm-spec-interpreter
=====================

This project shows how to use `ocaml-interop` to call into the Wasm spec
interpreter. There are several steps to making this work:
 - building the OCaml Wasm spec interpreter as a static library
 - building a Rust-to-OCaml FFI bridge using `ocaml-interop` and a custom OCaml
   wrapper
 - linking both things into a Rust crate

### Dependencies

This crate only builds in an environment with:
- `make` (the Wasm spec interpreter uses a `Makefile`)
- `ocamlopt`, `ocamlbuild` (available with, e.g., `dnf install ocaml`)
- Linux tools (e.g. `ar`); currently it is easiest to build the static
  libraries in a single environment but this could be fixed in the future (TODO)
- `libgmp`, for the OCaml `zarith` package
- `git` is used by `build.rs` to retrieve the repository containing the Wasm
  spec interpreter; it is safe to completely remove `ocaml/spec` to get a new
  copy

### Build

```
cargo build --features build-libinterpret
```

Use `FFI_LIB_DIR=path/to/lib/...` to specify a different location for the static
library (this is mainly for debugging). If the `build-libinterpret` feature is
not provided, this crate will build successfully but fail at runtime.

### Test

```
cargo test --features build-libinterpret
```
