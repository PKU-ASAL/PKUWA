// #include <netinet/in.h>
// #include <errno.h>

// int main(void)
// {
//     PKUFunc();
//     Func();
//     unsigned int* atypes = (unsigned int*)malloc(sizeof(unsigned int));
//     atypes[0] = FFI_TYPE_VOID;
//     int rc = 0;

//     int ret = NATIVELIBRARYCALL("./liblhw.so", foo, FFI_TYPE_INT, atypes, rc);
//     printf("%d\n", rc);

//     struct PKUPasswd *pwd = (struct PKUPasswd*)malloc(sizeof(struct PKUPasswd));
//     int ret = PKUGetpwnam("lhw", 3, pwd, sizeof(struct PKUPasswd));
//     printf("main: %d\n", pwd->pw_uid);

//     PKUOpen("./main.wat", O_RDONLY, S_IRUSR);
//     RaidenTest();
//     #ifdef _PKU_WASM
// 	printf("EWOULDBLOCK: %d\n", EWOULDBLOCK);
//     int listenfd = PKUSocket(2, 1, 0);
// 	if(-1 == listenfd)
// 	{
// 		printf("create socket error");
// 		return -1;
// 	}

// 	struct sockaddr_in bindaddr;
// 	bindaddr.sin_family = 2;
// 	bindaddr.sin_addr.s_addr = htonl(INADDR_ANY);
// 	bindaddr.sin_port = htons(6379);
// 	if(-1 == PKUBind(listenfd, (struct sockaddr*)&bindaddr, sizeof(bindaddr)))
// 	{
// 		printf("bind error");
// 		return -1;
// 	}

// 	if(PKUListen(listenfd, 2) == -1)
// 	{
// 		printf("listem error");
// 		return -1;
// 	}

// 	int maxfd;
//     while(1)
//     {
//         pku_fd_set readset;
//         PKU_FD_ZERO(&readset);
//         PKU_FD_SET(listenfd,&readset);
//         maxfd = listenfd;
//         int ret = PKUSelect(maxfd+1,&readset,NULL,NULL,NULL);
//         if(ret == -1)
//         {
//             printf("select error\n");
//         }
//         else if(ret == 0)
//         {
//             continue;
//         }
//         else
//         {
//             if(PKU_FD_ISSET(listenfd,&readset))
//             {
//                 // struct sockaddr_in clientaddr;
//                 // socklen_t clientaddrlen = sizeof(clientaddr);
// 				struct sockaddr_storage clientaddr;
//     			socklen_t clientaddrlen = sizeof(clientaddr);
//                 PKUAccept(listenfd,(struct sockaddr*)&clientaddr,&clientaddrlen);
//             }
//         }
//     }

// 	close(listenfd);
// 	return 0;
//     #else
//     int clientfd = socket(AF_INET, SOCK_STREAM, 0);
// 	if(-1 == clientfd) {
// 		printf("create socket error");
// 		return -1;
// 	}

// 	// connect server
// 	struct sockaddr_in serveraddr;
// 	serveraddr.sin_family = AF_INET;
// 	serveraddr.sin_addr.s_addr = inet_addr("127.0.0.1");;
//         serveraddr.sin_port = htons(6379);

// 	if(-1 == connect(clientfd, (struct sockaddr *)&serveraddr, sizeof(serveraddr))) {
// 		perror("connect error");
// 		return -1;
// 	}

//     char recvBuf[32] = {0};
// 	int ret = recv(clientfd, recvBuf, 32, 0);
//     return 0;
// 	#endif
// }

#include <stdio.h>
#include <stdlib.h>

// __attribute__((weak)) int PKUSharedMemory(void)
// {
//     printf("PKUShardedMemory not available in native mode\n");
//     return 0;
// }
int __imported_wasi_snapshot_preview1_PKUCreateSharedMemory(unsigned int size)
    __attribute__((__import_module__("env"), __import_name__("PKUCreateSharedMemory")));

int main(void)
{
    // extern void PKUNodeExporter(int);
    // PKUNodeExporter(0);
    //
    // extern int __imported_wasi_snapshot_preview1_PKUSharedMemory()  __attribute__((
    //     __import_module__("env"), __import_name__("PKUSharedMemory")));
    int ptr = __imported_wasi_snapshot_preview1_PKUCreateSharedMemory(1);
    printf("PKUShardedMemory called successfully: 0x%x.\n", ptr);
    int *p = (int *)ptr;
    *p = 100;
    printf("Value at pointer: %d\n", *p);
    return 0;
}

// #define _GNU_SOURCE
// #include <stdio.h>
// #include <sys/mman.h>
// #include <unistd.h>
// #include <errno.h>

// int main()
// {
//     // int pageSize = getpagesize();
//     // printf("Page size: %i\n", pageSize);

//     // // Request some memory
//     // size_t memSize = 4 * pageSize;
//     // void *mappedRegion = mmap(NULL, memSize, PROT_WRITE, MAP_SHARED | MAP_ANONYMOUS, -1, 0);
//     // int *original = (int*)(mappedRegion);
//     // *original = 55;

//     // printf("Mapped region: %p\n", mappedRegion);

//     // // Request more memory
//     // void *mappedRegion2 = mmap(NULL, 3 * pageSize, PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
//     // void *mappedRegion3 = mmap(NULL, 3 * pageSize, PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

//     // void *testRegion = (void*)((size_t)mappedRegion + pageSize);
//     // int *test = (int*)(testRegion);
//     // *test = 11;

//     // // Map shared region onto new region
//     // mremap(testRegion, 0, 3 * pageSize, MREMAP_FIXED | MREMAP_MAYMOVE, mappedRegion2);
//     // mremap(testRegion, 0, 3 * pageSize, MREMAP_FIXED | MREMAP_MAYMOVE, mappedRegion3);

//     // // Write something to original region and check it is reflected in others
//     // int *copy1 = (int*)(mappedRegion2);
//     // int *copy2 = (int*)(mappedRegion3);

//     // printf("%i  %i  %i  %i\n", *original, *test, *copy1, *copy2);

//     // // Unmap one region to anonymous
//     // munmap(mappedRegion2, 3 * pageSize);
//     // mmap(mappedRegion2, 3 * pageSize, PROT_WRITE, MAP_PRIVATE| MAP_ANONYMOUS, -1, 0);
//     // *copy1 = 99;

//     // // Update original and check value isn't passed to unmapped
//     // *original = 66;
//     // *test = 22;

//     // printf("%i  %i  %i  %i\n", *original, *test, *copy1, *copy2);
//     // 映射三次，第三次是MAP_SHARED，将前两次的映射区域mremap到第三次的映射区域上。
//     size_t initial_size = 4096;  // 初始大小为 4 KB
//     size_t expanded_size = 8192; // 扩展大小为 8 KB

//     // 创建映射区域
//     void *ptr = mmap(NULL, expanded_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
//     int *original = (int*)(ptr);
//     *original = 55;

//     printf("Mapped region: %p\n", ptr);

//     void *share = mmap(NULL, initial_size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_ANONYMOUS, -1, 0);

//     // 使用 mremap 扩展映射区域的大小
//     void *mappedRegion2 = mmap(NULL, initial_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
//     // void *testRegion = (void*)((size_t)mappedRegion2 + initial_size);

//     int *copy1 = (int*)(mappedRegion2);
//     // void *new_ptr1 = mremap(ptr, 0, initial_size, MREMAP_FIXED | MREMAP_MAYMOVE, testRegion);
//     // void *new_ptr2 = mremap(ptr, 0, initial_size, MREMAP_FIXED | MREMAP_MAYMOVE, mappedRegion3);
//     // int *copy1 = (int*)(testRegion);
//     void *new_ptr1 = mremap(share, 0, initial_size, MREMAP_FIXED | MREMAP_MAYMOVE, ptr);
//     void *new_ptr2 = mremap(share, 0, initial_size, MREMAP_FIXED | MREMAP_MAYMOVE, mappedRegion2);

//     // 使用新的映射区域进行读写操作...
//     mprotect(mappedRegion2, initial_size, PROT_READ);
//     // *copy1 = 99; // 修改映射区域的值
//     *original = 66;
//     printf("%i %i %p %p\n", *original, *copy1, ptr, mappedRegion2);

//     // 解除映射
//     munmap(new_ptr1, initial_size);
//     munmap(new_ptr2, initial_size);

//     return 0;
// }

// #include <stdio.h>
// #include <unistd.h>
// #include <string.h>
// #include <arpa/inet.h>

// int main(void)
// {
//     int sock = PKUSocket(AF_INET, SOCK_STREAM, 0);
//     struct sockaddr_in server_addr;

//     bzero(&server_addr, sizeof(server_addr));
//     server_addr.sin_family = 2;
//     server_addr.sin_addr.s_addr = INADDR_ANY;
//     server_addr.sin_port = htons(5000);

//     if (PKUBind(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0)
//     {
//         fprintf(stderr, "bind failed\n");
//         return -1;
//     }

//     if (PKUListen(sock, 128) < 0)
//     {
//         fprintf(stderr, "listen failed\n");
//         return -1;
//     }

//     int done = 1;
//     while (done)
//     {
//         struct sockaddr_in client;
//         char client_ip[64];
//         char buff[256] = {0};

//         socklen_t client_addr_len;
//         client_addr_len = sizeof(client);
//         int client_sock = PKUAccept(sock, (struct sockaddr *)&client, &client_addr_len);
//         printf("client ip: %s\t port : %d\n",
//                inet_ntop(2, &client.sin_addr.s_addr, client_ip,
//                          sizeof(client_ip)),
//                ntohs(client.sin_port));

//         read(client_sock, buff, 256);

//         char buf[1024] = {0};

//         strcat(buf, "HTTP/1.1 200 OK\r\n");

//         if (PKUSendto(client_sock, buf, strlen(buf), 0, NULL, 0) < 0)
//         {
//             fprintf(stderr, "send failed\n");
//             perror("send");
//         }

//         PKUClose(client_sock);
//         done = 0;
//     }
//     PKUClose(sock);
//     return 0;
// }