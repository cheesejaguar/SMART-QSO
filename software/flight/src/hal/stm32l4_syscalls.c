/**
 * @file stm32l4_syscalls.c
 * @brief newlib system calls for the bare-metal image (target only, SW-01)
 *
 * The flight modules log through printf/fprintf. On the target there is no
 * console yet, so output is discarded; a debug UART is a later driver.
 * There is no file system, so opens and removes fail and the caller falls back.
 *
 * There is no heap: _sbrk always fails, so any malloc in the C library
 * returns NULL instead of taking memory (JPL rule 3; the flight objects
 * themselves are checked by the no_dynamic_memory test). newlib-nano's stdio
 * then runs unbuffered.
 *
 * The leading underscores are the names newlib calls; they cannot change.
 * Not run on hardware.
 */

#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <sys/stat.h>

int _write(int fd, const char *buf, int len);
int _read(int fd, char *buf, int len);
int _open(const char *path, int flags, int mode);
int _close(int fd);
int _unlink(const char *path);
int _fstat(int fd, struct stat *st);
int _isatty(int fd);
int _lseek(int fd, int offset, int whence);
void *_sbrk(ptrdiff_t increment);
int _kill(int pid, int sig);
int _getpid(void);
void _exit(int status);

/* Stack-smashing protection (-fstack-protector-strong) */
extern uintptr_t __stack_chk_guard;
uintptr_t __stack_chk_guard = 0x595E9FBDU;
void __stack_chk_fail(void);

int _write(int fd, const char *buf, int len)
{
    (void)fd;
    (void)buf;
    return len;   /* discarded: no console */
}

int _read(int fd, char *buf, int len)
{
    (void)fd;
    (void)buf;
    (void)len;
    return 0;
}

int _open(const char *path, int flags, int mode)
{
    (void)path;
    (void)flags;
    (void)mode;
    errno = ENOENT;
    return -1;
}

int _close(int fd)
{
    (void)fd;
    errno = EBADF;
    return -1;
}

int _unlink(const char *path)
{
    (void)path;
    errno = ENOENT;
    return -1;   /* no file system */
}

int _fstat(int fd, struct stat *st)
{
    (void)fd;
    if (st != NULL) {
        st->st_mode = S_IFCHR;
    }
    return 0;
}

int _isatty(int fd)
{
    (void)fd;
    return 1;
}

int _lseek(int fd, int offset, int whence)
{
    (void)fd;
    (void)offset;
    (void)whence;
    return 0;
}

void *_sbrk(ptrdiff_t increment)
{
    (void)increment;
    errno = ENOMEM;
    return (void *)-1;   /* no heap */
}

int _kill(int pid, int sig)
{
    (void)pid;
    (void)sig;
    errno = EINVAL;
    return -1;
}

int _getpid(void)
{
    return 1;
}

void _exit(int status)
{
    (void)status;
    for (;;) {
        /* The watchdog resets the chip */
    }
}

void __stack_chk_fail(void)
{
    for (;;) {
        /* Stack corrupted: wait for the watchdog */
    }
}
