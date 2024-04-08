#include "PKUInternal.h"

unsigned int CURRENT_DID = 0;

unsigned int GetCurrentDid()
{
    return CURRENT_DID;
}

int SetCurrentDid(unsigned int did)
{
    CURRENT_DID = did;
    return 0;
}