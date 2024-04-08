#include <stdio.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include "pku.h"

int DomainProtect(void* addr, size_t length, unsigned int pkey)
{
    unsigned char buf[12] = {0x01, 0x49};
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
    int error = getentropy(buf, sizeof(buf));
    if(error != 0)
    {
        perror("DomainProtect failed");
    }
    return 0;
}

size_t g_malloc = 0;
size_t g_free = 0;
size_t g_extra = 0;

PKUKey keys[NUM_DOMAINS];
PKUCall RegisteredPKUCalls[NUM_REGISTERED_PKUCALLS];

__attribute__((constructor)) void PKUInitCtor()
{
    keys[0].perm = 0;
    keys[0].pkey = 0;
    keys[0].used = 1;
}

int PKUCreateDomain(unsigned int flags)
{
    // PKU_KEY_* flags
    unsigned char buf[12] = {0x01, 0x4A};
    int error = getentropy(buf, sizeof(buf));
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

static bool inline DomainExists(int did)
{
    return (did >= 0 && did < NUM_DOMAINS && keys[did].used);
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

int ReadPkru()
{
    unsigned char buf[12] = {0x0F, 0xEE};
    int error = getentropy(buf, sizeof(buf));
    if(error != 0)
    {
        perror("ReadPkru failed");
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

static int WritePkru(unsigned int pkru)
{
    unsigned char buf[12] = {0x0F, 0xEF};
    unsigned int temp = pkru;
    for(int i = 3; i >= 0; --i)
    {
        buf[i+3] = temp & 0xff;
        temp >>= 8;
    }
    int error = getentropy(buf, sizeof(buf));
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
    // if(!DomainExists(GetCurrentDid()))
    // {
    //     perror("PKUDomainAssignPkey GetCurrentDid does not exist");
    //     return -EINVAL;
    // }

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

int PKUInit(int flags)
{
    // Allocate DID for meta data in root domain
    return 0;
}

int PKUDeinit(void)
{
    // Unloaded module
    return 0;
}

int PKUDomainFree(int did)
{
    // unprotect memory ranges
    // free pku keys
    // wipe all related pku data structures
    return 0;
}

int PKUPkeyAlloc(unsigned int flags, unsigned int AccessRights)
{
    // pkey_alloc returns a negative number on errors, or a positive pkey. pkey 0 is reserved and should never be returned by kernel.
    return 0;
}

int PKUPkeyFree(int pkey)
{
    // free pkey in the kernel
    return 0;
}

int PKUPkeyMprotect(void* addr, size_t len, int prot, int pkey)
{
    // pkey_mprotect
    return 0;
}

static size_t GS_MmapMemory = 0;

// void* NaiveMmap(size_t bytes)
// {
//     bytes = PAGE_ALIGN(bytes);
//     GS_MmapMemory += bytes;
//     return PKUMalloc(bytes);
// }

// void* PKUMmap(void* addr, size_t length, int prot, int flags, int fd, int offset)
// {
//     unsigned char buf[12] = {0x00, 0x09};
//     size_t temp = (size_t)addr;
//     for(int i = 3; i >= 0; --i)
//     {
//         buf[i+2] = temp & 0xff;
//         temp >>= 8;
//     }
//     temp = length;
//     for(int i = 3; i >= 0; --i)
//     {
//         buf[i+6] = temp & 0xff;
//         temp >>= 8;
//     }
//     buf[10] = prot;
//     buf[11] = flags;
//     int error = getentropy(buf, sizeof(buf));
//     if(error != 0)
//     {
//         perror("PKUMmap failed");
//     }
//     temp = 0;
//     for(int i = 2; i < 6; i++)
//     {
//         temp <<= 8;
//         temp += buf[i];
//     }
//     size_t len = 0;
//     for(int i = 6; i < 10; i++)
//     {
//         len <<= 8;
//         len += buf[i];
//     }
//     if(len != length)
//     {
//         temp = 0;
//         printf("test\n");
//     }
//     GS_MmapMemory += len;
//     return (void*)temp;
// }

static void* MmapAddr = NULL;

// int PKUMprotect(void* addr, size_t len, int prot)
// {
//     if(addr == NULL && MmapAddr == NULL)
//     {
//         MmapAddr = PKUMmap(NULL, len, prot, 0x2 | 0x20, -1, 0);
//     }
//     if(MmapAddr != NULL)
//     {
//         DomainProtect(MmapAddr, len, 0);
//     }
//     return 0;
// }

int PKUDomainAllowCaller(int CallerDid, unsigned int flags)
{
    // Permit other domains to invoke pkucalls.
    return 0;
}

int EnableSectionGuardPage(int did)
{
    return 0;
}

int PKUSwitch(int PKUCallID)
{
    int did = RegisteredPKUCalls[PKUCallID].did;
    SetPkey(keys[did].pkey, keys[did].perm);
    // SetCurrentDid(did);
    return 0;
}

int PKURestore(int did)
{
    // SetPkey(keys[GetCurrentDid()].pkey, PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE);
    // SetCurrentDid(did);
    return 0;
}

// int PKUFopen(const char* pathname, const char* mode)
// {
//     int len1 = strlen(pathname);
//     int len2 = strlen(mode);
//     unsigned char* buf = (unsigned char*)malloc(len1 + len2 + 3);
//     buf[0] = 0x01;
//     buf[1] = 0x01;
//     buf[2] = len1;
//     strncpy(&buf[3], pathname, len1);
//     strncpy(&buf[3 + len1], mode, len2);
//     int ret = getentropy(buf, len1 + len2 + 3);
//     return ret;
// }

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

// size_t GetMemorySize()
// {
//     printf("%zd, %zd, %zd\n", MemorySize(), GS_MmapMemory, g_malloc);
//     return MemorySize() + GS_MmapMemory + g_extra;
// }