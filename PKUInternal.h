#ifndef _PKU_INTERNAL_H_
#define _PKU_INTERNAL_H_

#define NUM_DOMAINS 16

#ifdef __cplusplus
extern "C"
{
#endif

unsigned int GetCurrentDid();
int SetCurrentDid(unsigned int did);

#ifdef __cplusplus
}
#endif

#endif