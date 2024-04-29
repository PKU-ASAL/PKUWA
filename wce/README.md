## TODO
1、涉及void *的全局变量的共享，Mutex没有实现void *的sync特征，所以还有点问题

2、原本的C代码中出现了error = WASI()调用，这是一个wasi调用，如何修改到rust中？

3、rdpkru()和wrpkru()这两个函数是是不是需要分别调用ecx和eax寄存，如何实现？

4、检查alloc是否可以实现wasm 线性内存的分配：https://adlrocha.substack.com/p/adlrocha-playing-with-wasmtime-and


将以上问题解决之后，然后将fn wasi_for_dynlib()添加到preview_1.rs中

## 疑问
1、pkucreatedomain()的flags参数是什么意思？是domain号吗？
