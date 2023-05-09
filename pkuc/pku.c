#include <stdio.h>
#include <unistd.h>
#include <errno.h>
#include "pku.h"

size_t g_MallocNumber = 0;
size_t g_FreeNumber = 0;
size_t g_ExtraMemory = 0;

PKUKey keys[NUM_DOMAINS];
PKUCall RegisteredPKUCalls[NUM_REGISTERED_PKUCALLS];

__attribute__((constructor)) void PKUInitCtor()
{
    keys[0].perm = 0;
    keys[0].pkey = 0;
    keys[0].used = 1;
}

unsigned char g_initialized = 0;

typedef struct s_mprotect
{
    void*        addr;
    size_t       len;
    int          prot;
    PKUKey       pkey;
    bool         used;
    const char*  name;
    int          mmap_flags;
    int          mmap_fd;
} s_mprotect;

#define NUM_MPROTECT_RANGES 4096

typedef struct PKUData
{
    int        initialized;
    size_t     stacksize;
    void       (*UserHandler)(void*);
    PKUKey     domains[NUM_DOMAINS];
    s_mprotect ranges[NUM_MPROTECT_RANGES];
    size_t     ranges_max_used;
} PKUData;

PKUData g_data = {0,};

static bool g_LazyFree = false;

static bool inline DomainExists(int did)
{
    return (did >= 0 && did < NUM_DOMAINS && keys[did].used);
}

int DoInit(int flags)
{
    if(g_data.initialized)
    {
        printf("DoInit: PKU already initialized\n");
        errno = EACCES;
        goto error;
    }

    // verify page size
    long int pagesize = sysconf(_SC_PAGESIZE);
    if(-1 == pagesize)
    {
        printf("DoInit: sysconf(_SC_PAGESIZE) failed\n");
        errno = EACCES;
        goto error;
    }
    if(PAGESIZEPKU != pagesize)
    {
        printf("DoInit: pagesize does not match. It should be %d but it is %ld", PAGESIZEPKU, pagesize);
        errno = EACCES;
        goto error;
    }

    g_data.initialized = 1;
    return 0;
error:
    printf("DoInit error\n");
    return -1;
}

int PKUInit(int flags)
{
    int ret = -1;
    int DoInitFinished = 0;

    if(g_initialized)
    {
        ret = 0;
        return ret;
    }

    ret = DoInit(flags);
    if (-1 == ret) {
        goto error;
    }
    DoInitFinished = 1;
    g_initialized = 1;
    return ret;

error:
    if(DoInitFinished) 
    {
        if(PKUDeinit() != 0)
        {
            printf("PKUDeinit failed\n");
        }
    }
    return ret;
}

int PKUDeinit(void)
{
    return 0;
}

int PKUDomainFree(int domain)
{
    if(g_data.initialized == 0)
    {
        printf("Not initialized\n");
        return -1;
    }

    if(!DomainExists(domain))
    {
        printf("Invalid domain\n");
        errno = EINVAL;
        return -1;
    }

    for(size_t did = 0; did < NUM_DOMAINS; ++did)
    {
        if(g_data.domains[did].used)
        {
            printf("cannot free domains\n");
            errno = EINVAL;
            return -1;
        }
    }

    for(size_t rid = 0; rid < NUM_MPROTECT_RANGES; ++rid)
    {
        if(g_data.ranges[rid].used)
        {
            g_data.UserHandler(g_data.ranges[rid].addr);
        }
    }

    PKUKey* dom = &(g_data.domains[domain]);
    dom->pkey = 0;
    dom->perm = 0;
    dom->used = 0;

    return 0;
}

int PKUPkeyAlloc(unsigned int flags, unsigned int AccessRights)
{
    if(AccessRights & ~(unsigned int)(PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE))
    {
        printf("PKUPkeyAlloc invalid flags or access rights\n");
        errno = EINVAL;
        return -1;
    }

    int pka = PKUCreateDomain(flags);
    return pka;
}

int PKUPkeyFree(int pkey)
{
    int ret;

    for(size_t rid = 0; rid < NUM_DOMAINS; ++rid)
    {
        if(g_data.ranges[rid].used)
        {
            printf("range[%zu] addr %p len %zu (%s) still uses", rid, g_data.ranges[rid].addr, g_data.ranges[rid].len, g_data.ranges[rid].name);
            errno = EPERM;
            return -1;
        }
    }

    for(size_t did = 0; did < NUM_DOMAINS; ++did)
    {
        if(g_data.domains[did].used)
        {
            PKUKey* domain = &g_data.domains[did];
            if(domain->used)
            {
                domain->used = false;
                printf("revoked domain[%zu]\n", did);
            }
        }
    }

    if(g_LazyFree)
    {
        ret = 0;
    }
    else
    {
        ret = PKUDomainFree(pkey);
    }

    return ret;
}

int DomainProtect(void* addr, size_t length, unsigned int pkey)
{
    unsigned char buf[12] = {0x01, 0x20};
    size_t temp = (size_t)addr;
    for(int i = 3; i >= 0; --i)
    {
        buf[i+2] = temp & 0xff;
        temp >>= 8;
    }
    temp = length;
    for(int i = 3; i >= 0; --i)
    {
        buf[i+6] = temp & 0xff;
        temp >>= 8;
    }
    buf[10] = 3;
    buf[11] = pkey;
    int error = WASICALL(buf, sizeof(buf));
    if(error != 0)
    {
        perror("DomainProtect failed");
    }
    return 0;
}

int PKUCreateDomain(unsigned int flags)
{
    // PKU_KEY_* flags
    unsigned char buf[12] = {0x01, 0x21};
    int error = WASICALL(buf, sizeof(buf));
    if(error != 0)
    {
        perror("PKUCreateDomain failed");
        return -1;
    }
    if(buf[2] >= 16)
    {
        return 0;
    }
    else
    {
        keys[buf[2]].pkey = buf[2];
        keys[buf[2]].used = 1;
        return buf[2];
    }
}

int RegisterPKUCall(int did, pFunc entry)
{
    if(!DomainExists(did))
    {
        perror("Domain does not exist");
        return -EINVAL;
    }

    int PKUCallID = 0;
    for(; PKUCallID < NUM_REGISTERED_PKUCALLS; ++PKUCallID)
    {
        if(!RegisteredPKUCalls[PKUCallID].entry)
        {
            // We found an empty ecall slot
            break;
        }
    }

    // check for valid id
    if(PKUCallID < 0 || PKUCallID >= NUM_REGISTERED_PKUCALLS)
    {
        perror("pku call id is out of range");
        return -EACCES;
    }

    if(RegisteredPKUCalls[PKUCallID].entry != 0)
    {
        perror("pku call id already used");
        return -EACCES;
    }

    // register ecall
    RegisteredPKUCalls[PKUCallID].did   = did;
    RegisteredPKUCalls[PKUCallID].entry = entry;
    return PKUCallID;
}

int ReadPKRU()
{
    unsigned char buf[12] = {0x0F, 0x01, 0xEE};
    int error = WASICALL(buf, sizeof(buf));
    if(error != 0)
    {
        perror("ReadPKRU failed");
        return -1;
    }

    int pkru = 0;
    for(int i = 3; i < 7; ++i)
    {
        pkru <<= 8;
        pkru += buf[i];
    }
    return pkru;
}

static int WritePKRU(unsigned int pkru)
{
    unsigned char buf[12] = {0x0F, 0x01, 0xEF};
    unsigned int temp = pkru;
    for(int i = 3; i >= 0; --i)
    {
        buf[i+3] = temp & 0xff;
        temp >>= 8;
    }
    int error = WASICALL(buf, sizeof(buf));
    if(error != 0)
    {
        perror("WritePkru failed");
        return -1;
    }
    return 0;
}

static int __attribute__((noinline)) rdpkru()
{
    volatile int ecx = 0;
    return ecx;
}

static int __attribute__((noinline)) wrpkru(int pkru)
{
    volatile int eax = pkru;
    return eax;
}

static int SetPkey(pkey_t pkey, unsigned int prot)
{
    int pkey_shift = pkey * 2;
    int new_pkru_bits = 0;

    if(prot & PKEY_DISABLE_ACCESS)
    {
        new_pkru_bits |= PKEY_DISABLE_ACCESS;
    }
    if(prot & PKEY_DISABLE_WRITE)
    {
        new_pkru_bits |= PKEY_DISABLE_WRITE;
    }

    /* Shift the bits in to the correct place in PKRU for pkey: */
    new_pkru_bits <<= pkey_shift;

    /* Get old PKRU and mask off any old bits in place: */
    int old_pkru = rdpkru();
    if(old_pkru == 0)
    {
        old_pkru = 0x55555554;
    }
    old_pkru &= ~((PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE) << pkey_shift);

    /* Write old part along with new part: */
    wrpkru(old_pkru | new_pkru_bits);
    return 0;
}

int PKUDomainAssignPkey(int did, int pkey, int flags, int AccessRights)
{
    // set pkey prot
    if(!DomainExists(GetCurrentDid()))
    {
        perror("PKUDomainAssignPkey GetCurrentDid does not exist");
        return -EINVAL;
    }

    if(!DomainExists(did))
    {
        perror("PKUDomainAssignPkey target domain does not exist");
        return -EINVAL;
    }

    if(AccessRights & ~(unsigned int)(PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE))
    {
        perror("PKUDomainAssignPkey invalid AccessRights");
        return -EINVAL;
    }

    SetPkey(keys[did].pkey, keys[did].perm);
    return 0;
}

int PKUPkeyMprotect(void* addr, size_t len, int prot, int pkey)
{
    return PKUMprotect(addr, len, prot);
}

static size_t GS_MmapMemory = 0;

void* NaiveMmap(size_t bytes)
{
    bytes = PAGE_ALIGN(bytes);
    GS_MmapMemory += bytes;
    return PKUMalloc(bytes);
}

void* PKUMmap(void* addr, size_t length, int prot, int flags, int fd, int offset)
{
    unsigned char buf[12] = {0x01, 0x2B};
    size_t temp = (size_t)addr;
    for(int i = 3; i >= 0; --i)
    {
        buf[i+2] = temp & 0xff;
        temp >>= 8;
    }
    temp = length;
    for(int i = 3; i >= 0; --i)
    {
        buf[i+6] = temp & 0xff;
        temp >>= 8;
    }
    buf[10] = prot;
    buf[11] = flags;
    int error = WASICALL(buf, sizeof(buf));
    if(error != 0)
    {
        perror("PKUMmap failed");
    }
    temp = 0;
    for(int i = 2; i < 6; i++)
    {
        temp <<= 8;
        temp += buf[i];
    }
    size_t len = 0;
    for(int i = 6; i < 10; i++)
    {
        len <<= 8;
        len += buf[i];
    }
    if(len != length)
    {
        temp = 0;
        printf("test\n");
    }
    GS_MmapMemory += len;
    return (void*)temp;
}

int PKUMunmap(void* addr, size_t len)
{
    // munmap
    return 0;
}

static void* MmapAddr = NULL;

int PKUMprotect(void* addr, size_t len, int prot)
{
    if(addr == NULL && MmapAddr == NULL)
    {
        MmapAddr = PKUMmap(NULL, len, prot, 0x2 | 0x20, -1, 0);
    }
    if(MmapAddr != NULL)
    {
        DomainProtect(MmapAddr, len, 0);
    }
    return 0;
}

int EnableSectionGuardPage(int did)
{
    // Enable default in the runtime
    return 0;
}

int PKUSwitch(int PKUCallID)
{
    int did = RegisteredPKUCalls[PKUCallID].did;
    SetPkey(keys[did].pkey, keys[did].perm);
    SetCurrentDid(did);
    return 0;
}

int PKURestore(int did)
{
    SetPkey(keys[GetCurrentDid()].pkey, PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE);
    SetCurrentDid(did);
    return 0;
}

// int PKUSwitch(int PKUCallID)
// {
//     PKUMprotect(NULL, 4096, 3);
//     return 0;
// }

// int PKURestore(int did)
// {
//     PKUMprotect(NULL, 4096, 0);
//     return 0;
// }

size_t GetMemorySize()
{
    // printf("%zd, %zd, %zd\n", MemorySize(), GS_MmapMemory, g_MallocNumber);
    return MemorySize() + GS_MmapMemory + g_ExtraMemory;
}