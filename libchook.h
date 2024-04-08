#ifndef _LIBCHOOK_H_
#define _LIBCHOOK_H_

#include <stdio.h>
#include <fcntl.h>
#include <utime.h>
#include <unistd.h>
#include <sys/stat.h>
#include "pku.h"

extern unsigned long long g_LibcTime;

unsigned long long GetLibcTime();

static unsigned long long ReadTime()
{
    struct timespec tv;
    // #ifdef _PKU_WASM
    // PKUClockGettime(__WASI_CLOCKID_MONOTONIC, &tv);
    // #else
    clock_gettime(CLOCK_MONOTONIC, &tv);
    // #endif
    return (unsigned long long)tv.tv_sec * 1000000000 + tv.tv_nsec;
}

static FILE* FopenHook(const char * pathname, const char * mode)
{
    unsigned long long start = ReadTime();
    FILE* ret = fopen(pathname, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FopenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static FILE* FdopenHook(int fd, const char *mode)
{
    unsigned long long start = ReadTime();
    FILE* ret = fdopen(fd, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FdopenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static FILE* FmemopenHook(void* buf, size_t size, const char *mode)
{
    unsigned long long start = ReadTime();
    FILE* ret = fmemopen(buf, size, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FmemopenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FcloseHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = fclose(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FcloseHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FflushHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = fflush(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FflushHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int VfprintfHook(FILE *stream, const char *format, va_list arg)
{
    unsigned long long start = ReadTime();
    int ret = vfprintf(stream, format, arg);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int VprintfHook(const char * format, va_list ap)
{
    unsigned long long start = ReadTime();
    int ret = vprintf(format, ap);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int VsprintfHook(char * str, const char * format, va_list ap)
{
    unsigned long long start = ReadTime();
    int ret = vsprintf(str, format, ap);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int VsnprintfHook(char * str, size_t size, const char * format, va_list ap)
{
    unsigned long long start = ReadTime();
    int ret = vsnprintf(str, size, format, ap);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int VscanfHook(const char * format, va_list ap)
{
    unsigned long long start = ReadTime();
    int ret = vscanf(format, ap);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int VfscanfHook(FILE * stream, const char * format, va_list ap)
{
    unsigned long long start = ReadTime();
    int ret = vfscanf(stream, format, ap);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int VsscanfHook(const char * str, const char * format, va_list ap)
{
    unsigned long long start = ReadTime();
    int ret = vsscanf(str, format, ap);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int FgetcHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = fgetc(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FgetcHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int GetcHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = getc(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("GetcHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int GetcharHook()
{
    unsigned long long start = ReadTime();
    int ret = getchar();
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int FputcHook(int c, FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = fputc(c, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int PutcHook(int c, FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = putc(c, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int PutcharHook(int c)
{
    unsigned long long start = ReadTime();
    int ret = putchar(c);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static char* FgetsHook(char * s, int n, FILE * stream)
{
    unsigned long long start = ReadTime();
    char* ret = fgets(s, n, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int FputsHook(const char * s, FILE * stream)
{
    unsigned long long start = ReadTime();
    int ret = fputs(s, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int PutsHook(const char *s)
{
    unsigned long long start = ReadTime();
    int ret = puts(s);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static int UngetcHook(int c, FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = ungetc(c, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("UngetcHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static size_t FreadHook(void *ptr, size_t size, size_t nmemb, FILE * stream)
{
    unsigned long long start = ReadTime();
    size_t ret = fread(ptr, size, nmemb, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FreadHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static size_t FwriteHook(const void *ptr, size_t size, size_t nmemb, FILE * stream)
{
    unsigned long long start = ReadTime();
    size_t ret = fwrite(ptr, size, nmemb, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FwriteHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FseekHook(FILE *stream, long int offset, int whence)
{
    unsigned long long start = ReadTime();
    int ret = fseek(stream, offset, whence);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FseekHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static long int FtellHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    long int ret = ftell(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FtellHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static void RewindHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    rewind(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("RewindHook: %lld, %lld\n", end - start, g_LibcTime);
}

static int FgetposHook(FILE *stream, fpos_t *pos)
{
    unsigned long long start = ReadTime();
    int ret = fgetpos(stream, pos);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FgetposHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FeofHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = feof(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FeofHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FerrorHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = ferror(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FerrorHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static void PerrorHook(const char *s)
{
    unsigned long long start = ReadTime();
    perror(s);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PerrorHook: %lld, %lld\n", end - start, g_LibcTime);
}

static int FilenoHook(FILE *stream)
{
    unsigned long long start = ReadTime();
    int ret = fileno(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FilenoHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static long int RandomHook()
{
    unsigned long long start = ReadTime();
    long int ret = random();
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("RandomHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static void SrandHook(unsigned int seed)
{
    unsigned long long start = ReadTime();
    srand(seed);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("SrandHook: %lld, %lld\n", end - start, g_LibcTime);
}

static void* MallocHook(size_t size)
{
    unsigned long long start = ReadTime();
    void* ret = malloc(size);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("MallocHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static void* CallocHook(size_t nmemb, size_t size)
{
    unsigned long long start = ReadTime();
    void* ret = calloc(nmemb, size);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static void* ReallocHook(void *ptr, size_t size)
{
    unsigned long long start = ReadTime();
    void* ret = realloc(ptr, size);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    return ret;
}

static void FreeHook(void *ptr)
{
    unsigned long long start = ReadTime();
    free(ptr);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
}

static int PosixmemalignHook(void **memptr, size_t alignment, size_t size)
{
    unsigned long long start = ReadTime();
    int ret = posix_memalign(memptr, alignment, size);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PosixmemalignHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int AtexitHook(void (*function)(void))
{
    unsigned long long start = ReadTime();
    int ret = atexit(function);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("AtexitHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int SystemHook(const char *command)
{
    unsigned long long start = ReadTime();
    int ret = system(command);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("SystemHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int UtimeHook(const char *filename, const struct utimbuf * times)
{
    unsigned long long start = ReadTime();
    int ret = utime(filename, times);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("UtimeHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int StatHook(const char * pathname, struct stat * statbuf)
{
    unsigned long long start = ReadTime();
    int ret = stat(pathname, statbuf);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("StatHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int LstatHook(const char * pathname, struct stat * statbuf)
{
    unsigned long long start = ReadTime();
    int ret = lstat(pathname, statbuf);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("LstatHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int MkdirHook(const char *pathname, mode_t mode)
{
    unsigned long long start = ReadTime();
    int ret = mkdir(pathname, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("MkdirHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int AccessHook(const char *pathname, int mode)
{
    unsigned long long start = ReadTime();
    int ret = access(pathname, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("AccessHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int OpenHook(const char *pathname, int flags, size_t mode)
{
    unsigned long long start = ReadTime();
    int ret = open(pathname, flags, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("OpenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int CloseHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = close(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("CloseHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static ssize_t ReadHook(int fd, void* buf, size_t count)
{
    unsigned long long start = ReadTime();
    ssize_t ret = read(fd, buf, count);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("ReadHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static ssize_t WriteHook(int fd, const void* buf, size_t count)
{
    unsigned long long start = ReadTime();
    ssize_t ret = write(fd, buf, count);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("WriteHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static off_t LseekHook(int fd, off_t offset, int whence)
{
    unsigned long long start = ReadTime();
    off_t ret = lseek(fd, offset, whence);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("LseekHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FsyncHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = fsync(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FsyncHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FdatasyncHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = fdatasync(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FdatasyncHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FstatHook(int fd, struct stat *buf)
{
    unsigned long long start = ReadTime();
    int ret = fstat(fd, buf);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FstatHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static long int PathconfHook(const char *path, int name)
{
    unsigned long long start = ReadTime();
    long int ret = pathconf(path, name);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PathconfHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int IsattyHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = isatty(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("IsattyHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int UnlinkHook(const char *pathname)
{
    unsigned long long start = ReadTime();
    int ret = unlink(pathname);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("UnlinkHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int RmdirHook(const char *pathname)
{
    unsigned long long start = ReadTime();
    int ret = rmdir(pathname);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("RmdirHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int TruncateHook(const char *path, off_t length)
{
    unsigned long long start = ReadTime();
    int ret = truncate(path, length);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("TruncateHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int FtruncateHook(int fd, off_t length)
{
    unsigned long long start = ReadTime();
    int ret = ftruncate(fd, length);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("FtruncateHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

#ifdef _PKU_WASM

static int PKUFcloseHook(size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUFclose(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFcloseHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFflushHook(size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUFflush(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFflushHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFgetcHook(size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUFgetc(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFgetcHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUUngetcHook(int c, size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUUngetc(c, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUUngetcHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static size_t PKUFreadHook(void* ptr, size_t size, size_t n, size_t stream)
{
    unsigned long long start = ReadTime();
    size_t ret = PKUFread(ptr, size, n, stream);
    if(ret == 0) ret = size * n;
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFreadHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static size_t PKUFwriteHook(void* ptr, size_t size, size_t n, size_t stream)
{
    unsigned long long start = ReadTime();
    size_t ret = PKUFwrite(ptr, size, n, stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFwriteHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFseekHook(size_t stream, long offset, int whence)
{
    unsigned long long start = ReadTime();
    size_t ret = PKUFseek(stream, offset, whence);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFseekHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static void PKURewindHook(size_t stream)
{
    unsigned long long start = ReadTime();
    PKURewind(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKURewindHook: %lld, %lld\n", end - start, g_LibcTime);
}

static int PKUFeofHook(size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUFeof(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFeofHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFerrorHook(size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUFerror(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFerrorHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFilenoHook(size_t stream)
{
    unsigned long long start = ReadTime();
    int ret = PKUFileno(stream);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFilenoHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUCloseHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = PKUClose(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUCloseHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static void* PKUMallocHook(size_t size)
{
    unsigned long long start = ReadTime();
    void* ret = PKUMalloc(size);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUMallocHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static off_t PKUFopenHook(const char* pathname, const char* mode)
{
    unsigned long long start = ReadTime();
    off_t ret = PKUFopen(pathname, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFopenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static off_t PKUFdopenHook(int fildes, const char *mode)
{
    unsigned long long start = ReadTime();
    off_t ret = PKUFdopen(fildes, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFdopenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUOpenHook(const char *pathname, int flags, size_t mode)
{
    unsigned long long start = ReadTime();
    int ret = PKUOpen(pathname, flags, mode);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUOpenHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static ssize_t PKUReadHook(int fd, void* buf, size_t count)
{
    unsigned long long start = ReadTime();
    ssize_t ret = PKURead(fd, buf, count);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUReadHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static ssize_t PKUWriteHook(int fd, const void* buf, size_t count)
{
    unsigned long long start = ReadTime();
    ssize_t ret = PKUWrite(fd, buf, count);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUWriteHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUStatHook(const char* filename, struct stat* buf)
{
    unsigned long long start = ReadTime();
    int ret = PKUStat(filename, buf);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUStatHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUUtimeHook(const char *filename, const struct utimbuf *times)
{
    unsigned long long start = ReadTime();
    int ret = PKUUtime(filename, times);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUUtimeHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static off_t PKULseekHook(int fd, off_t offset, int whence)
{
    unsigned long long start = ReadTime();
    off_t ret = PKULseek(fd, offset, whence);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKULseekHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFsyncHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = PKUFsync(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFsyncHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFdatasyncHook(int fd)
{
    unsigned long long start = ReadTime();
    int ret = PKUFdatasync(fd);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFdatasyncHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

static int PKUFstatHook(int fd, struct stat *buf)
{
    unsigned long long start = ReadTime();
    int ret = PKUFstat(fd, buf);
    unsigned long long end = ReadTime();
    g_LibcTime += end - start;
    printf("PKUFstatHook: %lld, %lld\n", end - start, g_LibcTime);
    return ret;
}

// static off_t PKUFopenHook(const char* pathname, size_t len1, const char* mode, size_t len2, size_t* fd)
// {
//     unsigned long long start = ReadTime();
//     off_t ret = PKUFopen(pathname, len1, mode, len2, fd);
//     unsigned long long end = ReadTime();
//     g_LibcTime += end - start;
//     printf("PKUFopenHook: %lld, %lld\n", end - start, g_LibcTime);
//     return ret;
// }

// static off_t PKUFdopenHook(int fildes, const char *mode, size_t len, size_t* fd)
// {
//     unsigned long long start = ReadTime();
//     off_t ret = PKUFdopen(fildes, mode, len, fd);
//     unsigned long long end = ReadTime();
//     g_LibcTime += end - start;
//     printf("PKUFdopenHook: %lld, %lld\n", end - start, g_LibcTime);
//     return ret;
// }

// static int PKUOpenHook(const char *pathname, size_t len, int flags, size_t mode, int* fd)
// {
//     unsigned long long start = ReadTime();
//     int ret = PKUOpen(pathname, len, flags, mode, fd);
//     unsigned long long end = ReadTime();
//     g_LibcTime += end - start;
//     printf("PKUOpenHook: %lld, %lld\n", end - start, g_LibcTime);
//     return ret;
// }

// static int PKUStatHook(const char* filename, size_t len1, struct stat* buf, size_t len2)
// {
//     unsigned long long start = ReadTime();
//     int ret = PKUStat(filename, len1, buf, len2);
//     unsigned long long end = ReadTime();
//     g_LibcTime += end - start;
//     printf("PKUStatHook: %lld, %lld\n", end - start, g_LibcTime);
//     return ret;
// }

// static int PKUUtimeHook(const char *filename, size_t len1, const struct utimbuf *times, size_t len2)
// {
//     unsigned long long start = ReadTime();
//     int ret = PKUUtime(filename, len1, times, len2);
//     unsigned long long end = ReadTime();
//     g_LibcTime += end - start;
//     printf("PKUUtimeHook: %lld, %lld\n", end - start, g_LibcTime);
//     return ret;
// }

#endif

#define fopen(pathname, mode) FopenHook(pathname, mode)
#define fdopen(fd, mode) FdopenHook(fd, mode)
#define fmemopen(buf, size, mode) FmemopenHook(buf, size, mode)
#define fclose(stream) FcloseHook(stream)
#define fflush(stream) FflushHook(stream)
#define vfprintf(stream, format, arg) VfprintfHook(stream, format, arg)
#define vprintf(format, ap) VprintfHook(format, ap)
#define vsprintf(str, format, ap) VsprintfHook(str, format, ap)
#define vsnprintf(str, size, format, ap) VsnprintfHook(str, size, format, ap)
#define vscanf(format, ap) VscanfHook(format, ap)
#define vfscanf(stream, format, ap) VfscanfHook(stream, format, ap)
#define vsscanf(str, format, ap) VsscanfHook(str, format, ap)
#define fgetc(stream) FgetcHook(stream)
#define _IO_getc(stream) GetcHook(stream)
#define getchar() GetcharHook()
#define fputc(c, stream) FputcHook(c, stream)
#define _IO_putc(c, stream) PutcHook(c, stream)
#define putchar(c) PutcharHook(c)
#define fgets(s, n, stream) FgetsHook(s, n, stream)
#define fputs(s, stream) FputsHook(s, stream)
#define puts(s) PutsHook(s)
#define ungetc(c, stream) UngetcHook(c, stream)
#define fread(ptr, size, nmemb, stream) FreadHook(ptr, size, nmemb, stream)
#define fwrite(ptr, size, nmemb, stream) FwriteHook(ptr, size, nmemb, stream)
#define fseek(stream, offset, whence) FseekHook(stream, offset, whence)
#define ftell(stream) FtellHook(stream)
#define rewind(stream) RewindHook(stream)
#define fgetpos(stream, pos) FgetposHook(stream, pos)
#define feof(stream) FeofHook(stream)
#define ferror(stream) FerrorHook(stream)
#define perror(s) PerrorHook(s)
#define fileno(stream) FilenoHook(stream)
#define random() RandomHook()
#define srand(seed) SrandHook(seed)
// #define malloc(size) MallocHook(size)
// #define calloc(nmemb, size) CallocHook(nmemb, size)
// #define realloc(ptr, size) ReallocHook(ptr, size)
// #define free(ptr) FreeHook(ptr)
#define posix_memalign(memptr, alignment, size) PosixmemalignHook(memptr, alignment, size)
#define atexit(function) AtexitHook(function)
#define system(command) SystemHook(command)
#define utime(filename, times) UtimeHook(filename, times)
#define stat(pathname, statbuf) StatHook(pathname, statbuf)
#define lstat(pathname, statbuf) LstatHook(pathname, statbuf)
#define mkdir(pathname, mode) MkdirHook(pathname, mode)
#define access(pathname, mode) AccessHook(pathname, mode)
#define open(pathname, flags, mode) OpenHook(pathname, flags, mode)
#define close(fd) CloseHook(fd)
#define pathconf(path, name) PathconfHook(path, name)
#define isatty(fd) IsattyHook(fd)
#define unlink(pathname) UnlinkHook(pathname)
#define rmdir(pathname) RmdirHook(pathname)
#define truncate(path, length) TruncateHook(path, length)
#define ftruncate(fd, length) FtruncateHook(fd, length)
#define lseek(fd, offset, whence) LseekHook(fd, offset, whence)
#define fstat(fd, buf) FstatHook(fd, buf)
// #define fsync(fd) FsyncHook(fd)
#define fdatasync(fd) FdatasyncHook(fd)

#ifndef __cplusplus
#define read(fd, buf, count) ReadHook(fd, buf, count)
#define write(fd, buf, count) WriteHook(fd, buf, count)
#endif

#define PKUFclose(stream) PKUFcloseHook(stream)
#define PKUFflush(stream) PKUFflushHook(stream)
#define PKUFgetc(stream) PKUFgetcHook(stream)
#define PKUUngetc(c, stream) PKUUngetcHook(c, stream)
#define PKUFread(ptr, size, n, stream) PKUFreadHook(ptr, size, n, stream)
#define PKUFwrite(ptr, size, n, stream) PKUFwriteHook(ptr, size, n, stream)
#define PKUFseek(stream, offset, whence) PKUFseekHook(stream, offset, whence)
#define PKURewind(stream) PKURewindHook(stream)
#define PKUFeof(stream) PKUFeofHook(stream)
#define PKUFerror(stream) PKUFerrorHook(stream)
#define PKUFileno(stream) PKUFilenoHook(stream)
#define PKUClose(fd) PKUCloseHook(fd)
// #define PKUMalloc(size) PKUMallocHook(size)
#define PKUFopen(pathname, mode) PKUFopenHook(pathname, mode)
#define PKUFdopen(fildes, mode) PKUFdopenHook(fildes, mode)
#define PKUOpen(pathname, flags, mode) PKUOpenHook(pathname, flags, mode)
#define PKUStat(filename, buf) PKUStatHook(filename, buf)
#define PKUUtime(filename, times) PKUUtimeHook(filename, times)
#define PKULseek(fd, offset, whence) PKULseekHook(fd, offset, whence)
#define PKUFstat(fd, buf) PKUFstatHook(fd, buf)
#define PKURead(fd, buf, count) PKUReadHook(fd, buf, count)
#define PKUWrite(fd, buf, count) PKUWriteHook(fd, buf, count)
// #define PKUFsync(fd) PKUFsyncHook(fd)
#define PKUFdatasync(fd) PKUFdatasyncHook(fd)

// #define PKUFopen(pathname, len1, mode, len2, fd) PKUFopenHook(pathname, len1, mode, len2, fd)
// #define PKUFdopen(fildes, mode, len, fd) PKUFdopenHook(fildes, mode, len, fd)
// #define PKUOpen(pathname, len, flags, mode, fd) PKUOpenHook(pathname, len, flags, mode, fd)
// #define PKUStat(filename, len1, buf, len2) PKUStatHook(filename, len1, buf, len2)
// #define PKUUtime(filename, len1, times, len2) PKUUtimeHook(filename, len1, times, len2)

#endif