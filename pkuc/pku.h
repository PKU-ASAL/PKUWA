#ifndef _PKU_H_
#define _PKU_H_

#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <malloc.h>
#include <limits.h>
#include "pkumalloc.h"
#include <stdio.h>

#define WORDSIZE 8
#define PAGESIZEPKU 4096
#define PK_NUM_KEYS 16

typedef uint16_t pkey_t;
typedef uint64_t pkru_config_t;

#define PKUIsPkeyLoaded(pkey, pkru) ({ \
    const pkru_config_t mask = (pkru_config_t)(PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE) << (pkey*2); \
    bool _ret = ((pkru & mask) != mask); \
    _ret; \
})

#define SIZE_T_ONE ((size_t)1)
#define PAGE_ALIGN(S)\
    (((S) + (PAGESIZEPKU - SIZE_T_ONE)) & ~(PAGESIZEPKU - SIZE_T_ONE))

#define NUM_DOMAINS 16
#define NUM_REGISTERED_PKUCALLS 64

#ifndef PKEY_DISABLE_ACCESS
#define PKEY_DISABLE_ACCESS (0x1)
#endif

#ifndef PKEY_DISABLE_WRITE
#define PKEY_DISABLE_WRITE (0x2)
#endif

typedef void (*pFunc)(void);

#define PKUCALL(...) ({ \
        int did = GetCurrentDid(); \
        pkucall_##__VA_ARGS__; \
        size_t ret = __VA_ARGS__; \
        PKURestore(did); \
        ret; \
    })

#define PKU_CALL_REGISTER(did, name) ({ pkucall_id_##name = RegisterPKUCall(did, (pFunc)name); pkucall_id_##name; })
#define WASICALL(ptr, size) {getentropy(ptr, size)}

#define PK_DOMAIN_ROOT 1
#define PK_DEFAULT_KEY 0

typedef int vkey_t;
#define VKEY_MAX INT_MAX
#define VKEY_INVALID                -1
#define PKEY_INVALID       ((pkey_t)-1)

#define GENPKU(name, ...) \
    static int pkucall_id_##name = 0; \
    \
    void pkucall_##name(__VA_ARGS__) { \
        PKUSwitch(pkucall_id_##name); \
    } \
    \

#define GENMPK(name, return_type, ...) \
    static int pkucall_id_##name = 0; \
    \
    return_type __attribute__((naked)) pkucall_##name(__VA_ARGS__) { \
        PKUSwitch(pkucall_id_##name); \
    } \
    \

#ifdef __cplusplus
extern "C"
{
#endif

extern size_t g_MallocNumber;
extern size_t g_FreeNumber;
extern size_t g_ExtraMemory;

int DomainProtect(void* addr, size_t length, unsigned int pkey);

void* NaiveMmap(size_t bytes);

/**
 * @brief Map a memory range
 *
 * This function maps a memory range via a call to @c mmap in WASI.
 * In addition, it protects the mapped range with the domain's default
 *
 * @param did
 *        The domain to which the memory range shall be assigned.
 * @param addr
 *        see @c mmap
 * @param length
 *        see @c mmap
 * @param prot
 *        see @c mmap
 * @param flags
 *        see @c mmap
 * @param fd
 *        see @c mmap
 * @param offset
 *        see @c mmap
 * @return
 *        0 on success, or -1 on error, and errno is set according to
 *        @c mmap, or @c pk_pkey_mprotect.
 */
void* PKUMmap(void* addr, size_t length, int prot, int flags, int fd, int offset);

#ifdef __cplusplus
}
#endif

static void* MallocHook(size_t bytes)
{
    void* ptr = NULL;
    size_t size = bytes;
    if(g_MallocNumber < 0)
    {
        #ifdef __WASI_MMAP__
        if(GetCurrentDid() != 0)
        {
            size = PAGE_ALIGN(bytes);
            ptr = NaiveMmap(size);
            // ptr = PKUMmap(NULL, size, 3, 0x2 | 0x20, -1, 0);
            // DomainProtect(ptr, size, GetCurrentDid());
        }
        else
        {
            ptr = NaiveMmap(size);
        }
        #else
        ptr = PKUMalloc(size);
        #endif
    }
    else
    {
        ptr = PKUMalloc(size);
    }
    return ptr;
}

static void FreeHook(void* ptr)
{
    if(GetCurrentDid() != 0)
    {
        DomainProtect(ptr, PAGESIZEPKU, 0);
    }
    if(g_FreeNumber < 0)
    {
        #ifdef __WASI_MMAP__
        PKUFree(ptr);
        #else
        PKUFree(ptr);
        #endif
    }
    else
    {
        PKUFree(ptr);
    }
}

#ifdef __cplusplus
extern "C"
{
#endif

/* Struct for maintaining a domain's protection keys */
typedef struct PKUKey
{
    pkey_t pkey;
    unsigned int perm; // Key permissions. Bitwise OR of one or more of the following flags: PKEY_DISABLE_ACCESS, PKEY_DISABLE_WRITE
    bool used;         // Is key slot used
} PKUKey;

/* Struct for registered ecalls */
typedef struct PKUCall
{
    int did;        // PKU registered for this domain
    pFunc entry;   // PKU call entry point
} PKUCall;

/**
 * @brief Initialize PKU
 *
 * This function initializes PKU. It needs to be called
 * at the beginning of your program.
 *
 * @param flags
 *        0
 * @return
 *        0, or -1 on error, and errno is set to:
 *        @c EACCES
 *            Todo: specify
 */
int PKUInit(int flags);

/**
 * @brief Deinitialize PKU
 *
 * This function deinitializes PKU. For this to work, all domains except
 * the root domain have to be freed first via @c PKUDomainFree.
 *
 * @return
 *        0 on success, or -1 on error, and errno is set to:
 *        @c EACCES
 *            Todo: specify
 */
int PKUDeinit(void);

/**
 * @brief Create a new domain
 *
 * This function creates a new protection domain with its own private
 * protection key.
 *
 * @param flags
 *         0
 *            Protection key of new domain is guaranteed to be unique
 *         @c PKU_KEY_SHARED
 *            Protection key of new domain can be shared. This allows
 *            to allocate a higher number of protection keys than the
 *            architecture natively supports. However, it only gives
 *            probabilistic isolation guarantees.
 *         @c PKU_KEY_INHERIT
 *            Protection key of new domain is inherited to the calling
 *            domain as if the new domain executed @c PKUDomainAssignPkey(
 *                did, vkey, 0 [|PKU_KEY_COPY] [|PKU_KEY_OWNER], 0)
 *         @c PKU_KEY_COPY
 *            Only valid with @c PKU_KEY_INHERIT
 *         @c PKU_KEY_OWNER
 *            Only valid with @c PKU_KEY_INHERIT
 * @return
 *        The new domain id @c did, which is always positive,
 *        or -1 on error, and errno is set to:
 *        @c EINVAL
 *            if @p flags are invalid.
 *        @c ENOMEM
 *            if there is no more space for new domains
 */
int PKUCreateDomain(unsigned int flags);

/**
 * @brief Frees a domain
 *
 * This function cleans up a protection domain.
 * It requires any allocated protection keys to be freed.
 *
 * @param domain
 *        The domain to free.
 * @return
 *        0 on success, or -1 on error, and errno is set to:
 *        @c EACCES
 *            if @p domain is not a child domain
 *        @c EPERM
 *            if @p domain has allocated protection keys that are not yet
 *            freed
 */
int PKUDomainFree(int domain);

/**
 * @brief Allocate a new protection key.
 *
 * This function allocates a new protection key via @c pkey_alloc in WASI.
 * In addition, it assigns the allocated protection key to the
 * current protection domain. The protection key might be transferred to
 * other domains via @a PKUDomainAssignPkey.
 *
 * @param flags
 *         0
 *            Allocated protection key is guaranteed to be unique
 *         @c PKU_KEY_SHARED
 *            Allocated protection key can be shared across different calls to
 *            @c PKUPkeyAlloc. This allows to allocate a higher number
 *            of protection keys than the architecture natively
 *            supports. However, it only gives probabilistic isolation
 *            guarantees.
 * @param AccessRights
 *         may contain zero or more disable operations:
 *         @c PKEY_DISABLE_ACCESS
 *            Disables both read and write access. Code fetces might
 *            still be allowed, depending on the architecture.
 *         @c PKEY_DISABLE_WRITE
 *            Disables write access.
 * @return
 *        The allocated protection key, which is always positive,
 *        or -1 on error, and errno is set to:
 *        @c EINVAL
 *            if @p flags or @p AccessRights is invalid.
 *        @c ENOSPC
 *            if no more free protection keys are available, or the
 *            architecture does not support protection keys.
 */
int PKUPkeyAlloc(unsigned int flags, unsigned int AccessRights);

/**
 * @brief Free a protection key
 *
 * This function frees a protection key.
 *
 * The protection key has to be allocated via @a PKUPkeyAlloc. A domain
 * cannot free its default key. Moreover, the current domain needs
 * to have ownership. A domain has ownership either by executing
 * @a PKUPkeyAlloc itself and not delegating ownership, or it was
 * granted ownership via @a PKUDomainAssignPkey and the
 * @c PK_KEY_OWNER flag.
 *
 * The @p pkey is freed for all domains that may hold a copy via
 * @a PKUDomainAssignPkey with the @c PKU_KEY_COPY flag.
 *
 * @return
 *        0 on success, or -1 on error, and errno is set to:
 *        @c EACCES
 *            if current domain does not own @p pkey.
 *        @c EPERM
 *            if key is still in use
 */
int PKUPkeyFree(int pkey);

/**
 * @brief Assign a protection key to a domain
 *
 * This function assigns a protection key to a domain. The key has to
 * be allocated via @a PKUPkeyAlloc, and the current domain needs to
 * have proper access to it. A domain can assign a protection key if
 * it has executed @a PKUPkeyAlloc itself and did not transfer ownership,
 * or it was granted access to the key via @a PKUDomainAssignPkey without
 * the @c PKU_KEY_COPY flag.
 *
 * A domain can also assign a protection key to itself, in which case
 * the original key will be lost. E.g. a domain can drop @p AccessRights
 * onto a @p pkey while keeping ownership via @c PKU_KEY_OWNER, or losing
 * ownership via @c PKU_KEY_COPY.
 *
 * @param did
 *        The domain to assign the protection key to.
 * @param pkey
 *        The protection key
 * @param flags
 *        A bitwise combination by any of those
 *        @c PKU_KEY_OWNER
 *            The new domain gets ownership, allowing it to use @p pkey
 *            for memory mapping (@p PKUMmap, @p PKUMunmap, @p PKUMprotect,
 *            @p PKUPkeyMprotect) and free it via @p PKUPkeyFree.
 *        @c PKU_KEY_COPY
 *            The new domain gets a copy of @p pkey which it can use for
 *            accessing memory assigned to @p pkey, or making other
 *            copies with the @c PKU_KEY_COPY flag. Depending on
 *            @c PKU_KEY_OWNER, this copy has ownership access. Without
 *            this flag, the current domain loses access to @p pkey.
 * @param AccessRights
 *        The access rights for @p pkey. They must be equal or more
 *        restrictive than the current domain's access rights for this
 *        @p pkey.
 * @return
 *        0 on success, or -1 on error, and errno is set to:
 *        @c EINVAL
 *            if @p did, @p flags or @p AccessRights is invalid.
 *        @c EACCES
 *            if the current domain does not own @p pkey, or
 *            if @p AccessRights is more permissive than current
 *            domain's access rights
 *        @c ENOMEM
 *            if there is no more space for assigning @p pkeys
 */
int PKUDomainAssignPkey(int did, int pkey, int flags, int AccessRights);

/**
 * @brief Protect a memory range with a protection key
 *
 * This function protects a memory range via @c pkey_mprotect in WASI. In addition,
 * the current domain needs to have access to @p pkey, and the requested
 * protection needs to be allowed by this @p pkey.
 *
 * @param did
 *        The domain to which the memory range belongs.
 * @param addr
 *        The page-aligned address pointing to the start of the memory
 *        range to protect.
 * @param len
 *        The length in bytes of the memory range to protect.
 * @param prot
 *        Any combination of @c PROT_NONE, @c PROT_READ, @c PROT_WRITE,
 *        @c PROT_EXEC, etc. allowed by @c pkey_mprotect in WASI.
 * @param pkey
 *        The protection key.
 * @return
 *        0 on success, or -1 on error, and errno is set according to @c pkey_mprotect in WASI
 *        @c EINVAL
 *            if @p pkey is invalid.
 *        @c EACCES
 *            if @p did is not current, or
 *            if current domain does not own @p pkey, or
 *            if the address range is owned by a different @p pkey
 *        @c ENOMEM
 *            if there is no more space for keeping track of mprotect calls
 *            any other error code specified for @c pkey_mprotect in WASI.
 */
int PKUPkeyMprotect(void* addr, size_t len, int prot, int pkey);

/**
 * @brief Unmap a memory range
 *
 * This function unmaps a memory range via a call to @c munmap in WASI.
 * The memory has to be owned by @p did.
 *
 * @param did
 *        The domain from which the memory range shall be freed.
 * @param addr
 *        see @c munmap in WASI
 * @param len
 *        see @c munmap in WASI
 * @return
 *        0 on success, or -1 on error, and errno is set according to
 *        @c munmap in WASI
 */
int PKUMunmap(void* addr, size_t len);

/**
 * @brief Protect a memory range
 *
 * This function protects a memory range via a call to @c mprotect in WASI.
 * While this function does not modify protection keys, it verifies
 * that the memory range's protection keys are owned by the calling
 * domain.
 *
 * @param did
 *        The domain from which the memory range shall be protected.
 * @param addr
 *        see @c mprotect in WASI
 * @param len
 *        see @c mprotect in WASI
 * @param prot
 *        see @c mprotect in WASI
 * @return
 *        0 on success, or -1 on error, and errno is set according to
 *        @c mprotect in WASI
 */
int PKUMprotect(void* addr, size_t len, int prot);

/**
 * @brief Register a new pkucall.
 *
 * This function registers a new pkucall with which another domain can
 * call the specified domain.
 *
 * @param did
 *        The domain for which a new pkucall shall be registered.
 * @param entry
 *        The entry point of the pkucall
 * @return
 *        the positive pkucall_id on success, or -1 on error, and errno is set to:
 *        @c EACCES
 *            if @p did is neither the current domain.
 *        @c EINVAL
 *            if @p entry does not point into the memory of @p did, or
 *            @p pkucall_id is invalid.
 *        @c ENOMEM
 *            if there is no more space for registering pkucalls
 */
int RegisterPKUCall(int did, pFunc entry);

/**
 * @brief Permit other domains to invoke pkucalls.
 *
 * This function permits other domains to invoke pkucalls of the specified
 * domain.
 *
 * @param did
 *        The domain to which pkucalls are permitted.
 * @param CallerDid
 *        The domain which is permitted to invoke pkucalls of @p did.
 * @param flags
 *        Optional flags for future use. Must be 0 in current implementations.
 * @return
 *        0 on success, or -1 on error, and errno is set to:
 *        @c EACCES
 *            if @p did is neither the current domain.
 *        @c EINVAL
 *            if @p CallerDid or @p flags are invalid.
 *        @c ENOMEM
 *            if there is no more space for new callers
 */
int PKUDomainAllowCaller(int CallerDid, unsigned int flags);

int EnableSectionGuardPage(int did);

int PKUSwitch(int PKUCallID);
int PKURestore(int did);

int ReadPKRU();

size_t GetMemorySize();

#ifdef __cplusplus
}
#endif

#define malloc(bytes) MallocHook(bytes)
#define free(ptr) FreeHook(ptr)

#endif