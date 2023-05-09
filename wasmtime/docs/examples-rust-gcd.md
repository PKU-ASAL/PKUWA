# Calculating the GCD

You can also [browse this source code online][code] and clone the wasmtime
repository to run the example locally.

[code]: https://github.com/bytecodealliance/wasmtime/blob/main/examples/gcd.rs

This example shows off how run a wasm program which calculates the GCD of two
numbers.

## `gcd.wat`

```wat
{{#include ../examples/gcd.wat}}
```


## `gcd.rs`

```rust,ignore
{{#include ../examples/gcd.rs}}
```
