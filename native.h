#ifndef _NATIVE_H_
#define _NATIVE_H_

#include <string.h>
#include "arg.h"

/* ---- System configuration information --------------------------------- */

#define FFI_TYPE_VOID       0
#define FFI_TYPE_INT        1
#define FFI_TYPE_FLOAT      2
#define FFI_TYPE_DOUBLE     3
#if HAVE_LONG_DOUBLE
#define FFI_TYPE_LONGDOUBLE 4
#else
#define FFI_TYPE_LONGDOUBLE FFI_TYPE_DOUBLE
#endif
#define FFI_TYPE_UINT8      5
#define FFI_TYPE_SINT8      6
#define FFI_TYPE_UINT16     7
#define FFI_TYPE_SINT16     8
#define FFI_TYPE_UINT32     9
#define FFI_TYPE_SINT32     10
#define FFI_TYPE_UINT64     11
#define FFI_TYPE_SINT64     12
#define FFI_TYPE_STRUCT     13
#define FFI_TYPE_POINTER    14
#define FFI_TYPE_COMPLEX    15

/* This should always refer to the last type code (for sanity checks).  */
#define FFI_TYPE_LAST       FFI_TYPE_COMPLEX

/* ---- Generic type definitions ----------------------------------------- */

typedef unsigned long ffi_arg;
typedef signed long ffi_sarg;

typedef enum ffi_abi {
#if defined(X86_WIN64)
  FFI_FIRST_ABI = 0,
  FFI_WIN64,            /* sizeof(long double) == 8  - microsoft compilers */
  FFI_GNUW64,           /* sizeof(long double) == 16 - GNU compilers */
  FFI_LAST_ABI,
#ifdef __GNUC__
  FFI_DEFAULT_ABI = FFI_GNUW64
#else
  FFI_DEFAULT_ABI = FFI_WIN64
#endif

#elif defined(X86_64) || (defined (__x86_64__) && defined (X86_DARWIN))
  FFI_FIRST_ABI = 1,
  FFI_UNIX64,
  FFI_WIN64,
  FFI_EFI64 = FFI_WIN64,
  FFI_GNUW64,
  FFI_LAST_ABI,
  FFI_DEFAULT_ABI = FFI_UNIX64

#elif defined(X86_WIN32)
  FFI_FIRST_ABI = 0,
  FFI_SYSV      = 1,
  FFI_STDCALL   = 2,
  FFI_THISCALL  = 3,
  FFI_FASTCALL  = 4,
  FFI_MS_CDECL  = 5,
  FFI_PASCAL    = 6,
  FFI_REGISTER  = 7,
  FFI_LAST_ABI,
  FFI_DEFAULT_ABI = FFI_MS_CDECL
#else
  FFI_FIRST_ABI = 0,
  FFI_SYSV      = 1,
  FFI_THISCALL  = 3,
  FFI_FASTCALL  = 4,
  FFI_STDCALL   = 5,
  FFI_PASCAL    = 6,
  FFI_REGISTER  = 7,
  FFI_MS_CDECL  = 8,
  FFI_LAST_ABI,
  FFI_DEFAULT_ABI = FFI_SYSV
#endif
} ffi_abi;

#ifndef LIBFFI_ASM

#if defined(_MSC_VER) && !defined(__clang__)
#define __attribute__(X)
#endif

#include <stddef.h>
#include <limits.h>

/* LONG_LONG_MAX is not always defined (not if STRICT_ANSI, for example).
   But we can find it either under the correct ANSI name, or under GNU
   C's internal name.  */

#define FFI_64_BIT_MAX 9223372036854775807

#ifdef LONG_LONG_MAX
# define FFI_LONG_LONG_MAX LONG_LONG_MAX
#else
# ifdef LLONG_MAX
#  define FFI_LONG_LONG_MAX LLONG_MAX
#  ifdef _AIX52 /* or newer has C99 LLONG_MAX */
#   undef FFI_64_BIT_MAX
#   define FFI_64_BIT_MAX 9223372036854775807LL
#  endif /* _AIX52 or newer */
# else
#  ifdef __GNUC__
#   define FFI_LONG_LONG_MAX __LONG_LONG_MAX__
#  endif
#  ifdef _AIX /* AIX 5.1 and earlier have LONGLONG_MAX */
#   ifndef __PPC64__
#    if defined (__IBMC__) || defined (__IBMCPP__)
#     define FFI_LONG_LONG_MAX LONGLONG_MAX
#    endif
#   endif /* __PPC64__ */
#   undef  FFI_64_BIT_MAX
#   define FFI_64_BIT_MAX 9223372036854775807LL
#  endif
# endif
#endif

/* The closure code assumes that this works on pointers, i.e. a size_t
   can hold a pointer.  */

typedef struct _ffi_type
{
  size_t size;
  unsigned short alignment;
  unsigned short type;
  struct _ffi_type** elements;
} ffi_type;

#define FFI_API

/* The externally visible type declarations also need the MSVC DLL
   decorations, or they will not be exported from the object file.  */
#if defined LIBFFI_HIDE_BASIC_TYPES
# define FFI_EXTERN FFI_API
#else
# define FFI_EXTERN extern FFI_API
#endif

#ifndef LIBFFI_HIDE_BASIC_TYPES
#if SCHAR_MAX == 127
# define ffi_type_uchar                ffi_type_uint8
# define ffi_type_schar                ffi_type_sint8
#else
 #error "char size not supported"
#endif

#if SHRT_MAX == 32767
# define ffi_type_ushort       ffi_type_uint16
# define ffi_type_sshort       ffi_type_sint16
#elif SHRT_MAX == 2147483647
# define ffi_type_ushort       ffi_type_uint32
# define ffi_type_sshort       ffi_type_sint32
#else
 #error "short size not supported"
#endif

#if INT_MAX == 32767
# define ffi_type_uint         ffi_type_uint16
# define ffi_type_sint         ffi_type_sint16
#elif INT_MAX == 2147483647
# define ffi_type_uint         ffi_type_uint32
# define ffi_type_sint         ffi_type_sint32
#elif INT_MAX == 9223372036854775807
# define ffi_type_uint         ffi_type_uint64
# define ffi_type_sint         ffi_type_sint64
#else
 #error "int size not supported"
#endif

#if LONG_MAX == 2147483647
# if FFI_LONG_LONG_MAX != FFI_64_BIT_MAX
 #error "no 64-bit data type supported"
# endif
#elif LONG_MAX != FFI_64_BIT_MAX
 #error "long size not supported"
#endif

#if LONG_MAX == 2147483647
# define ffi_type_ulong        ffi_type_uint32
# define ffi_type_slong        ffi_type_sint32
#elif LONG_MAX == FFI_64_BIT_MAX
# define ffi_type_ulong        ffi_type_uint64
# define ffi_type_slong        ffi_type_sint64
#else
 #error "long size not supported"
#endif

#endif /* LIBFFI_HIDE_BASIC_TYPES */

typedef enum {
  FFI_OK = 0,
  FFI_BAD_TYPEDEF,
  FFI_BAD_ABI,
  FFI_BAD_ARGTYPE
} ffi_status;

typedef struct {
  ffi_abi abi;
  unsigned nargs;
  ffi_type** arg_types;
  ffi_type* rtype;
  unsigned bytes;
  unsigned flags;
#ifdef FFI_EXTRA_CIF_FIELDS
  FFI_EXTRA_CIF_FIELDS;
#endif
} ffi_cif;

/* ---- Definitions for the raw API -------------------------------------- */

#ifndef FFI_SIZEOF_ARG
# if LONG_MAX == 2147483647
#  define FFI_SIZEOF_ARG        4
# elif LONG_MAX == FFI_64_BIT_MAX
#  define FFI_SIZEOF_ARG        8
# endif
#endif

#ifndef FFI_SIZEOF_JAVA_RAW
#  define FFI_SIZEOF_JAVA_RAW FFI_SIZEOF_ARG
#endif

typedef union {
  ffi_sarg  sint;
  ffi_arg   uint;
  float	    flt;
  char      data[FFI_SIZEOF_ARG];
  void*     ptr;
} ffi_raw;

/* ---- Definitions for closures ----------------------------------------- */

#if FFI_CLOSURES

#ifdef _MSC_VER
__declspec(align(8))
#endif
typedef struct {
#if FFI_EXEC_TRAMPOLINE_TABLE
  void *trampoline_table;
  void *trampoline_table_entry;
#else
  union {
    char tramp[FFI_TRAMPOLINE_SIZE];
    void *ftramp;
  };
#endif
  ffi_cif   *cif;
  void     (*fun)(ffi_cif*,void*,void**,void*);
  void      *user_data;
#if defined(_MSC_VER) && defined(_M_IX86)
  void      *padding;
#endif
} ffi_closure
#ifdef __GNUC__
    __attribute__((aligned (8)))
#endif
    ;

#ifndef __GNUC__
# ifdef __sgi
#  pragma pack 0
# endif
#endif

#ifdef __sgi
# pragma pack 8
#endif
typedef struct {
#if FFI_EXEC_TRAMPOLINE_TABLE
  void *trampoline_table;
  void *trampoline_table_entry;
#else
  char tramp[FFI_TRAMPOLINE_SIZE];
#endif
  ffi_cif   *cif;

#if !FFI_NATIVE_RAW_API

  /* If this is enabled, then a raw closure has the same layout
     as a regular closure.  We use this to install an intermediate
     handler to do the translation, void** -> ffi_raw*.  */

  void     (*translate_args)(ffi_cif*,void*,void**,void*);
  void      *this_closure;

#endif

  void     (*fun)(ffi_cif*,void*,ffi_raw*,void*);
  void      *user_data;

} ffi_raw_closure;

#endif /* FFI_CLOSURES */

/* Convert between closure and function pointers.  */
#if defined(PA_LINUX) || defined(PA_HPUX)
#define FFI_FN(f) ((void (*)(void))((unsigned int)(f) | 2))
#define FFI_CL(f) ((void *)((unsigned int)(f) & ~3))
#else
#define FFI_FN(f) ((void (*)(void))f)
#define FFI_CL(f) ((void *)(f))
#endif

union ValueType
{
    bool    b;
    char    c;
    short   s;
    int     i;
    long    l;
    float   f;
    double  d;
};

/* ---- Public interface definition -------------------------------------- */

int SetArgValue(unsigned int ArgType, void** ArgValue, void* arg);
int MakeArgs(int NumArgs, unsigned int* ArgTypes, void** ArgValue, ...);
int SetValue(void* src, void* dst, unsigned int ValueType);
int ClearCallData(unsigned int NumArgs, void** ArgValue);

#define NATIVELIBRARYCALL(LibName, FuncName, ReturnType, ArgTypes, ReturnValue, ...) ({ \
    int NumArgs##FuncName = ARGC(__VA_ARGS__); \
    void** ArgValue##FuncName = NULL; \
    void* rc##FuncName = NULL; \
    if(NumArgs##FuncName > 0) \
    { \
        ArgValue##FuncName = malloc(sizeof(void*) * NumArgs##FuncName); \
        MakeArgs(NumArgs##FuncName, ArgTypes, ArgValue##FuncName, ##__VA_ARGS__); \
        SetArgValue(ReturnType, &rc##FuncName, (void*)(unsigned long long)ReturnValue); \
    } \
    int ret = NativeLibraryCall(LibName, strlen(LibName), #FuncName, strlen(#FuncName), NumArgs##FuncName, ReturnType, ArgTypes, ArgValue##FuncName, rc##FuncName); \
    SetValue(rc##FuncName, &ReturnValue, ReturnType); \
    ClearCallData(NumArgs##FuncName, ArgValue##FuncName); \
    ret; \
  })

#endif

#ifdef __cplusplus
extern "C" {
#endif

/* ---- Host interface -------------------------------------- */

#ifdef __wasm__
#define HOST_IFACE_FUNC
#else
// Avoid need to define any of these functions when compiling natively
#define HOST_IFACE_FUNC __attribute__((weak))
#endif

HOST_IFACE_FUNC
extern int NativeLibraryCall(const char* LibName, unsigned int LibLen, const char* FuncName, unsigned int FuncLen, unsigned int NumArgs, unsigned int ReturnType, unsigned int* ArgTypes, void* ReturnValue, void** ArgValue);

#ifdef __cplusplus
}
#endif

#endif