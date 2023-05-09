CC      = gcc
C++     = g++
WASMCC  = /home/lhw/wasi-sdk-14.0/bin/clang
WASMC++ = /home/lhw/wasi-sdk-14.0/bin/clang++
CFLAGS  = -Wall -g -O3
LDFLAGS =
WASMLDFLAGS = -L . -lpku
OBJ     = pku.o pkumalloc.o PKUInternal.o
WASMOBJ = pku-wasm.o pkumalloc-wasm.o PKUInternal-wasm.o
INCLUDE_PATH =
AR      = /home/lhw/wasi-sdk-14.0/bin/ar

all: libpku.a libnativepku.a main main.wasm

libpku.a: $(WASMOBJ)
	$(AR) crD $(@) $(WASMOBJ)

libnativepku.a: $(OBJ)
	ar crD $(@) $(OBJ)

main: main.o $(OBJ)
	$(CC) -o $(@) $(^) $(LDFLAGS)

# main.wasm: main-wasm.o
# 	$(WASMCC) --sysroot /home/lhw/wasi-sdk-14.0/share/wasi-sysroot -o $(@) $(<) $(WASMLDFLAGS)

main.wasm: main-wasm.o
	$(WASMCC) $(<) $(WASMLDFLAGS) -o $(@)

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

%.o: %.cpp
	$(C++) $(CFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

%-wasm.o: %.c
	$(WASMCC) $(CFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

%-wasm.o: %.cpp
	$(WASMC++) $(CFLAGS) $(INCLUDE_PATH) -c $(<) -o $(@)

clean:
	rm -f *.o libnativepku.a libpku.a main main.wasm