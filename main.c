// #include <stdio.h>
// #include "pku.h"

// GENPKU(lhw, int a, int b);

// int lhw(int a, int b)
// {
//     int c = a + b;
//     return c;
// }

// int init()
// {
//     int domain = PKUCreateDomain(0);
//     int ret = PKUDomainAssignPkey(domain, 0, 0, 0);
//     if(ret < 0)
//     {
//         perror("PKUDomainAssignPkey");
//     }
//     ret = PKU_CALL_REGISTER(domain, lhw);
//     if(ret < 0)
//     {
//         perror("PKU_CALL_REGISTER");
//     }
//     return ret;
// }

// int main(void)
// {
//     // printf("%x\n", ReadPkru());
//     init();
//     int ret = PKUCALL(lhw(1, 2));
//     printf("ret = %d\n", ret);
//     // printf("%x\n", ReadPkru());
//     return 0;
// }

// #include "libchook.h"

// int main(int argc, char **argv)
// {
//     FILE* fd = fopen("/home/lhw/wasmpku/libpku/main.wat", "rb");
//     if(fd != NULL)
//     {
//         unsigned char buffer[4];
//         size_t ret = fread(buffer, sizeof(*buffer), 4, fd);
//         if(ret != 4)
//         {
//             fprintf(stderr, "fread() failed: %zu\n", ret);
//             perror("fread");
//         }
//         else
//         {
//             printf("wat: %c%c%c%c\n", buffer[0], buffer[1], buffer[2], buffer[3]);
//         }
//         fclose(fd);
//     }
//     else
//     {
//         perror("fopen");
//     }
//     printf("%ld\n", g_LibcTime);
//     return 0;
// }

#include <stdio.h>
#include "libchook.h"

// int PKUFunc()
// {
//     unsigned long long ret = 0;
//     int fp = PKUFopen("./main.wat", 10, "r+", 2, &ret);
//     printf("PKUFunc: 0x%llx\n", ret);
//     if(ret == 0)
//     {
//         perror("fopen");
//         return 0;
//     }
//     PKUFerror(ret);
//     // unsigned char buffer[4];
//     // size_t ret = PKUFread(buffer, sizeof(*buffer), 4, fp);
//     // if(ret != 4)
//     // {
//     //     fprintf(stderr, "fread() failed: %zu\n", ret);
//     //     return 0;
//     // }
//     // printf("WAT: %c%c%c%c\n", buffer[0], buffer[1], buffer[2], buffer[3]);
//     // ret = PKUFseek(fp, 1, SEEK_SET);
//     // buffer[1] = 'w';
//     // ret = PKUFwrite(buffer, sizeof(*buffer), 4, fp);
//     // for(int i = 0; i < 100; ++i)
//     // {
//     //     PKUFerror(ret);
//     // }
//     PKUFclose(ret);
//     return 0;
// }

// int Func()
// {
//     FILE* fp = fopen("/home/lhw/wasmpku/libpku/main.wat", "r");
//     if(!fp)
//     {
//         perror("fopen");
//         return 0;
//     }
//     ferror(fp);
//     unsigned char buffer[4];
//     size_t ret = fread(buffer, sizeof(*buffer), 4, fp);
//     if(ret != 4)
//     {
//         fprintf(stderr, "fread() failed: %zu\n", ret);
//         return 0;
//     }
//     printf("WAT: %c%c%c%c\n", buffer[0], buffer[1], buffer[2], buffer[3]);
//     ret = fseek(fp, 1, SEEK_SET);
//     buffer[1] = 'w';
//     ret = fwrite(buffer, sizeof(*buffer), 4, fp);
//     // for(int i = 0; i < 100; ++i)
//     // {
//     //     ferror(fp);
//     // }
//     fclose(fp);
//     return 0;
// }

int main(void)
{
    // PKUFunc();
    // Func();
    // unsigned int* atypes = (unsigned int*)malloc(sizeof(unsigned int));
    // atypes[0] = FFI_TYPE_VOID;
    // int rc = 0;

    // int ret = NATIVELIBRARYCALL("./liblhw.so", foo, FFI_TYPE_INT, atypes, rc);
    // printf("%d\n", rc);

    // struct PKUPasswd *pwd = (struct PKUPasswd*)malloc(sizeof(struct PKUPasswd));
    // int ret = PKUGetpwnam("lhw", 3, pwd, sizeof(struct PKUPasswd));
    // printf("main: %d\n", pwd->pw_uid);

    // PKUOpen("./main.wat", O_RDONLY, S_IRUSR);
    RaidenTest();
    return 0;
}