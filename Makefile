# CC      = gcc
# C++     = g++
CC      = /usr/bin/clang-15
C++     = /usr/bin/clang-15++
WASMCC  = /home/lhw/wasi-sdk-14.0/bin/clang
WASMC++ = /home/lhw/wasi-sdk-14.0/bin/clang++
MUSLCC = /home/lhw/wasmpku/libpku/musl/bin/musl-gcc
CFLAGS  = -Wall -g -O0
WASMCFLAGS = -Wall -g -O0 -D_PKU_WASM
LDFLAGS = -L . -lnativepku -Wl,-rpath=.
# LDFLAGS = -L . -lnativepku -Wl,-rpath=. -Wl,--defsym,malloc=PKUMalloc,--defsym,free=PKUFree
WASMLDFLAGS = -L . -lpku
OBJ     = pku.o pkumalloc.o PKUInternal.o libchook.o
WASMOBJ = pku-wasm.o pkumalloc-wasm.o PKUInternal-wasm.o libchook-wasm.o
SHARDOBJ = pkulibc.o
INCLUDE_PATH =
AR      = /home/lhw/wasi-sdk-14.0/bin/ar
SYSROOT = --sysroot /home/lhw/wasi-sdk-14.0/share/wasi-sysroot

all: libpku.a libnativepku.a libpkulibc.so main.wasm main

libpku.a: $(WASMOBJ)
	$(AR) crD $(@) $(WASMOBJ)

libnativepku.a: $(OBJ)
	ar crD $(@) $(OBJ)

libpkulibc.so: $(SHARDOBJ)
	./getmusl.sh $(shell pwd)
	$(MUSLCC) -shared -fPIC $(^) -o $(@) -static-libgcc -Wl,-Bstatic -lc

main: main.o $(OBJ)
	$(CC) -o $(@) $(^) $(LDFLAGS)

main.wasm: main-wasm.o $(WASMOBJ)
	cp ./libpku.imports /home/lhw/wasi-sdk-14.0/share/wasi-sysroot/lib/wasm32-wasi
	cp ./libpku.imports /home/lhw/test2/wasi-libc/sysroot/lib/wasm64-wasi
	cp ./libpku.imports /home/lhw/test2/wasix-libc/sysroot32/lib/wasm32-wasi
	$(WASMCC) $(SYSROOT) $(<) $(WASMLDFLAGS) -o $(@)
# $(WASMCC) --target=wasm64-wasi --sysroot /home/lhw/test2/wasi-libc/sysroot $(<) $(WASMLDFLAGS) -o $(@)

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

%.o: %.cpp
	$(C++) $(CFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

%-wasm.o: %.c
	$(WASMCC) $(SYSROOT) $(WASMCFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)
# $(WASMCC) --target=wasm64-wasi --sysroot /home/lhw/test2/wasi-libc/sysroot $(WASMCFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

%-wasm.o: %.cpp
	$(WASMC++) $(WASMCFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

clean:
	rm -f *.o libpku.a libnativepku.a main main.wasm libpkulibc.so