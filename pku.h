#ifndef _PKU_H_
#define _PKU_H_

#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <malloc.h>
#include <time.h>
#include <sys/socket.h>
// #include "pkumalloc.h"
// #include "native.h"

#define SIZE_T_ONE ((size_t)1)
#define PAGE_ALIGN(S)\
    (((S) + (PAGE_SIZE - SIZE_T_ONE)) & ~(PAGE_SIZE - SIZE_T_ONE))

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

#define GENPKU(name, ...) \
    static int pkucall_id_##name = 0; \
    \
    void pkucall_##name(__VA_ARGS__) { \
        PKUSwitch(pkucall_id_##name); \
    } \
    \

#define GENPK(name, return_type, ...) \
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

extern size_t g_malloc;
extern size_t g_free;
extern size_t g_extra;

int DomainProtect(void* addr, size_t length, unsigned int pkey);

// void* NaiveMmap(size_t bytes);

#ifdef __cplusplus
}
#endif

// static void* MallocHook(size_t bytes)
// {
//     void* ptr = NULL;
//     // size_t size = bytes;
//     // if(g_malloc < 4000022)
//     // {
//     //     #ifdef __WASI_MMAP__
//     //     if(GetCurrentDid() != 0)
//     //     {
//     //         size = PAGE_ALIGN(bytes);
//     //         ptr = NaiveMmap(size);
//     //         // ptr = PKUMmap(NULL, size, 3, 0x2 | 0x20, -1, 0);
//     //         // DomainProtect(ptr, size, GetCurrentDid());
//     //     }
//     //     else
//     //     {
//     //         ptr = NaiveMmap(size);
//     //     }
//     //     #else
//     //     ptr = PKUMalloc(size);
//     //     // g_extra += PAGE_SIZE;
//     //     // size = PAGE_ALIGN(bytes);
//     //     // ptr = malloc(size);
//     //     // g_extra += size;
//     //     #endif
//     // }
//     // else
//     // {
//     //     // size = PAGE_ALIGN(bytes);
//     //     ptr = malloc(size);
//     //     g_extra += size;
//     // }
//     // // ptr = PKUMalloc(size);
//     // g_malloc += 1;
//     ptr = malloc(bytes);
//     return ptr;
// }

// static void FreeHook(void* ptr)
// {
//     free(ptr);
//     // if(GetCurrentDid() != 0)
//     // {
//     //     DomainProtect(ptr, PAGE_SIZE, 0);
//     // }
//     // if(g_free < 4000022)
//     // {
//     //     #ifdef __WASI_MMAP__
//     //     PKUFree(ptr);
//     //     #else
//     //     PKUFree(ptr);
//     //     #endif
//     //     // free(ptr);
//     // }
//     // else
//     // {
//     //     free(ptr);
//     // }
//     // PKUFree(ptr);
//     // g_free += 1;
// }

#ifdef __cplusplus
extern "C"
{
#endif

typedef uint16_t pkey_t;

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

int PKUInit(int flags);

int PKUDeinit(void);

int PKUCreateDomain(unsigned int flags);

int PKUDomainFree(int did);

int PKUPkeyAlloc(unsigned int flags, unsigned int AccessRights);

int PKUPkeyFree(int pkey);

int PKUDomainAssignPkey(int did, int pkey, int flags, int AccessRights);

int PKUPkeyMprotect(void* addr, size_t len, int prot, int pkey);

int PKUMprotect(void* addr, size_t len, int prot);

int RegisterPKUCall(int did, pFunc entry);

int PKUDomainAllowCaller(int CallerDid, unsigned int flags);

int EnableSectionGuardPage(int did);

int PKUSwitch(int PKUCallID);
int PKURestore(int did);

int ReadPkru();

size_t GetMemorySize();

// extern off_t PKUFopen(const char* pathname, const char* mode);
// extern off_t PKUFdopen(int fildes, const char *mode);
// extern int PKUFclose(size_t stream);
// extern int PKUFflush(size_t stream);
// extern int PKUFgetc(size_t stream);
// extern int PKUUngetc(int c, size_t stream);
// extern size_t PKUFread(void* ptr, size_t size, size_t n, size_t stream);
// extern size_t PKUFwrite(void* ptr, size_t size, size_t n, size_t stream);
// extern int PKUFseek(size_t stream, long offset, int whence);
extern void PKURewind(size_t stream);
// extern int PKUFeof(size_t stream);
// extern int PKUFerror(size_t stream);
// extern int PKUFileno(size_t stream);
// extern int PKUOpen(const char *pathname, int flags, size_t mode);
// extern int PKUClose(int fd);
// extern int PKUStat(const char* pathname, struct stat* statbuf);
// extern int PKUUtime(const char *filename, const struct utimbuf *times);
// extern void* PKUMalloc(size_t size);
// extern void PKUFree(void* ptr);
// extern int PKUClockGettime(clockid_t clockid, struct timespec *tp);

// extern size_t PKUFopen(const char* pathname, size_t path, const char* mode, size_t m, size_t* ret);
// extern size_t PKUFdopen(int fildes, const char *mode, size_t m, size_t* ret);
// extern int PKUFclose(size_t stream);
// extern int PKUFflush(size_t stream);
// extern int PKUFgetc(size_t stream);
// extern int PKUUngetc(int c, size_t stream);
// extern size_t PKUFread(void* ptr, size_t size, size_t n, size_t stream);
// extern size_t PKUFwrite(void* ptr, size_t size, size_t n, size_t stream);
// extern int PKUFseek(size_t stream, long offset, int whence);
// extern void PKURewind(size_t stream);
// extern int PKUFeof(size_t stream);
// extern int PKUFerror(size_t stream);
// extern int PKUFileno(size_t stream);
// extern int PKUOpen(const char *pathname, size_t path, int flags, size_t mode, int* ret);
// extern int PKUClose(int fd);
// extern int PKUStat(const char* pathname, size_t path, struct stat* statbuf, size_t s);
// extern int PKUUtime(const char *filename, size_t file, const struct utimbuf *times, size_t t);
// extern void PKUFree(void* ptr);
// extern int PKUClockGettime(clockid_t clockid, struct timespec *tp, size_t t);
extern off_t PKULseek(int fd, off_t offset, int whence);

#ifndef EHOSTDOWN
#define EHOSTDOWN 112
#endif

#ifndef F_RDLCK
# define F_RDLCK		0	/* Read lock.  */
# define F_WRLCK		1	/* Write lock.  */
# define F_UNLCK		2	/* Remove lock.  */
#endif

#ifndef FIOASYNC
#define FIOASYNC	0x5452
#endif

#ifndef F_GETLK
#define F_GETLK		5
#define F_SETLK		6
#define F_SETLKW	7
#endif
#ifndef F_SETOWN
#define F_SETOWN	8	/* for sockets. */
#define F_GETOWN	9	/* for sockets. */
#endif
#ifndef F_SETSIG
#define F_SETSIG	10	/* for sockets. */
#define F_GETSIG	11	/* for sockets. */
#endif

#ifndef CMSG_DATA

enum PKU_priority_which
{
  PRIO_PROCESS = 0,		/* WHO is a process ID.  */
#define PRIO_PROCESS PRIO_PROCESS
  PRIO_PGRP = 1,		/* WHO is a process group ID.  */
#define PRIO_PGRP PRIO_PGRP
  PRIO_USER = 2			/* WHO is a user ID.  */
#define PRIO_USER PRIO_USER
};

struct cmsghdr {
#if __LONG_MAX > 0x7fffffff && __BYTE_ORDER == __BIG_ENDIAN
	int __pad1;
#endif
	socklen_t cmsg_len;
#if __LONG_MAX > 0x7fffffff && __BYTE_ORDER == __LITTLE_ENDIAN
	int __pad1;
#endif
	int cmsg_level;
	int cmsg_type;
};

#define __CMSG_LEN(cmsg) (((cmsg)->cmsg_len + sizeof(long) - 1) & ~(long)(sizeof(long) - 1))
#define __CMSG_NEXT(cmsg) ((unsigned char *)(cmsg) + __CMSG_LEN(cmsg))
#define __MHDR_END(mhdr) ((unsigned char *)(mhdr)->msg_control + (mhdr)->msg_controllen)

#define CMSG_DATA(cmsg) ((unsigned char *) (((struct cmsghdr *)(cmsg)) + 1))
#define CMSG_NXTHDR(mhdr, cmsg) ((cmsg)->cmsg_len < sizeof (struct cmsghdr) || \
	__CMSG_LEN(cmsg) + sizeof(struct cmsghdr) >= __MHDR_END(mhdr) - (unsigned char *)(cmsg) \
	? 0 : (struct cmsghdr *)__CMSG_NEXT(cmsg))
#define CMSG_FIRSTHDR(mhdr) ((size_t) (mhdr)->msg_controllen >= sizeof (struct cmsghdr) ? (struct cmsghdr *) (mhdr)->msg_control : (struct cmsghdr *) 0)

#define CMSG_ALIGN(len) (((len) + sizeof (size_t) - 1) & (size_t) ~(sizeof (size_t) - 1))
#define CMSG_SPACE(len) (CMSG_ALIGN (len) + CMSG_ALIGN (sizeof (struct cmsghdr)))
#define CMSG_LEN(len)   (CMSG_ALIGN (sizeof (struct cmsghdr)) + (len))

#define SCM_RIGHTS      0x01
#define SCM_CREDENTIALS 0x02

enum PKU_rlimit_resource
{
  /* Per-process CPU limit, in seconds.  */
  RLIMIT_CPU = 0,
#define RLIMIT_CPU RLIMIT_CPU

  /* Largest file that can be created, in bytes.  */
  RLIMIT_FSIZE = 1,
#define	RLIMIT_FSIZE RLIMIT_FSIZE

  /* Maximum size of data segment, in bytes.  */
  RLIMIT_DATA = 2,
#define	RLIMIT_DATA RLIMIT_DATA

  /* Maximum size of stack segment, in bytes.  */
  RLIMIT_STACK = 3,
#define	RLIMIT_STACK RLIMIT_STACK

  /* Largest core file that can be created, in bytes.  */
  RLIMIT_CORE = 4,
#define	RLIMIT_CORE RLIMIT_CORE

  /* Largest resident set size, in bytes.
     This affects swapping; processes that are exceeding their
     resident set size will be more likely to have physical memory
     taken from them.  */
  __RLIMIT_RSS = 5,
#define	RLIMIT_RSS __RLIMIT_RSS

  /* Number of open files.  */
  RLIMIT_NOFILE = 7,
  __RLIMIT_OFILE = RLIMIT_NOFILE, /* BSD name for same.  */
#define RLIMIT_NOFILE RLIMIT_NOFILE
#define RLIMIT_OFILE __RLIMIT_OFILE

  /* Address space limit.  */
  RLIMIT_AS = 9,
#define RLIMIT_AS RLIMIT_AS

  /* Number of processes.  */
  __RLIMIT_NPROC = 6,
#define RLIMIT_NPROC __RLIMIT_NPROC

  /* Locked-in-memory address space.  */
  __RLIMIT_MEMLOCK = 8,
#define RLIMIT_MEMLOCK __RLIMIT_MEMLOCK

  /* Maximum number of file locks.  */
  __RLIMIT_LOCKS = 10,
#define RLIMIT_LOCKS __RLIMIT_LOCKS

  /* Maximum number of pending signals.  */
  __RLIMIT_SIGPENDING = 11,
#define RLIMIT_SIGPENDING __RLIMIT_SIGPENDING

  /* Maximum bytes in POSIX message queues.  */
  __RLIMIT_MSGQUEUE = 12,
#define RLIMIT_MSGQUEUE __RLIMIT_MSGQUEUE

  /* Maximum nice priority allowed to raise to.
     Nice levels 19 .. -20 correspond to 0 .. 39
     values of this resource limit.  */
  __RLIMIT_NICE = 13,
#define RLIMIT_NICE __RLIMIT_NICE

  /* Maximum realtime priority allowed for non-priviledged
     processes.  */
  __RLIMIT_RTPRIO = 14,
#define RLIMIT_RTPRIO __RLIMIT_RTPRIO

  /* Maximum CPU time in µs that a process scheduled under a real-time
     scheduling policy may consume without making a blocking system
     call before being forcibly descheduled.  */
  __RLIMIT_RTTIME = 15,
#define RLIMIT_RTTIME __RLIMIT_RTTIME

  __RLIMIT_NLIMITS = 16,
  __RLIM_NLIMITS = __RLIMIT_NLIMITS
#define RLIMIT_NLIMITS __RLIMIT_NLIMITS
#define RLIM_NLIMITS __RLIM_NLIMITS
};
#endif

struct PKUPasswd
{
    unsigned long long pw_name;
    unsigned long long pw_passwd;
    unsigned int pw_uid;
    unsigned int pw_gid;
    unsigned long long pw_gecos;
    unsigned long long pw_dir;
    unsigned long long pw_shell;
};

struct PKUGroup
{
    unsigned long long gr_name;
    unsigned long longgr_passwd;
    unsigned int gr_gid;
    unsigned long long gr_mem;
};

struct PKURlimit
{
    unsigned long long rlim_cur;
    unsigned long long rlim_max;
};

extern int PKUGetpwnam(const char* name, size_t path, struct PKUPasswd* pwd, size_t len);
extern int PKUGetgrnam(const char* name, size_t path, struct PKUGroup* grp, size_t len);
extern int PKUSetpriority(int which, int who, int prio);
extern int PKUSetrlimit(int resource, const struct PKURlimit* rlim, size_t len);
extern int PKUGetrlimit(int resource, struct PKURlimit* rlim, size_t len);
extern int PKUInitgroups(const char* user, size_t len, gid_t group);
extern int PKUChown(const char* pathname, size_t len, uid_t owner, gid_t group);
extern void* PKUMmap(void* addr, size_t length, int prot, int flags, int fd, off_t offset);
extern int PKUMunmap(void* addr, size_t length);
extern void RaidenTest(void);

#ifdef __cplusplus
}
#endif

// #define malloc(bytes) MallocHook(bytes)
// #define free(ptr) FreeHook(ptr)

#endif