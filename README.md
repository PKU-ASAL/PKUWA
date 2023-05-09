## PKUWA
---
PKUWA(Protection Key in Userspace for WebAssembly) is a framework to provide linear memory protection for WebAssembly.

## Getting Started
---
These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

## Prerequisites
---
We conducted the experiments in a Proxmox virtual machine with 8-core vCPU, 16GB memory, and Ubuntu 18.04 LTS with Linux kernel 4.15.

## Installation
---
1. Clone the [PKUWA](https://anonymous.4open.science/r/PKUWA-2321) GitHub repository to your local machine.
2. Install required dependencies by running the following command:
   ```sh
   sudo apt update && sudo apt install -y make build-essential bison clang linux-tools-common libssl-dev
   ```

## Run
PKUWA can be compiled with the below commands.
```sh
cd wasmtime
cargo build --release
```

To ensure everything woks, run examples:
```sh
cd examples/demo
../../wasmtime/target/release/wasmtime demo.wat
```
or
```sh
cd examples/democ
../../wasmtime/target/release/wasmtime main.wat
```

## License
This project is licensed under the MIT License.

## Acknowledgements
We would like to thank the anonymous reviewers for their valuable feedback and suggestions.