## TODO
4、检查alloc是否可以实现wasm 线性内存的分配：https://adlrocha.substack.com/p/adlrocha-playing-with-wasmtime-and


将以上问题解决之后，然后将fn wasi_for_dynlib()添加到preview_1.rs中

## 疑问
1、pkucreatedomain()的flags参数是什么意思？是domain号吗？
暂时留着了，但是还没有用（除了）