(module
  (import "wasi_unstable" "proc_exit"
    (func $__wasi_proc_exit (param i32)))
  (memory (export "memory") 0)
  (func $_start
    (call $__wasi_proc_exit (i32.const 125))
  )
  (export "_start" (func $_start))
)
