/home/wce/wasm_tools/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04/bin/clang-16 -Wno-implicit-function-declaration --target=wasm32-wasi --sysroot=/home/wce/wasm_tools/wasi-sdk-24.0-x86_64-linux/share/wasi-sysroot hello.c -o hello.wasm
/home/wce/wabt/bin/wasm2wat hello.wasm -o hello.wat
/home/wce/wasm_tools/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04/bin/clang++   -fno-exceptions   -Wno-implicit-function-declaration   --target=wasm32-wasi   --sysroot=/home/wce/wasm_tools/wasi-sdk-24.0-x86_64-linux/share/wasi-sysroot   -D_WASI_EMULATED_PROCESS_CLOCKS   -lwasi-emulated-process-clocks  /home/wce/Benchmarks/test-suite/SingleSource/Benchmarks/Shootout-C++/fibo.cpp -o fibo.wasm
export http_proxy=http://127.0.0.1:7890; export https_proxy=http://127.0.0.1:7890;
unset http_proxy; unset https_proxy;
