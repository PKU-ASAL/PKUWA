#include <stdarg.h>
#include "libchook.h"

unsigned long long g_LibcTime = 0;

inline unsigned long long GetLibcTime()
{
    return g_LibcTime;
}

// int ClearCallData(unsigned int NumArgs, void** ArgValue)
// {
//     for(int i = 0; i < NumArgs; ++i)
//     {
//         free(ArgValue[i]);
//     }
//     free(ArgValue);
//     return 0;
// }

// int SetArgValue(unsigned int ArgType, void** ArgValue, void* arg)
// {
//     switch(ArgType)
//     {
//         case (FFI_TYPE_UINT8):
//         {
//             *ArgValue = malloc(sizeof(uint8_t));
//             *(uint8_t*)(*ArgValue) = (uint8_t)arg;
//             break;
//         }
//         case (FFI_TYPE_SINT8):
//         {
//             *ArgValue = malloc(sizeof(int8_t));
//             *(int8_t*)(*ArgValue) = (int8_t)arg;
//             break;
//         }
//         case (FFI_TYPE_UINT16):
//         {
//             *ArgValue = malloc(sizeof(uint16_t));
//             *(uint16_t*)(*ArgValue) = (uint16_t)arg;
//             break;
//         }
//         case (FFI_TYPE_SINT16):
//         {
//             *ArgValue = malloc(sizeof(int16_t));
//             *(int16_t*)(*ArgValue) = (int16_t)arg;
//             break;
//         }
//         case (FFI_TYPE_UINT32):
//         {
//             *ArgValue = malloc(sizeof(uint32_t));
//             *(uint32_t*)(*ArgValue) = (uint32_t)arg;
//             break;
//         }
//         case (FFI_TYPE_POINTER):
//         {
//             *ArgValue = arg;
//             break;
//         }
//         case (FFI_TYPE_SINT32):
//         case (FFI_TYPE_INT):
//         {
//             *ArgValue = malloc(sizeof(int32_t));
//             *(int32_t*)(*ArgValue) = (int32_t)arg;
//             break;
//         }
//         case (FFI_TYPE_UINT64):
//         {
//             *ArgValue = malloc(sizeof(uint64_t));
//             *(uint64_t*)(*ArgValue) = (uint64_t)arg;
//             break;
//         }
//         case (FFI_TYPE_SINT64):
//         {
//             *ArgValue = malloc(sizeof(int64_t));
//             *(int64_t*)(*ArgValue) = (int64_t)arg;
//             break;
//         }
//         case (FFI_TYPE_FLOAT):
//         {
//             *ArgValue = malloc(sizeof(float));
//             *(float*)(*ArgValue) = (float)(unsigned int)arg;
//             break;
//         }
//         case (FFI_TYPE_DOUBLE):
//         {
//             *ArgValue = malloc(sizeof(double));
//             *(double*)(*ArgValue) = (double)(unsigned long long)arg;
//             break;
//         }
//         default:
//         {
//             *ArgValue = malloc(sizeof(unsigned int));
//             *(unsigned int*)(*ArgValue) = (unsigned int)arg;
//         }
//     }
//     return 0;
// }

// int MakeArgs(int NumArgs, unsigned int* ArgTypes, void** ArgValue, ...)
// {
//     va_list ap;
//     va_start(ap, NumArgs);

//     for(int i = 0; i < NumArgs; ++i)
//     {
//         switch(ArgTypes[i])
//         {
//             case (FFI_TYPE_UINT8):
//             {
//                 ArgValue[i] = malloc(sizeof(uint8_t));
//                 *(uint8_t*)ArgValue[i] = va_arg(ap, uint8_t);
//                 break;
//             }
//             case (FFI_TYPE_SINT8):
//             {
//                 ArgValue[i] = malloc(sizeof(int8_t));
//                 *(int8_t*)ArgValue[i] = va_arg(ap, int8_t);
//                 break;
//             }
//             case (FFI_TYPE_UINT16):
//             {
//                 ArgValue[i] = malloc(sizeof(uint16_t));
//                 *(uint16_t*)ArgValue[i] = va_arg(ap, uint16_t);
//                 break;
//             }
//             case (FFI_TYPE_SINT16):
//             {
//                 ArgValue[i] = malloc(sizeof(int16_t));
//                 *(int16_t*)ArgValue[i] = va_arg(ap, int16_t);
//                 break;
//             }
//             case (FFI_TYPE_UINT32):
//             {
//                 ArgValue[i] = malloc(sizeof(uint32_t));
//                 *(uint32_t*)ArgValue[i] = va_arg(ap, uint32_t);
//                 break;
//             }
//             case (FFI_TYPE_POINTER):
//             {
//                 ArgValue[i] = va_arg(ap, void*);
//                 break;
//             }
//             case (FFI_TYPE_SINT32):
//             case (FFI_TYPE_INT):
//             {
//                 ArgValue[i] = malloc(sizeof(int32_t));
//                 *(int32_t*)ArgValue[i] = va_arg(ap, int32_t);
//                 break;
//             }
//             case (FFI_TYPE_UINT64):
//             {
//                 ArgValue[i] = malloc(sizeof(uint64_t));
//                 *(uint64_t*)ArgValue[i] = va_arg(ap, uint64_t);
//                 break;
//             }
//             case (FFI_TYPE_SINT64):
//             {
//                 ArgValue[i] = malloc(sizeof(int64_t));
//                 *(int64_t*)ArgValue[i] = va_arg(ap, int64_t);
//                 break;
//             }
//             case (FFI_TYPE_FLOAT):
//             {
//                 ArgValue[i] = malloc(sizeof(float));
//                 *(float*)ArgValue[i] = va_arg(ap, float);
//                 break;
//             }
//             case (FFI_TYPE_DOUBLE):
//             {
//                 ArgValue[i] = malloc(sizeof(double));
//                 *(double*)ArgValue[i] = va_arg(ap, double);
//                 break;
//             }
//             default:
//             {
//                 ArgValue[i] = malloc(sizeof(unsigned int));
//                 *(unsigned int*)ArgValue[i] = va_arg(ap, unsigned int);
//             }
//         }
//     }

//     va_end(ap);
//     return 0;
// }

// int SetValue(void* src, void* dst, unsigned int ValueType)
// {
//     switch(ValueType)
//     {
//         case (FFI_TYPE_UINT8):
//         {
//             *(unsigned char*)dst = *(unsigned char*)src;
//             break;
//         }
//         case (FFI_TYPE_SINT8):
//         {
//             *(char*)dst = *(char*)src;
//             break;
//         }
//         case (FFI_TYPE_UINT16):
//         {
//             *(unsigned short*)dst = *(unsigned short*)src;
//             break;
//         }
//         case (FFI_TYPE_SINT16):
//         {
//             *(short*)dst = *(short*)src;
//             break;
//         }
//         case (FFI_TYPE_UINT32):
//         {
//             *(unsigned int*)dst = *(unsigned int*)src;
//             break;
//         }
//         case (FFI_TYPE_POINTER):
//         {
//             dst = src;
//             break;
//         }
//         case (FFI_TYPE_SINT32):
//         case (FFI_TYPE_INT):
//         {
//             *(int*)dst = *(int*)src;
//             break;
//         }
//         case (FFI_TYPE_UINT64):
//         {
//             *(uint64_t*)dst = *(uint64_t*)src;
//             break;
//         }
//         case (FFI_TYPE_SINT64):
//         {
//             *(int64_t*)dst = *(int64_t*)src;
//             break;
//         }
//         case (FFI_TYPE_FLOAT):
//         {
//             *(float*)dst = *(float*)src;
//             break;
//         }
//         case (FFI_TYPE_DOUBLE):
//         {
//             *(double*)dst = *(double*)src;
//             break;
//         }
//         default:
//         {
//             dst = src;
//         }
//     }
//     return 0;
// }