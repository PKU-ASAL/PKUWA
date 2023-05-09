;; smoke test with no arguments and no results
(component
  (core module $m
    (func (export ""))
  )
  (core instance $m (instantiate $m))
  (func $foo (canon lift (core func $m "")))

  (component $c
    (import "" (func $foo))

    (core func $foo (canon lower (func $foo)))
    (core module $m2
      (import "" "" (func))
      (start 0)
    )
    (core instance $m2 (instantiate $m2 (with "" (instance (export "" (func $foo))))))
  )

  (instance $c (instantiate $c (with "" (func $foo))))
)

;; boolean parameters
(component
  (core module $m
    (func (export "assert_true") (param i32)
      local.get 0
      i32.const 1
      i32.eq
      i32.eqz
      if unreachable end
    )
    (func (export "assert_false") (param i32)
      local.get 0
      if unreachable end
    )
    (func (export "ret-bool") (param i32) (result i32)
      local.get 0
    )
  )
  (core instance $m (instantiate $m))
  (func $assert_true (param bool) (canon lift (core func $m "assert_true")))
  (func $assert_false (param bool) (canon lift (core func $m "assert_false")))
  (func $ret_bool (param u32) (result bool) (canon lift (core func $m "ret-bool")))

  (component $c
    (import "assert-true" (func $assert_true (param bool)))
    (import "assert-false" (func $assert_false (param bool)))
    (import "ret-bool" (func $ret_bool (param u32) (result bool)))

    (core func $assert_true (canon lower (func $assert_true)))
    (core func $assert_false (canon lower (func $assert_false)))
    (core func $ret_bool (canon lower (func $ret_bool)))

    (core module $m2
      (import "" "assert-true" (func $assert_true (param i32)))
      (import "" "assert-false" (func $assert_false (param i32)))
      (import "" "ret-bool" (func $ret_bool (param i32) (result i32)))

      (func $start
        (call $assert_true (i32.const 1))
        (call $assert_true (i32.const 2))
        (call $assert_true (i32.const -1))
        (call $assert_false (i32.const 0))

        (if (i32.ne (call $ret_bool (i32.const 1)) (i32.const 1))
          (unreachable))
        (if (i32.ne (call $ret_bool (i32.const 2)) (i32.const 1))
          (unreachable))
        (if (i32.ne (call $ret_bool (i32.const -1)) (i32.const 1))
          (unreachable))
        (if (i32.ne (call $ret_bool (i32.const 0)) (i32.const 0))
          (unreachable))
      )
      (start $start)
    )
    (core instance $m2 (instantiate $m2
      (with "" (instance
        (export "assert-true" (func $assert_true))
        (export "assert-false" (func $assert_false))
        (export "ret-bool" (func $ret_bool))
      ))
    ))
  )

  (instance $c (instantiate $c
    (with "assert-true" (func $assert_true))
    (with "assert-false" (func $assert_false))
    (with "ret-bool" (func $ret_bool))
  ))
)

;; lots of parameters and results
(component
  (type $roundtrip (func
    ;; 20 u32 params
    (param "a1" u32) (param "a2" u32) (param "a3" u32) (param "a4" u32) (param "a5" u32)
    (param "a6" u32) (param "a7" u32) (param "a8" u32) (param "a9" u32) (param "a10" u32)
    (param "a11" u32) (param "a12" u32) (param "a13" u32) (param "a14" u32) (param "a15" u32)
    (param "a16" u32) (param "a17" u32) (param "a18" u32) (param "a19" u32) (param "a20" u32)

    ;; 10 u32 results
    (result (tuple u32 u32 u32 u32 u32 u32 u32 u32 u32 u32))
  ))

  (core module $m
    (memory (export "memory") 1)
    (func (export "roundtrip") (param $src i32) (result i32)
      (local $dst i32)
      (if (i32.ne (local.get $src) (i32.const 16))
        (unreachable))

      (if (i32.ne (i32.load offset=0 (local.get $src)) (i32.const 1)) (unreachable))
      (if (i32.ne (i32.load offset=4 (local.get $src)) (i32.const 2)) (unreachable))
      (if (i32.ne (i32.load offset=8 (local.get $src)) (i32.const 3)) (unreachable))
      (if (i32.ne (i32.load offset=12 (local.get $src)) (i32.const 4)) (unreachable))
      (if (i32.ne (i32.load offset=16 (local.get $src)) (i32.const 5)) (unreachable))
      (if (i32.ne (i32.load offset=20 (local.get $src)) (i32.const 6)) (unreachable))
      (if (i32.ne (i32.load offset=24 (local.get $src)) (i32.const 7)) (unreachable))
      (if (i32.ne (i32.load offset=28 (local.get $src)) (i32.const 8)) (unreachable))
      (if (i32.ne (i32.load offset=32 (local.get $src)) (i32.const 9)) (unreachable))
      (if (i32.ne (i32.load offset=36 (local.get $src)) (i32.const 10)) (unreachable))
      (if (i32.ne (i32.load offset=40 (local.get $src)) (i32.const 11)) (unreachable))
      (if (i32.ne (i32.load offset=44 (local.get $src)) (i32.const 12)) (unreachable))
      (if (i32.ne (i32.load offset=48 (local.get $src)) (i32.const 13)) (unreachable))
      (if (i32.ne (i32.load offset=52 (local.get $src)) (i32.const 14)) (unreachable))
      (if (i32.ne (i32.load offset=56 (local.get $src)) (i32.const 15)) (unreachable))
      (if (i32.ne (i32.load offset=60 (local.get $src)) (i32.const 16)) (unreachable))
      (if (i32.ne (i32.load offset=64 (local.get $src)) (i32.const 17)) (unreachable))
      (if (i32.ne (i32.load offset=68 (local.get $src)) (i32.const 18)) (unreachable))
      (if (i32.ne (i32.load offset=72 (local.get $src)) (i32.const 19)) (unreachable))
      (if (i32.ne (i32.load offset=76 (local.get $src)) (i32.const 20)) (unreachable))

      (local.set $dst (i32.const 500))

      (i32.store offset=0 (local.get $dst) (i32.const 21))
      (i32.store offset=4 (local.get $dst) (i32.const 22))
      (i32.store offset=8 (local.get $dst) (i32.const 23))
      (i32.store offset=12 (local.get $dst) (i32.const 24))
      (i32.store offset=16 (local.get $dst) (i32.const 25))
      (i32.store offset=20 (local.get $dst) (i32.const 26))
      (i32.store offset=24 (local.get $dst) (i32.const 27))
      (i32.store offset=28 (local.get $dst) (i32.const 28))
      (i32.store offset=32 (local.get $dst) (i32.const 29))
      (i32.store offset=36 (local.get $dst) (i32.const 30))

      local.get $dst
    )

    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      i32.const 16)
  )
  (core instance $m (instantiate $m))

  (func $roundtrip (type $roundtrip)
    (canon lift (core func $m "roundtrip") (memory $m "memory")
      (realloc (func $m "realloc")))
  )

  (component $c
    (import "roundtrip" (func $roundtrip (type $roundtrip)))

    (core module $libc
      (memory (export "memory") 1)
      (func (export "realloc") (param i32 i32 i32 i32) (result i32) unreachable)
    )
    (core instance $libc (instantiate $libc))
    (core func $roundtrip
      (canon lower (func $roundtrip)
        (memory $libc "memory")
        (realloc (func $libc "realloc")) ;; FIXME(wasm-tools#693) should not be necessary
      )
    )

    (core module $m2
      (import "libc" "memory" (memory 1))
      (import "" "roundtrip" (func $roundtrip (param i32 i32)))

      (func $start
        (local $addr i32)
        (local $retaddr i32)

        (local.set $addr (i32.const 100))
        (call $store_many (i32.const 20) (local.get $addr))

        (local.set $retaddr (i32.const 200))
        (call $roundtrip (local.get $addr) (local.get $retaddr))

        (if (i32.ne (i32.load offset=0 (local.get $retaddr)) (i32.const 21)) (unreachable))
        (if (i32.ne (i32.load offset=4 (local.get $retaddr)) (i32.const 22)) (unreachable))
        (if (i32.ne (i32.load offset=8 (local.get $retaddr)) (i32.const 23)) (unreachable))
        (if (i32.ne (i32.load offset=12 (local.get $retaddr)) (i32.const 24)) (unreachable))
        (if (i32.ne (i32.load offset=16 (local.get $retaddr)) (i32.const 25)) (unreachable))
        (if (i32.ne (i32.load offset=20 (local.get $retaddr)) (i32.const 26)) (unreachable))
        (if (i32.ne (i32.load offset=24 (local.get $retaddr)) (i32.const 27)) (unreachable))
        (if (i32.ne (i32.load offset=28 (local.get $retaddr)) (i32.const 28)) (unreachable))
        (if (i32.ne (i32.load offset=32 (local.get $retaddr)) (i32.const 29)) (unreachable))
        (if (i32.ne (i32.load offset=36 (local.get $retaddr)) (i32.const 30)) (unreachable))
      )

      (func $store_many (param $amt i32) (param $addr i32)
        (local $c i32)
        (loop $loop
          (local.set $c (i32.add (local.get $c) (i32.const 1)))
          (i32.store (local.get $addr) (local.get $c))
          (local.set $addr (i32.add (local.get $addr) (i32.const 4)))

          (if (i32.ne (local.get $amt) (local.get $c)) (br $loop))
        )
      )
      (start $start)
    )
    (core instance $m2 (instantiate $m2
      (with "libc" (instance $libc))
      (with "" (instance (export "roundtrip" (func $roundtrip))))
    ))
  )

  (instance $c (instantiate $c
    (with "roundtrip" (func $roundtrip))
  ))
)

;; this will require multiple adapter modules to get generated
(component
  (core module $root (func (export "") (result i32)
    i32.const 0
  ))
  (core instance $root (instantiate $root))
  (func $root (result u32) (canon lift (core func $root "")))

  (component $c
    (import "thunk" (func $import (result u32)))
    (core func $import (canon lower (func $import)))
    (core module $reexport
      (import "" "" (func $thunk (result i32)))
      (func (export "thunk") (result i32)
        call $thunk
        i32.const 1
        i32.add)
    )
    (core instance $reexport (instantiate $reexport
      (with "" (instance
        (export "" (func $import))
      ))
    ))
    (func $export (export "thunk") (result u32)
      (canon lift (core func $reexport "thunk"))
    )
  )

  (instance $c1 (instantiate $c (with "thunk" (func $root))))
  (instance $c2 (instantiate $c (with "thunk" (func $c1 "thunk"))))
  (instance $c3 (instantiate $c (with "thunk" (func $c2 "thunk"))))
  (instance $c4 (instantiate $c (with "thunk" (func $c3 "thunk"))))
  (instance $c5 (instantiate $c (with "thunk" (func $c4 "thunk"))))
  (instance $c6 (instantiate $c (with "thunk" (func $c5 "thunk"))))

  (component $verify
    (import "thunk" (func $thunk (result u32)))
    (core func $thunk (canon lower (func $thunk)))
    (core module $verify
      (import "" "" (func $thunk (result i32)))

      (func $start
        call $thunk
        i32.const 6
        i32.ne
        if unreachable end
      )
      (start $start)
    )
    (core instance (instantiate $verify
      (with "" (instance
        (export "" (func $thunk))
      ))
    ))
  )
  (instance (instantiate $verify (with "thunk" (func $c6 "thunk"))))
)

;; Fancy case of an adapter using an adapter. Note that this is silly and
;; doesn't actually make any sense at runtime, we just shouldn't panic on a
;; valid component.
(component
  (type $tuple20 (tuple
    u32 u32 u32 u32 u32
    u32 u32 u32 u32 u32
    u32 u32 u32 u32 u32
    u32 u32 u32 u32 u32))

  (component $realloc
    (core module $realloc
      (memory (export "memory") 1)
      (func (export "realloc") (param i32 i32 i32 i32) (result i32)
        unreachable)
    )
    (core instance $realloc (instantiate $realloc))
    (func $realloc (param (tuple u32 u32 u32 u32)) (result u32)
      (canon lift (core func $realloc "realloc"))
    )
    (export "realloc" (func $realloc))
  )
  (instance $realloc (instantiate $realloc))
  (core func $realloc (canon lower (func $realloc "realloc")))

  (core module $m
    (memory (export "memory") 1)
    (func (export "foo") (param i32))
  )
  (core instance $m (instantiate $m))
  (func $foo (param $tuple20)
    (canon lift
      (core func $m "foo")
      (memory $m "memory")
      (realloc (func $realloc))
    )
  )

  (component $c
    (import "foo" (func $foo (param $tuple20)))

    (core module $libc
      (memory (export "memory") 1)
      (func (export "realloc") (param i32 i32 i32 i32) (result i32)
        unreachable)
    )
    (core instance $libc (instantiate $libc))
    (core func $foo
      (canon lower (func $foo)
        (memory $libc "memory")
        (realloc (func $libc "realloc")) ;; FIXME(wasm-tools#693) should not be necessary
      )
    )
    (core module $something
      (import "" "foo" (func (param i32)))
    )
    (core instance (instantiate $something
      (with "" (instance
        (export "foo" (func $foo))
      ))
    ))
  )
  (instance (instantiate $c
    (with "foo" (func $foo))
  ))
)

;; Don't panic or otherwise create extraneous adapter modules when the same
;; adapter is used twice for a module's argument.
(component
  (core module $m
    (func (export "foo") (param))
  )
  (core instance $m (instantiate $m))
  (func $foo (canon lift (core func $m "foo")))

  (component $c
    (import "foo" (func $foo))
    (core func $foo (canon lower (func $foo)))

    (core module $something
      (import "" "a" (func))
      (import "" "b" (func))
    )
    (core instance (instantiate $something
      (with "" (instance
        (export "a" (func $foo))
        (export "b" (func $foo))
      ))
    ))
  )
  (instance (instantiate $c (with "foo" (func $foo))))
)

;; post-return should get invoked by the generated adapter, if specified
(component
  (core module $m
    (global $post_called (mut i32) (i32.const 0))
    (func (export "foo")
      ;; assert `foo-post` not called yet
      global.get $post_called
      i32.const 1
      i32.eq
      if unreachable end
    )
    (func (export "foo-post")
      ;; assert `foo-post` not called before
      global.get $post_called
      i32.const 1
      i32.eq
      if unreachable end
      ;; ... then flag as called
      i32.const 1
      global.set $post_called
    )
    (func (export "assert-post")
      global.get $post_called
      i32.const 1
      i32.ne
      if unreachable end
    )
  )
  (core instance $m (instantiate $m))
  (func $foo (canon lift (core func $m "foo") (post-return (func $m "foo-post"))))
  (func $assert_post (canon lift (core func $m "assert-post")))

  (component $c
    (import "foo" (func $foo))
    (import "assert-post" (func $assert_post))
    (core func $foo (canon lower (func $foo)))
    (core func $assert_post (canon lower (func $assert_post)))

    (core module $something
      (import "" "foo" (func $foo))
      (import "" "assert-post" (func $assert_post))

      (func $start
        call $foo
        call $assert_post
      )
      (start $start)
    )
    (core instance (instantiate $something
      (with "" (instance
        (export "foo" (func $foo))
        (export "assert-post" (func $assert_post))
      ))
    ))
  )
  (instance (instantiate $c
    (with "foo" (func $foo))
    (with "assert-post" (func $assert_post))
  ))
)

;; post-return passes the results
(component
  (core module $m
    (func (export "foo") (result i32) i32.const 100)
    (func (export "foo-post") (param i32)
      (if (i32.ne (local.get 0) (i32.const 100)) (unreachable)))
  )
  (core instance $m (instantiate $m))
  (func $foo (result u32)
    (canon lift (core func $m "foo") (post-return (func $m "foo-post"))))

  (component $c
    (import "foo" (func $foo (result u32)))
    (core func $foo (canon lower (func $foo)))

    (core module $something
      (import "" "foo" (func $foo (result i32)))
      (func $start
        (if (i32.ne (call $foo) (i32.const 100)) (unreachable)))
      (start $start)
    )
    (core instance (instantiate $something
      (with "" (instance
        (export "foo" (func $foo))
      ))
    ))
  )
  (instance (instantiate $c
    (with "foo" (func $foo))
  ))
)

;; struct field reordering
(component
  (component $c1
    (type $in (record
      (field "a" u32)
      (field "b" bool)
      (field "c" u8)
    ))
    (type $out (record
      (field "x" u8)
      (field "y" u32)
      (field "z" bool)
    ))

    (core module $m
      (memory (export "memory") 1)
      (func (export "r") (param i32 i32 i32) (result i32)
        (if (i32.ne (local.get 0) (i32.const 3)) (unreachable)) ;; a == 3
        (if (i32.ne (local.get 1) (i32.const 1)) (unreachable)) ;; b == true
        (if (i32.ne (local.get 2) (i32.const 2)) (unreachable)) ;; c == 2


        (i32.store8 offset=0 (i32.const 200) (i32.const 0xab)) ;; x == 0xab
        (i32.store  offset=4 (i32.const 200) (i32.const 200))  ;; y == 200
        (i32.store8 offset=8 (i32.const 200) (i32.const 0))    ;; z == false
        i32.const 200
      )
    )
    (core instance $m (instantiate $m))
    (func (export "r") (param $in) (result $out)
      (canon lift (core func $m "r") (memory $m "memory"))
    )
  )
  (component $c2
    ;; note the different field orderings than the records specified above
    (type $in (record
      (field "b" bool)
      (field "c" u8)
      (field "a" u32)
    ))
    (type $out (record
      (field "z" bool)
      (field "x" u8)
      (field "y" u32)
    ))
    (import "r" (func $r (param $in) (result $out)))
    (core module $libc (memory (export "memory") 1))
    (core instance $libc (instantiate $libc))
    (core func $r (canon lower (func $r) (memory $libc "memory")))

    (core module $m
      (import "" "r" (func $r (param i32 i32 i32 i32)))
      (import "libc" "memory" (memory 0))
      (func $start
        i32.const 100 ;; b: bool
        i32.const 2   ;; c: u8
        i32.const 3   ;; a: u32
        i32.const 100 ;; retptr
        call $r

        ;; z == false
        (if (i32.ne (i32.load8_u offset=0 (i32.const 100)) (i32.const 0)) (unreachable))
        ;; x == 0xab
        (if (i32.ne (i32.load8_u offset=1 (i32.const 100)) (i32.const 0xab)) (unreachable))
        ;; y == 200
        (if (i32.ne (i32.load offset=4 (i32.const 100)) (i32.const 200)) (unreachable))
      )
      (start $start)
    )
    (core instance (instantiate $m
      (with "libc" (instance $libc))
      (with "" (instance
        (export "r" (func $r))
      ))
    ))
  )
  (instance $c1 (instantiate $c1))
  (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
)

;; callee retptr misaligned
(assert_trap
  (component
    (component $c1
      (core module $m
        (memory (export "memory") 1)
        (func (export "r") (result i32) i32.const 1)
      )
      (core instance $m (instantiate $m))
      (func (export "r") (result (tuple u32 u32))
        (canon lift (core func $m "r") (memory $m "memory"))
      )
    )
    (component $c2
      (import "r" (func $r (result (tuple u32 u32))))
      (core module $libc (memory (export "memory") 1))
      (core instance $libc (instantiate $libc))
      (core func $r (canon lower (func $r) (memory $libc "memory")))

      (core module $m
        (import "" "r" (func $r (param i32)))
        (func $start
          i32.const 4
          call $r
        )
        (start $start)
      )
      (core instance (instantiate $m
        (with "" (instance (export "r" (func $r))))
      ))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
  )
  "unreachable")

;; caller retptr misaligned
(assert_trap
  (component
    (component $c1
      (core module $m
        (memory (export "memory") 1)
        (func (export "r") (result i32) i32.const 0)
      )
      (core instance $m (instantiate $m))
      (func (export "r") (result (tuple u32 u32))
        (canon lift (core func $m "r") (memory $m "memory"))
      )
    )
    (component $c2
      (import "r" (func $r (result (tuple u32 u32))))
      (core module $libc (memory (export "memory") 1))
      (core instance $libc (instantiate $libc))
      (core func $r (canon lower (func $r) (memory $libc "memory")))

      (core module $m
        (import "" "r" (func $r (param i32)))
        (func $start
          i32.const 1
          call $r
        )
        (start $start)
      )
      (core instance (instantiate $m
        (with "" (instance (export "r" (func $r))))
      ))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
  )
  "unreachable")

;; callee argptr misaligned
(assert_trap
  (component
    (type $big (tuple u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32))

    (component $c1
      (core module $m
        (memory (export "memory") 1)
        (func (export "r") (param i32))
        (func (export "realloc") (param i32 i32 i32 i32) (result i32)
          i32.const 1)
      )
      (core instance $m (instantiate $m))
      (func (export "r") (param $big)
        (canon lift (core func $m "r") (memory $m "memory") (realloc (func $m "realloc")))
      )
    )
    (component $c2
      (import "r" (func $r (param $big)))
      (core module $libc
        (memory (export "memory") 1)
        (func (export "realloc") (param i32 i32 i32 i32) (result i32) unreachable)
      )
      (core instance $libc (instantiate $libc))
      (core func $r
        (canon lower (func $r)
          (memory $libc "memory")
          (realloc (func $libc "realloc")) ;; FIXME(wasm-tools#693) should not be necessary
        )
      )

      (core module $m
        (import "" "r" (func $r (param i32)))
        (func $start
          i32.const 4
          call $r
        )
        (start $start)
      )
      (core instance (instantiate $m
        (with "" (instance (export "r" (func $r))))
      ))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
  )
  "unreachable")

;; caller argptr misaligned
(assert_trap
  (component
    (type $big (tuple u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32 u32))

    (component $c1
      (core module $m
        (memory (export "memory") 1)
        (func (export "r") (param i32))
        (func (export "realloc") (param i32 i32 i32 i32) (result i32)
          i32.const 4)
      )
      (core instance $m (instantiate $m))
      (func (export "r") (param $big)
        (canon lift (core func $m "r") (memory $m "memory") (realloc (func $m "realloc")))
      )
    )
    (component $c2
      (import "r" (func $r (param $big)))
      (core module $libc
        (memory (export "memory") 1)
        (func (export "realloc") (param i32 i32 i32 i32) (result i32) unreachable)
      )
      (core instance $libc (instantiate $libc))
      (core func $r
        (canon lower (func $r)
          (memory $libc "memory")
          (realloc (func $libc "realloc")) ;; FIXME(wasm-tools#693) should not be necessary
        )
      )


      (core module $m
        (import "" "r" (func $r (param i32)))
        (func $start
          i32.const 1
          call $r
        )
        (start $start)
      )
      (core instance (instantiate $m
        (with "" (instance (export "r" (func $r))))
      ))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
  )
  "unreachable")

;; simple variant translation
(component
  (type $a (variant (case "x")))
  (type $b (variant (case "y")))

  (component $c1
    (core module $m
      (func (export "r") (param i32) (result i32)
        (if (i32.ne (local.get 0) (i32.const 0)) (unreachable))
        i32.const 0
      )
    )
    (core instance $m (instantiate $m))
    (func (export "r") (param $a) (result $b) (canon lift (core func $m "r")))
  )
  (component $c2
    (import "r" (func $r (param $a) (result $b)))
    (core func $r (canon lower (func $r)))

    (core module $m
      (import "" "r" (func $r (param i32) (result i32)))
      (func $start
        i32.const 0
        call $r
        i32.const 0
        i32.ne
        if unreachable end
      )
      (start $start)
    )
    (core instance (instantiate $m
      (with "" (instance (export "r" (func $r))))
    ))
  )
  (instance $c1 (instantiate $c1))
  (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
)

;; invalid variant discriminant in a parameter
(assert_trap
  (component
    (type $a (variant (case "x")))

    (component $c1
      (core module $m
        (func (export "r") (param i32))
      )
      (core instance $m (instantiate $m))
      (func (export "r") (param $a) (canon lift (core func $m "r")))
    )
    (component $c2
      (import "r" (func $r (param $a)))
      (core func $r (canon lower (func $r)))

      (core module $m
        (import "" "r" (func $r (param i32)))
        (func $start
          i32.const 1
          call $r
        )
        (start $start)
      )
      (core instance (instantiate $m
        (with "" (instance (export "r" (func $r))))
      ))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
  )
  "unreachable")

;; invalid variant discriminant in a result
(assert_trap
  (component
    (type $a (variant (case "x")))

    (component $c1
      (core module $m
        (func (export "r") (result i32) i32.const 1)
      )
      (core instance $m (instantiate $m))
      (func (export "r") (result $a) (canon lift (core func $m "r")))
    )
    (component $c2
      (import "r" (func $r (result $a)))
      (core func $r (canon lower (func $r)))

      (core module $m
        (import "" "r" (func $r (result i32)))
        (func $start call $r drop)
        (start $start)
      )
      (core instance (instantiate $m
        (with "" (instance (export "r" (func $r))))
      ))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "r" (func $c1 "r"))))
  )
  "unreachable")


;; extra bits are chopped off
(component
  (component $c1
    (core module $m
      (func (export "u") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0)) (unreachable))
      )
      (func (export "s") (param i32)
        (if (i32.ne (local.get 0) (i32.const -1)) (unreachable))
      )
    )
    (core instance $m (instantiate $m))
    (func (export "u8") (param u8) (canon lift (core func $m "u")))
    (func (export "u16") (param u16) (canon lift (core func $m "u")))
    (func (export "s8") (param s8) (canon lift (core func $m "s")))
    (func (export "s16") (param s16) (canon lift (core func $m "s")))
  )
  (component $c2
    (import "" (instance $i
      (export "u8" (func (param u8)))
      (export "s8" (func (param s8)))
      (export "u16" (func (param u16)))
      (export "s16" (func (param s16)))
    ))

    (core func $u8 (canon lower (func $i "u8")))
    (core func $s8 (canon lower (func $i "s8")))
    (core func $u16 (canon lower (func $i "u16")))
    (core func $s16 (canon lower (func $i "s16")))

    (core module $m
      (import "" "u8" (func $u8 (param i32)))
      (import "" "s8" (func $s8 (param i32)))
      (import "" "u16" (func $u16 (param i32)))
      (import "" "s16" (func $s16 (param i32)))

      (func $start
        (call $u8 (i32.const 0))
        (call $u8 (i32.const 0xff00))
        (call $s8 (i32.const -1))
        (call $s8 (i32.const 0xff))
        (call $s8 (i32.const 0xffff))

        (call $u16 (i32.const 0))
        (call $u16 (i32.const 0xff0000))
        (call $s16 (i32.const -1))
        (call $s16 (i32.const 0xffff))
        (call $s16 (i32.const 0xffffff))
      )
      (start $start)
    )
    (core instance (instantiate $m
      (with "" (instance
        (export "u8" (func $u8))
        (export "s8" (func $s8))
        (export "u16" (func $u16))
        (export "s16" (func $s16))
      ))
    ))
  )
  (instance $c1 (instantiate $c1))
  (instance $c2 (instantiate $c2 (with "" (instance $c1))))
)

;; translation of locals between different types
(component
  (type $a (variant (case "a" u8) (case "b" float32)))
  (type $b (variant (case "a" u16) (case "b" s64)))
  (type $c (variant (case "a" u64) (case "b" float64)))
  (type $d (variant (case "a" float32) (case "b" float64)))
  (type $e (variant (case "a" float32) (case "b" s64)))

  (type $func_a (func (param "x" bool) (param "a" $a)))
  (type $func_b (func (param "x" bool) (param "b" $b)))
  (type $func_c (func (param "x" bool) (param "c" $c)))
  (type $func_d (func (param "x" bool) (param "d" $d)))
  (type $func_e (func (param "x" bool) (param "e" $d)))

  (component $c1
    (core module $m
      (func (export "a") (param i32 i32 i32)
        (i32.eqz (local.get 0))
        if
          (if (i32.ne (local.get 1) (i32.const 0)) (unreachable))
          (if (i32.ne (local.get 2) (i32.const 2)) (unreachable))
        else
          (if (i32.ne (local.get 1) (i32.const 1)) (unreachable))
          (if (f32.ne (f32.reinterpret_i32 (local.get 2)) (f32.const 3)) (unreachable))
        end
      )
      (func (export "b") (param i32 i32 i64)
        (i32.eqz (local.get 0))
        if
          (if (i32.ne (local.get 1) (i32.const 0)) (unreachable))
          (if (i64.ne (local.get 2) (i64.const 4)) (unreachable))
        else
          (if (i32.ne (local.get 1) (i32.const 1)) (unreachable))
          (if (i64.ne (local.get 2) (i64.const 5)) (unreachable))
        end
      )
      (func (export "c") (param i32 i32 i64)
        (i32.eqz (local.get 0))
        if
          (if (i32.ne (local.get 1) (i32.const 0)) (unreachable))
          (if (i64.ne (local.get 2) (i64.const 6)) (unreachable))
        else
          (if (i32.ne (local.get 1) (i32.const 1)) (unreachable))
          (if (f64.ne (f64.reinterpret_i64 (local.get 2)) (f64.const 7)) (unreachable))
        end
      )
      (func (export "d") (param i32 i32 i64)
        (i32.eqz (local.get 0))
        if
          (if (i32.ne (local.get 1) (i32.const 0)) (unreachable))
          (if (f32.ne (f32.reinterpret_i32 (i32.wrap_i64 (local.get 2))) (f32.const 8)) (unreachable))
        else
          (if (i32.ne (local.get 1) (i32.const 1)) (unreachable))
          (if (f64.ne (f64.reinterpret_i64 (local.get 2)) (f64.const 9)) (unreachable))
        end
      )
      (func (export "e") (param i32 i32 i64)
        (i32.eqz (local.get 0))
        if
          (if (i32.ne (local.get 1) (i32.const 0)) (unreachable))
          (if (f32.ne (f32.reinterpret_i32 (i32.wrap_i64 (local.get 2))) (f32.const 10)) (unreachable))
        else
          (if (i32.ne (local.get 1) (i32.const 1)) (unreachable))
          (if (i64.ne (local.get 2) (i64.const 11)) (unreachable))
        end
      )
    )
    (core instance $m (instantiate $m))
    (func (export "a") (type $func_a) (canon lift (core func $m "a")))
    (func (export "b") (type $func_b) (canon lift (core func $m "b")))
    (func (export "c") (type $func_c) (canon lift (core func $m "c")))
    (func (export "d") (type $func_d) (canon lift (core func $m "d")))
    (func (export "e") (type $func_e) (canon lift (core func $m "e")))
  )
  (component $c2
    (import "" (instance $i
      (export "a" (func (type $func_a)))
      (export "b" (func (type $func_b)))
      (export "c" (func (type $func_c)))
      (export "d" (func (type $func_d)))
      (export "e" (func (type $func_e)))
    ))

    (core func $a (canon lower (func $i "a")))
    (core func $b (canon lower (func $i "b")))
    (core func $c (canon lower (func $i "c")))
    (core func $d (canon lower (func $i "d")))
    (core func $e (canon lower (func $i "e")))

    (core module $m
      (import "" "a" (func $a (param i32 i32 i32)))
      (import "" "b" (func $b (param i32 i32 i64)))
      (import "" "c" (func $c (param i32 i32 i64)))
      (import "" "d" (func $d (param i32 i32 i64)))
      (import "" "e" (func $e (param i32 i32 i64)))

      (func $start
                                                ;; upper bits should get masked
        (call $a (i32.const 0) (i32.const 0) (i32.const 0xff_02))
        (call $a (i32.const 1) (i32.const 1) (i32.reinterpret_f32 (f32.const 3)))

                                                ;; upper bits should get masked
        (call $b (i32.const 0) (i32.const 0) (i64.const 0xff_00_04))
        (call $b (i32.const 1) (i32.const 1) (i64.const 5))

        (call $c (i32.const 0) (i32.const 0) (i64.const 6))
        (call $c (i32.const 1) (i32.const 1) (i64.reinterpret_f64 (f64.const 7)))

        (call $d (i32.const 0) (i32.const 0) (i64.extend_i32_u (i32.reinterpret_f32 (f32.const 8))))
        (call $d (i32.const 1) (i32.const 1) (i64.reinterpret_f64 (f64.const 9)))

        (call $e (i32.const 0) (i32.const 0) (i64.extend_i32_u (i32.reinterpret_f32 (f32.const 10))))
        (call $e (i32.const 1) (i32.const 1) (i64.const 11))
      )
      (start $start)
    )
    (core instance (instantiate $m
      (with "" (instance
        (export "a" (func $a))
        (export "b" (func $b))
        (export "c" (func $c))
        (export "d" (func $d))
        (export "e" (func $e))
      ))
    ))
  )
  (instance $c1 (instantiate $c1))
  (instance $c2 (instantiate $c2 (with "" (instance $c1))))
)

;; different size variants
(component
  (type $a (variant
    (case "a")
    (case "b" float32)
    (case "c" (tuple float32 u32))
    (case "d" (tuple float32 (record)  u64 u8))
  ))

  (component $c1
    (core module $m
      (func (export "a") (param i32 i32 f32 i64 i32)
        (if (i32.eq (local.get 0) (i32.const 0))
          (block
            (if (i32.ne (local.get 1) (i32.const 0)) (unreachable))
            (if (f32.ne (local.get 2) (f32.const 0)) (unreachable))
            (if (i64.ne (local.get 3) (i64.const 0)) (unreachable))
            (if (i32.ne (local.get 4) (i32.const 0)) (unreachable))
          )
        )
        (if (i32.eq (local.get 0) (i32.const 1))
          (block
            (if (i32.ne (local.get 1) (i32.const 1)) (unreachable))
            (if (f32.ne (local.get 2) (f32.const 1)) (unreachable))
            (if (i64.ne (local.get 3) (i64.const 0)) (unreachable))
            (if (i32.ne (local.get 4) (i32.const 0)) (unreachable))
          )
        )
        (if (i32.eq (local.get 0) (i32.const 2))
          (block
            (if (i32.ne (local.get 1) (i32.const 2)) (unreachable))
            (if (f32.ne (local.get 2) (f32.const 2)) (unreachable))
            (if (i64.ne (local.get 3) (i64.const 2)) (unreachable))
            (if (i32.ne (local.get 4) (i32.const 0)) (unreachable))
          )
        )
        (if (i32.eq (local.get 0) (i32.const 3))
          (block
            (if (i32.ne (local.get 1) (i32.const 3)) (unreachable))
            (if (f32.ne (local.get 2) (f32.const 3)) (unreachable))
            (if (i64.ne (local.get 3) (i64.const 3)) (unreachable))
            (if (i32.ne (local.get 4) (i32.const 3)) (unreachable))
          )
        )
        (if (i32.gt_u (local.get 0) (i32.const 3))
          (unreachable))
      )
    )
    (core instance $m (instantiate $m))
    (func (export "a") (param "x" u8) (param "a" $a) (canon lift (core func $m "a")))
  )
  (component $c2
    (import "" (instance $i
      (export "a" (func (param "x" u8) (param "a" $a)))
    ))

    (core func $a (canon lower (func $i "a")))

    (core module $m
      (import "" "a" (func $a (param i32 i32 f32 i64 i32)))

      (func $start
        ;; variant a
        (call $a
          (i32.const 0)
          (i32.const 0)
          (f32.const 0)
          (i64.const 0)
          (i32.const 0))
        ;; variant b
        (call $a
          (i32.const 1)
          (i32.const 1)
          (f32.const 1)
          (i64.const 0)
          (i32.const 0))
        ;; variant c
        (call $a
          (i32.const 2)
          (i32.const 2)
          (f32.const 2)
          (i64.const 2)
          (i32.const 0))
        ;; variant d
        (call $a
          (i32.const 3)
          (i32.const 3)
          (f32.const 3)
          (i64.const 3)
          (i32.const 3))
      )
      (start $start)
    )
    (core instance (instantiate $m
      (with "" (instance
        (export "a" (func $a))
      ))
    ))
  )
  (instance $c1 (instantiate $c1))
  (instance $c2 (instantiate $c2 (with "" (instance $c1))))
)

;; roundtrip some valid chars
(component
  (component $c1
    (core module $m
      (func (export "a") (param i32) (result i32) local.get 0)
    )
    (core instance $m (instantiate $m))
    (func (export "a") (param char) (result char) (canon lift (core func $m "a")))
  )
  (component $c2
    (import "" (instance $i
      (export "a" (func (param char) (result char)))
    ))

    (core func $a (canon lower (func $i "a")))

    (core module $m
      (import "" "a" (func $a (param i32) (result i32)))

      (func $start
        (call $roundtrip (i32.const 0))
        (call $roundtrip (i32.const 0xab))
        (call $roundtrip (i32.const 0xd7ff))
        (call $roundtrip (i32.const 0xe000))
        (call $roundtrip (i32.const 0x10ffff))
      )
      (func $roundtrip (export "roundtrip") (param i32)
        local.get 0
        call $a
        local.get 0
        i32.ne
        if unreachable end
      )
      (start $start)
    )
    (core instance $m (instantiate $m
      (with "" (instance
        (export "a" (func $a))
      ))
    ))

    (func (export "roundtrip") (param char) (canon lift (core func $m "roundtrip")))
  )
  (instance $c1 (instantiate $c1))
  (instance $c2 (instantiate $c2 (with "" (instance $c1))))

  (export "roundtrip" (func $c2 "roundtrip"))
)

(assert_return (invoke "roundtrip" (char.const "x")))
(assert_return (invoke "roundtrip" (char.const "⛳")))
(assert_return (invoke "roundtrip" (char.const "🍰")))

;; invalid chars
(assert_trap
  (component
    (component $c1
      (core module $m (func (export "a") (param i32)))
      (core instance $m (instantiate $m))
      (func (export "a") (param char) (canon lift (core func $m "a")))
    )
    (component $c2
      (import "" (instance $i (export "a" (func (param char)))))
      (core func $a (canon lower (func $i "a")))
      (core module $m
        (import "" "a" (func $a (param i32)))
        (func $start (call $a (i32.const 0xd800)))
        (start $start)
      )
      (core instance (instantiate $m (with "" (instance (export "a" (func $a))))))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "" (instance $c1))))
  )
  "unreachable")
(assert_trap
  (component
    (component $c1
      (core module $m (func (export "a") (param i32)))
      (core instance $m (instantiate $m))
      (func (export "a") (param char) (canon lift (core func $m "a")))
    )
    (component $c2
      (import "" (instance $i (export "a" (func (param char)))))
      (core func $a (canon lower (func $i "a")))
      (core module $m
        (import "" "a" (func $a (param i32)))
        (func $start (call $a (i32.const 0xdfff)))
        (start $start)
      )
      (core instance (instantiate $m (with "" (instance (export "a" (func $a))))))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "" (instance $c1))))
  )
  "unreachable")
(assert_trap
  (component
    (component $c1
      (core module $m (func (export "a") (param i32)))
      (core instance $m (instantiate $m))
      (func (export "a") (param char) (canon lift (core func $m "a")))
    )
    (component $c2
      (import "" (instance $i (export "a" (func (param char)))))
      (core func $a (canon lower (func $i "a")))
      (core module $m
        (import "" "a" (func $a (param i32)))
        (func $start (call $a (i32.const 0x110000)))
        (start $start)
      )
      (core instance (instantiate $m (with "" (instance (export "a" (func $a))))))
    )
    (instance $c1 (instantiate $c1))
    (instance $c2 (instantiate $c2 (with "" (instance $c1))))
  )
  "unreachable")

;; test that flags get their upper bits all masked off
(component
  (type $f0 (flags))
  (type $f1 (flags "f1"))
  (type $f8 (flags "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"))
  (type $f9 (flags "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8" "f9"))
  (type $f16 (flags
    "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"
    "g1" "g2" "g3" "g4" "g5" "g6" "g7" "g8"
  ))
  (type $f17 (flags
    "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"
    "g1" "g2" "g3" "g4" "g5" "g6" "g7" "g8"
    "g9"
  ))
  (type $f32 (flags
    "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"
    "g1" "g2" "g3" "g4" "g5" "g6" "g7" "g8"
    "h1" "h2" "h3" "h4" "h5" "h6" "h7" "h8"
    "i1" "i2" "i3" "i4" "i5" "i6" "i7" "i8"
  ))
  (type $f33 (flags
    "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"
    "g1" "g2" "g3" "g4" "g5" "g6" "g7" "g8"
    "h1" "h2" "h3" "h4" "h5" "h6" "h7" "h8"
    "i1" "i2" "i3" "i4" "i5" "i6" "i7" "i8"
    "i9"
  ))
  (type $f64 (flags
    "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"
    "g1" "g2" "g3" "g4" "g5" "g6" "g7" "g8"
    "h1" "h2" "h3" "h4" "h5" "h6" "h7" "h8"
    "i1" "i2" "i3" "i4" "i5" "i6" "i7" "i8"
    "j1" "j2" "j3" "j4" "j5" "j6" "j7" "j8"
    "k1" "k2" "k3" "k4" "k5" "k6" "k7" "k8"
    "l1" "l2" "l3" "l4" "l5" "l6" "l7" "l8"
    "m1" "m2" "m3" "m4" "m5" "m6" "m7" "m8"
  ))
  (type $f65 (flags
    "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8"
    "g1" "g2" "g3" "g4" "g5" "g6" "g7" "g8"
    "h1" "h2" "h3" "h4" "h5" "h6" "h7" "h8"
    "i1" "i2" "i3" "i4" "i5" "i6" "i7" "i8"
    "j1" "j2" "j3" "j4" "j5" "j6" "j7" "j8"
    "k1" "k2" "k3" "k4" "k5" "k6" "k7" "k8"
    "l1" "l2" "l3" "l4" "l5" "l6" "l7" "l8"
    "m1" "m2" "m3" "m4" "m5" "m6" "m7" "m8"
    "m9"
  ))

  (component $c1
    (core module $m
      (func (export "f0"))
      (func (export "f1") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0x1)) (unreachable))
      )
      (func (export "f8") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0x11)) (unreachable))
      )
      (func (export "f9") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0x111)) (unreachable))
      )
      (func (export "f16") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0x1111)) (unreachable))
      )
      (func (export "f17") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0x11111)) (unreachable))
      )
      (func (export "f32") (param i32)
        (if (i32.ne (local.get 0) (i32.const 0x11111111)) (unreachable))
      )
      (func (export "f33") (param i32 i32)
        (if (i32.ne (local.get 0) (i32.const 0x11111111)) (unreachable))
        (if (i32.ne (local.get 1) (i32.const 0x1)) (unreachable))
      )
      (func (export "f64") (param i32 i32)
        (if (i32.ne (local.get 0) (i32.const 0x11111111)) (unreachable))
        (if (i32.ne (local.get 1) (i32.const 0x11111111)) (unreachable))
      )
      (func (export "f65") (param i32 i32 i32)
        (if (i32.ne (local.get 0) (i32.const 0x11111111)) (unreachable))
        (if (i32.ne (local.get 1) (i32.const 0x11111111)) (unreachable))
        (if (i32.ne (local.get 2) (i32.const 0x1)) (unreachable))
      )
    )
    (core instance $m (instantiate $m))
    (func (export "f0") (param $f0) (canon lift (core func $m "f0")))
    (func (export "f1") (param $f1) (canon lift (core func $m "f1")))
    (func (export "f8") (param $f8) (canon lift (core func $m "f8")))
    (func (export "f9") (param $f9) (canon lift (core func $m "f9")))
    (func (export "f16") (param $f16) (canon lift (core func $m "f16")))
    (func (export "f17") (param $f17) (canon lift (core func $m "f17")))
    (func (export "f32") (param $f32) (canon lift (core func $m "f32")))
    (func (export "f33") (param $f33) (canon lift (core func $m "f33")))
    (func (export "f64") (param $f64) (canon lift (core func $m "f64")))
    (func (export "f65") (param $f65) (canon lift (core func $m "f65")))
  )
  (instance $c1 (instantiate $c1))

  (component $c2
    (import "" (instance $i
      (export "f0" (func (param $f0)))
      (export "f1" (func (param $f1)))
      (export "f8" (func (param $f8)))
      (export "f9" (func (param $f9)))
      (export "f16" (func (param $f16)))
      (export "f17" (func (param $f17)))
      (export "f32" (func (param $f32)))
      (export "f33" (func (param $f33)))
      (export "f64" (func (param $f64)))
      (export "f65" (func (param $f65)))
    ))
    (core func $f0 (canon lower (func $i "f0")))
    (core func $f1 (canon lower (func $i "f1")))
    (core func $f8 (canon lower (func $i "f8")))
    (core func $f9 (canon lower (func $i "f9")))
    (core func $f16 (canon lower (func $i "f16")))
    (core func $f17 (canon lower (func $i "f17")))
    (core func $f32 (canon lower (func $i "f32")))
    (core func $f33 (canon lower (func $i "f33")))
    (core func $f64 (canon lower (func $i "f64")))
    (core func $f65 (canon lower (func $i "f65")))

    (core module $m
      (import "" "f0" (func $f0))
      (import "" "f1" (func $f1 (param i32)))
      (import "" "f8" (func $f8 (param i32)))
      (import "" "f9" (func $f9 (param i32)))
      (import "" "f16" (func $f16 (param i32)))
      (import "" "f17" (func $f17 (param i32)))
      (import "" "f32" (func $f32 (param i32)))
      (import "" "f33" (func $f33 (param i32 i32)))
      (import "" "f64" (func $f64 (param i32 i32)))
      (import "" "f65" (func $f65 (param i32 i32 i32)))

      (func $start
        (call $f0)
        (call $f1 (i32.const 0xffffff01))
        (call $f8 (i32.const 0xffffff11))
        (call $f9 (i32.const 0xffffff11))
        (call $f16 (i32.const 0xffff1111))
        (call $f17 (i32.const 0xffff1111))
        (call $f32 (i32.const 0x11111111))
        (call $f33 (i32.const 0x11111111) (i32.const 0xffffffff))
        (call $f64 (i32.const 0x11111111) (i32.const 0x11111111))
        (call $f65 (i32.const 0x11111111) (i32.const 0x11111111) (i32.const 0xffffffff))
      )

      (start $start)
    )
    (core instance $m (instantiate $m
      (with "" (instance
        (export "f0" (func $f0))
        (export "f1" (func $f1))
        (export "f8" (func $f8))
        (export "f9" (func $f9))
        (export "f16" (func $f16))
        (export "f17" (func $f17))
        (export "f32" (func $f32))
        (export "f33" (func $f33))
        (export "f64" (func $f64))
        (export "f65" (func $f65))
      ))
    ))
  )
  (instance (instantiate $c2 (with "" (instance $c1))))
)

;; Adapters are used slightly out-of-order here to stress the internals of
;; dependencies between adapters.
(component
  (core module $m
    (func (export "execute"))
    (func (export "realloc") (param i32 i32 i32 i32) (result i32) unreachable)
    (memory (export "memory") 1)
  )

  (component $root
    (core instance $m (instantiate $m))
    (func (export "execute")
      (canon lift (core func $m "execute"))
    )
  )
  (component $c
    (import "backend" (instance $i
      (export "execute" (func))
    ))
    (core module $shim2 (import "" "0" (func)))
    (core instance $m (instantiate $m))

    ;; This adapter, when fused with itself on the second instantiation of this
    ;; component, will dependend on the prior instance `$m` so it which means
    ;; that the adapter module containing this must be placed in the right
    ;; location.
    (core func $execute
      (canon lower (func $i "execute") (memory $m "memory") (realloc (func $m "realloc")))
    )
    (core instance (instantiate $shim2
      (with "" (instance
        (export "0" (func $execute))
      ))
    ))
    (func (export "execute") (canon lift (core func $m "execute")))
  )
  (instance $root (instantiate $root))
  (instance $c1 (instantiate $c (with "backend" (instance $root))))
  (instance $c2 (instantiate $c (with "backend" (instance $c1))))
)
