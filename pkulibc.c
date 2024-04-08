#include <pwd.h>
#include <grp.h>
#include <unistd.h>
#include <sys/time.h>
#include <sys/resource.h>

struct passwd* getpwnamhook(const char* name)
{
    return getpwnam(name);
}

struct group* getgrnamhook(const char* name)
{
    return getgrnam(name);
}

int setpriorityhook(int which, int who, int prio)
{
    return setpriority(which, who, prio);
}

int setrlimithook(int resource, const struct rlimit* rlim)
{
    return setrlimit(resource, rlim);
}

int getrlimithook(int resource, struct rlimit* rlim)
{
    return getrlimit(resource, rlim);
}

int initgroupshook(const char* user, gid_t group)
{
    return initgroups(user, group);
}

int chownhook(const char* pathname, uid_t owner, gid_t group)
{
    return chown(pathname, owner, group);
}