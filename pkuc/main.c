#include <stdio.h>
#include "pku.h"

GENPKU(func);

int func()
{
    void* temp = malloc(8);
    free(temp);
    return 0;
}

int init()
{
    int domain = PKUCreateDomain(0);
    int ret = PKUDomainAssignPkey(domain, 0, 0, 0);
    if(ret < 0)
    {
        perror("PKUDomainAssignPkey");
    }
    ret = PKU_CALL_REGISTER(domain, func);
    if(ret < 0)
    {
        perror("PKU_CALL_REGISTER");
    }
    return ret;
}

int main(void)
{
    printf("%x\n", ReadPKRU());
    init();
    PKUCALL(func());
    printf("%x\n", ReadPKRU());
    return 0;
}