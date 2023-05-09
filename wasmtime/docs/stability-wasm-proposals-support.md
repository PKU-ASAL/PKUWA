# WebAssembly Proposals Support

The following table summarizes Wasmtime's support for WebAssembly proposals as
well as the command line flag and [`wasmtime::Config`][config] method you can
use to enable or disable support for a proposal.

If a proposal is not listed, then it is not supported by Wasmtime.

Wasmtime will never enable a proposal by default unless it has reached phase 4
of [the WebAssembly standardizations process][phases] and its implementation in
Wasmtime has been [thoroughly
vetted](./contributing-implementing-wasm-proposals.html).

| WebAssembly Proposal                        | Supported in Wasmtime?           | Command Line Name  | [`Config`][config] Method |
|---------------------------------------------|----------------------------------|--------------------|---------------------------|
| **[Import and Export Mutable Globals]**     | **Yes.**<br/>Always enabled.     | (none)             | (none)                    |
| **[Sign-Extension Operations]**             | **Yes.**<br/>Always enabled.     | (none)             | (none)                    |
| **[Non-Trapping Float-to-Int Conversions]** | **Yes.**<br/>Always enabled.     | (none)             | (none)                    |
| **[Multi-Value]**                           | **Yes.**<br/>Enabled by default. | `multi-value`      | [`wasm_multi_value`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_multi_value) |
| **[Bulk Memory Operations]**                | **Yes.**<br/>Enabled by default. | `bulk-memory`      | [`wasm_bulk_memory`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_bulk_memory) |
| **[Reference Types]**                       | **Yes.**<br/>Enabled by default. | `reference-types`  | [`wasm_reference_types`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_reference_types) |
| **[Fixed-Width SIMD]**                      | **Yes.**<br/>Enabled by default. | `simd`             | [`wasm_simd`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_simd) |
| **[Threads and Atomics]**                   | **In progress.**                 | `threads`          | [`wasm_threads`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_threads) |
| **[Multi-Memory]**                          | **Yes.**                         | `multi-memory`     | [`wasm_multi_memory`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_multi_memory) |
| **[Component Model]**                       | **In progress.**                 | `component-model`  | [`wasm_component_model`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_component_model) |
| **[Memory64]**                              | **Yes.**                         | `memory64`         | [`wasm_memory64`](https://docs.rs/wasmtime/*/wasmtime/struct.Config.html#method.wasm_memory64) |

The "Command Line Name" refers to the `--wasm-features` CLI argument of the
`wasmtime` executable and the name which must be passed to enable it.

[config]: https://docs.rs/wasmtime/*/wasmtime/struct.Config.html
[Multi-Value]: https://github.com/WebAssembly/spec/blob/master/proposals/multi-value/Overview.md
[Bulk Memory Operations]: https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md
[Import and Export Mutable Globals]: https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md
[Reference Types]: https://github.com/WebAssembly/reference-types/blob/master/proposals/reference-types/Overview.md
[Non-Trapping Float-to-Int Conversions]: https://github.com/WebAssembly/spec/blob/master/proposals/nontrapping-float-to-int-conversion/Overview.md
[Sign-Extension Operations]: https://github.com/WebAssembly/spec/blob/master/proposals/sign-extension-ops/Overview.md
[Fixed-Width SIMD]: https://github.com/WebAssembly/simd/blob/master/proposals/simd/SIMD.md
[phases]: https://github.com/WebAssembly/meetings/blob/master/process/phases.md
[Threads and Atomics]: https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md
[Multi-Memory]: https://github.com/WebAssembly/multi-memory/blob/master/proposals/multi-memory/Overview.md
[Component Model]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md
[Memory64]: https://github.com/WebAssembly/memory64/blob/master/proposals/memory64/Overview.md
