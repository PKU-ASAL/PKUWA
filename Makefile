# CC      = gcc
# C++     = g++
# CC      = /home/wce/wasm_tools/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04/bin/clang
# C++     = /home/wce/wasm_tools/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04/bin/clang++
WASMCC  = /home/wce/wasm_tools/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04/bin/clang
WASMC++ = /home/wce/wasm_tools/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04/bin/clang++
CFLAGS  = -Wall -g -O0
WASMCFLAGS = -Wall -g -O0 -D_PKU_WASM --target=wasm32-wasi
WASMLDFLAGS = -L .
INCLUDE_PATH =
AR      = /home/wce/wasm_tools/wasi-sdk-24.0-x86_64-linux/bin/ar
SYSROOT = --sysroot /home/wce/wasm_tools/wasi-sdk-24.0-x86_64-linux/share/wasi-sysroot

all: main.wasm main2.wasm
# main

# main: main.o $(OBJ)
# 	$(CC) -o $(@) $(^) $(LDFLAGS)

main.wasm: main-wasm.o $(WASMOBJ)
# 	cp ./libpku.imports /home/wce/wasm_tools/wasi-sdk-24.0-x86_64-linux/share/wasi-sysroot
# 	cp ./libpku.imports /home/wce/wasm_tools/wasi-libc/sysroot/lib/wasm32-wasi
	$(WASMCC) --target=wasm32-wasi $(SYSROOT) $(<) $(WASMLDFLAGS) -o $(@)
# $(WASMCC) --target=wasm64-wasi --sysroot /home/lhw/test2/wasi-libc/sysroot $(<) $(WASMLDFLAGS) -o $(@)

main2.wasm: main2-wasm.o $(WASMOBJ)
# 	cp ./libpku.imports /home/wce/wasm_tools/wasi-sdk-24.0-x86_64-linux/share/wasi-sysroot
	$(WASMCC) --target=wasm32-wasi $(SYSROOT) $(<) $(WASMLDFLAGS) -o $(@)
	
%-wasm.o: %.c
	$(WASMCC) $(SYSROOT) $(WASMCFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

clean:
	rm -f *.o libpku.a libnativepku.a main main.wasm libpkulibc.so